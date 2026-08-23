//! Draws an antialiased circle with Vulkan, using the safe `vulkano` wrapper and
//! `winit` for the window. The circle is a screen-space quad whose fragment shader
//! computes the distance to the center (a signed-distance-field shape), so it stays
//! perfectly round at any window size. Close the window or press Escape to exit.

use std::{error::Error, mem::size_of, sync::Arc};
use vulkano::{
    Validated, VulkanError, VulkanLibrary,
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo,
        PrimaryCommandBufferAbstract, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
        allocator::StandardCommandBufferAllocator,
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::PhysicalDeviceType,
    },
    format::Format,
    image::{Image, ImageCreateInfo, ImageUsage, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::VertexInputState,
            viewport::{Viewport, ViewportState},
        },
        layout::{PipelineDescriptorSetLayoutCreateInfo, PushConstantRange},
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    shader::{ShaderModule, ShaderModuleCreateInfo, ShaderStages, spirv},
    swapchain::{
        Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo, acquire_next_image,
    },
    sync::{self, GpuFuture},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

fn main() -> Result<(), impl Error> {
    // `--selftest`: render one frame offscreen and verify the pixels, no window needed.
    if std::env::args().any(|arg| arg == "--selftest") {
        run_selftest();
        return Ok(());
    }

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(&event_loop);

    event_loop.run_app(&mut app)
}

/// Per-frame data passed to the shaders through push constants.
#[derive(BufferContents)]
#[repr(C)]
struct CircleParams {
    // x = framebuffer width, y = framebuffer height, z = circle radius (pixels)
    data: [f32; 4],
}

struct App {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    rcx: Option<RenderContext>,
}

struct RenderContext {
    window: Arc<Window>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline_layout: Arc<PipelineLayout>,
    pipeline: Arc<GraphicsPipeline>,
    viewport: Viewport,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl App {
    fn new(event_loop: &EventLoop<()>) -> Self {
        // Load the Vulkan loader (libvulkan) provided by the system / SDK.
        let library = VulkanLibrary::new().unwrap();

        // Drawing to a window requires extra instance extensions (VK_KHR_surface and
        // friends); ask winit which ones are needed on this platform.
        let required_extensions = Surface::required_extensions(event_loop).unwrap();

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .unwrap();

        // Presenting images requires the VK_KHR_swapchain device extension.
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        // Pick a physical device that can draw AND present to our window, preferring
        // faster GPUs.
        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.presentation_support(i as u32, event_loop).unwrap()
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .expect("no suitable physical device found");

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let rcx = None;

        App {
            instance,
            device,
            queue,
            command_buffer_allocator,
            rcx,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Vulkan — Circle")
                        .with_inner_size(LogicalSize::new(900.0, 700.0)),
                )
                .unwrap(),
        );
        let surface = Surface::from_window(self.instance.clone(), window.clone()).unwrap();
        let window_size = window.inner_size();

        // Create the swapchain: the pool of images we render into and present to the screen.
        let (swapchain, images) = {
            let surface_capabilities = self
                .device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();

            let (image_format, _) = self
                .device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0];

            Swapchain::new(
                self.device.clone(),
                surface,
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    image_extent: window_size.into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap(),
                    ..Default::default()
                },
            )
            .unwrap()
        };

        // A single-subpass render pass that clears its one color attachment each frame.
        let render_pass = vulkano::single_pass_renderpass!(
            self.device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )
        .unwrap();

        // Load the SPIR-V bytecode compiled from GLSL by build.rs (glslc) and build the
        // graphics pipeline.
        let (pipeline, pipeline_layout) = create_circle_pipeline(&self.device, &render_pass);

        let framebuffers = window_size_dependent_setup(&images, &render_pass);

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window_size.into(),
            depth_range: 0.0..=1.0,
        };

        let previous_frame_end = Some(sync::now(self.device.clone()).boxed());

        self.rcx = Some(RenderContext {
            window,
            swapchain,
            render_pass,
            framebuffers,
            pipeline_layout,
            pipeline,
            viewport,
            recreate_swapchain: false,
            previous_frame_end,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(rcx) = self.rcx.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key == Key::Named(NamedKey::Escape) {
                    event_loop.exit();
                }
            }

            WindowEvent::Resized(_) => rcx.recreate_swapchain = true,

            WindowEvent::RedrawRequested => {
                let window_size = rcx.window.inner_size();

                // Skip frames while the window is minimized / has zero area.
                if window_size.width == 0 || window_size.height == 0 {
                    return;
                }

                // Free resources belonging to finished GPU work.
                rcx.previous_frame_end.as_mut().unwrap().cleanup_finished();

                if rcx.recreate_swapchain {
                    let (new_swapchain, new_images) = rcx
                        .swapchain
                        .recreate(SwapchainCreateInfo {
                            image_extent: window_size.into(),
                            ..rcx.swapchain.create_info()
                        })
                        .expect("failed to recreate swapchain");

                    rcx.swapchain = new_swapchain;
                    rcx.framebuffers = window_size_dependent_setup(&new_images, &rcx.render_pass);
                    rcx.viewport.extent = window_size.into();
                    rcx.recreate_swapchain = false;
                }

                // Take ownership of one swapchain image to draw into.
                let (image_index, suboptimal, acquire_future) = match acquire_next_image(
                    rcx.swapchain.clone(),
                    None,
                )
                .map_err(Validated::unwrap)
                {
                    Ok(r) => r,
                    Err(VulkanError::OutOfDate) => {
                        rcx.recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("failed to acquire next image: {e}"),
                };

                if suboptimal {
                    rcx.recreate_swapchain = true;
                }

                // Circle fills ~64% of the smaller window dimension.
                let radius = window_size.width.min(window_size.height) as f32 * 0.32;
                let params = CircleParams {
                    data: [
                        window_size.width as f32,
                        window_size.height as f32,
                        radius,
                        0.0,
                    ],
                };

                let mut builder = AutoCommandBufferBuilder::primary(
                    self.command_buffer_allocator.clone(),
                    self.queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.07, 0.08, 0.12, 1.0].into())],
                            ..RenderPassBeginInfo::framebuffer(
                                rcx.framebuffers[image_index as usize].clone(),
                            )
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..Default::default()
                        },
                    )
                    .unwrap()
                    .set_viewport(0, [rcx.viewport.clone()].into_iter().collect())
                    .unwrap()
                    .bind_pipeline_graphics(rcx.pipeline.clone())
                    .unwrap()
                    .push_constants(rcx.pipeline_layout.clone(), 0, params)
                    .unwrap();

                // Six vertices: two triangles forming the quad that contains the circle.
                unsafe { builder.draw(6, 1, 0, 0) }.unwrap();

                builder.end_render_pass(Default::default()).unwrap();

                let command_buffer = builder.build().unwrap();

                let future = rcx
                    .previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(self.queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        self.queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(
                            rcx.swapchain.clone(),
                            image_index,
                        ),
                    )
                    .then_signal_fence_and_flush();

                match future.map_err(Validated::unwrap) {
                    Ok(future) => {
                        rcx.previous_frame_end = Some(future.boxed());
                    }
                    Err(VulkanError::OutOfDate) => {
                        rcx.recreate_swapchain = true;
                        rcx.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
                    }
                    Err(e) => panic!("failed to flush future: {e}"),
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Render continuously.
        if let Some(rcx) = self.rcx.as_mut() {
            rcx.window.request_redraw();
        }
    }
}

/// Wraps raw SPIR-V bytes in a vulkano `ShaderModule`.
///
/// Safety: the bytes must be valid SPIR-V — guaranteed here because they were
/// produced by `glslc` at build time.
fn load_shader_module(device: Arc<Device>, spirv_bytes: &[u8]) -> Arc<ShaderModule> {
    let words = spirv::bytes_to_words(spirv_bytes).unwrap();
    unsafe { ShaderModule::new(device, ShaderModuleCreateInfo::new(&words)) }
        .map_err(Validated::unwrap)
        .unwrap()
}

/// One framebuffer per swapchain image; called on init and after resizes.
fn window_size_dependent_setup(
    images: &[Arc<Image>],
    render_pass: &Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

/// Builds the graphics pipeline that draws the circle, usable with any render pass that
/// has a compatible color attachment. Returns the pipeline and its layout (the layout is
/// needed to push constants at draw time).
fn create_circle_pipeline(
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
) -> (Arc<GraphicsPipeline>, Arc<PipelineLayout>) {
    let vs = load_shader_module(
        device.clone(),
        include_bytes!(concat!(env!("OUT_DIR"), "/circle.vert.spv")),
    );
    let fs = load_shader_module(
        device.clone(),
        include_bytes!(concat!(env!("OUT_DIR"), "/circle.frag.spv")),
    );

    let stages = [
        PipelineShaderStageCreateInfo::new(vs.entry_point("main").unwrap()),
        PipelineShaderStageCreateInfo::new(fs.entry_point("main").unwrap()),
    ];

    // `from_stages` reflects the push-constant ranges straight out of the shaders; fall
    // back to an explicit declaration only if reflection found none.
    let mut layout_create_info = PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
        .into_pipeline_layout_create_info(device.clone())
        .unwrap();
    if layout_create_info.push_constant_ranges.is_empty() {
        layout_create_info
            .push_constant_ranges
            .push(PushConstantRange {
                stages: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                offset: 0,
                size: size_of::<CircleParams>() as u32,
            });
    }
    let pipeline_layout = PipelineLayout::new(device.clone(), layout_create_info).unwrap();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    let pipeline = GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            // No vertex buffers: the quad is generated procedurally in the vertex shader,
            // so the vertex input state is empty.
            vertex_input_state: Some(VertexInputState::default()),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            // Blend using straight alpha so the feathered rim fades smoothly into
            // whatever was cleared before.
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend::alpha()),
                    ..Default::default()
                },
            )),
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(pipeline_layout.clone())
        },
    )
    .unwrap();

    (pipeline, pipeline_layout)
}

/// Renders one frame into an offscreen image on a headless Vulkan instance, reads the
/// pixels back and checks that the circle landed where it should. Run with `--selftest`.
fn run_selftest() {
    const SIZE: u32 = 512;
    const RADIUS: f32 = 160.0;
    const CLEAR: [f32; 4] = [0.07, 0.08, 0.12, 1.0];
    // vec3(1.0, 0.45, 0.15) from the fragment shader, quantized to bytes.
    const CIRCLE_RGB: [u8; 3] = [255, 115, 38];

    let library = VulkanLibrary::new().unwrap();
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            ..Default::default()
        },
    )
    .unwrap();

    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .position(|q| q.queue_flags.intersects(QueueFlags::GRAPHICS))
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .expect("no suitable physical device found");

    println!("Using device: {}", physical_device.properties().device_name);

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap();
    let queue = queues.next().unwrap();

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        Default::default(),
    ));

    let format = Format::R8G8B8A8_UNORM;
    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                format: format,
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
        },
        pass: {
            color: [color],
            depth_stencil: {},
        },
    )
    .unwrap();

    let image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            format,
            usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC,
            extent: [SIZE, SIZE, 1],
            ..Default::default()
        },
        AllocationCreateInfo::default(),
    )
    .unwrap();
    let framebuffer = Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments: vec![ImageView::new_default(image.clone()).unwrap()],
            ..Default::default()
        },
    )
    .unwrap();

    let (pipeline, pipeline_layout) = create_circle_pipeline(&device, &render_pass);

    let readback = Buffer::from_iter(
        memory_allocator,
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        (0..(SIZE * SIZE * 4)).map(|_| 0u8),
    )
    .unwrap();

    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [SIZE as f32, SIZE as f32],
        depth_range: 0.0..=1.0,
    };
    let params = CircleParams {
        data: [SIZE as f32, SIZE as f32, RADIUS, 0.0],
    };

    let mut builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![Some(CLEAR.into())],
                ..RenderPassBeginInfo::framebuffer(framebuffer)
            },
            SubpassBeginInfo {
                contents: SubpassContents::Inline,
                ..Default::default()
            },
        )
        .unwrap()
        .set_viewport(0, [viewport].into_iter().collect())
        .unwrap()
        .bind_pipeline_graphics(pipeline)
        .unwrap()
        .push_constants(pipeline_layout, 0, params)
        .unwrap();
    unsafe { builder.draw(6, 1, 0, 0) }.unwrap();

    builder.end_render_pass(Default::default()).unwrap();
    builder
        .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
            image.clone(),
            readback.clone(),
        ))
        .unwrap();

    let command_buffer = builder.build().unwrap();
    command_buffer
        .execute(queue)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    let pixels = readback.read().unwrap();
    let px = |x: u32, y: u32| {
        let i = ((y * SIZE + x) * 4) as usize;
        [pixels[i], pixels[i + 1], pixels[i + 2]]
    };

    assert!(
        px(SIZE / 2, SIZE / 2)
            .iter()
            .zip(CIRCLE_RGB)
            .all(|(a, b)| { (i32::from(*a) - i32::from(b)).abs() <= 3 }),
        "center pixel is not the circle color"
    );
    assert!(
        px(SIZE / 2 + 150, SIZE / 2)
            .iter()
            .zip(CIRCLE_RGB)
            .all(|(a, b)| { (i32::from(*a) - i32::from(b)).abs() <= 3 }),
        "pixel inside the rim is not filled"
    );
    for (name, sample) in [
        ("corner", px(4, 4)),
        ("just outside the rim", px(SIZE / 2 + 170, SIZE / 2)),
    ] {
        assert!(
            sample
                .iter()
                .zip(CLEAR)
                .all(|(a, b)| { (i32::from(*a) - (b * 255.0).round() as i32).abs() <= 3 }),
            "{name} pixel {sample:?} was not the clear color"
        );
    }

    // Optionally dump the rendered frame as a binary PPM (P6) image.
    if let Ok(path) = std::env::var("CIRCLE_PPM") {
        let mut ppm = format!("P6\n{SIZE} {SIZE}\n255\n").into_bytes();
        for px4 in pixels.chunks_exact(4) {
            ppm.extend_from_slice(&px4[..3]);
        }
        std::fs::write(&path, ppm).expect("failed to write PPM");
        println!("Frame saved to {path}");
    }

    println!("PASS: circle rendered correctly offscreen");
}
