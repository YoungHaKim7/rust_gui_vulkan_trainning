```
use ash::{
    Entry,
    khr::{surface, swapchain},
    vk,
};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::{
    ffi::CString,
    mem::size_of,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

/// Owns the Vulkan objects required to render one triangle.
///
/// # Vulkan object hierarchy
///
/// The important lifetime relationships are roughly:
///
/// `Entry -> Instance -> PhysicalDevice -> Device -> Queue`
///
/// and, for presentation:
///
/// `Instance -> Surface -> Swapchain -> Images -> ImageViews -> Framebuffers`
///
/// The graphics pipeline depends on the render pass, while command buffers
/// refer to the pipeline and the framebuffer selected for the acquired
/// swapchain image.
///
/// Vulkan handles are generally lightweight, non-owning values. The Rust
/// struct therefore acts as the owner of the corresponding Vulkan resources,
/// and `destroy()` releases them in dependency-safe reverse order.
struct VulkanApp {
    // Held to keep the Vulkan loader alive for the instance lifetime.
    #[allow(dead_code)]
    entry: Entry,
    instance: ash::Instance,

    surface_loader: surface::Instance,
    surface: vk::SurfaceKHR,

    #[allow(dead_code)]
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    queue: vk::Queue,

    swapchain_loader: swapchain::Device,
    swapchain: vk::SwapchainKHR,

    // Owned by the swapchain; kept only to document what is present.
    #[allow(dead_code)]
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_extent: vk::Extent2D,

    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,

    framebuffers: Vec<vk::Framebuffer>,

    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,

    image_available: vk::Semaphore,
    render_finished: vk::Semaphore,
    in_flight: vk::Fence,
}

impl VulkanApp {
    /// Creates all Vulkan state needed by the triangle renderer.
    ///
    /// Vulkan exposes a relatively explicit initialization model. In broad
    /// terms this function performs these steps:
    ///
    /// 1. Load the Vulkan loader (`Entry`).
    /// 2. Create a Vulkan `Instance`.
    /// 3. Create a window `Surface` that Vulkan can present to.
    /// 4. Select a physical GPU and a queue family supporting graphics and
    ///    presentation.
    /// 5. Create a logical `Device` and obtain a graphics queue.
    /// 6. Query surface capabilities and create a `Swapchain`.
    /// 7. Create image views for the swapchain images.
    /// 8. Create a render pass describing the color attachment.
    /// 9. Load Slang-generated SPIR-V and create shader modules.
    /// 10. Build the graphics pipeline.
    /// 11. Create framebuffers, command infrastructure, and synchronization.
    ///
    /// Most Vulkan functions are `unsafe` here because Vulkan's C API cannot
    /// express resource validity, synchronization, or lifetime dependencies
    /// in its type system. The surrounding Rust code establishes those
    /// invariants manually.
    unsafe fn new(window: &Window) -> Self {
        unsafe {
            let entry = Entry::load().expect("failed to load Vulkan");

            //
            // ------------------------------------------------------------
            // Vulkan Instance
            // ------------------------------------------------------------
            //
            // The instance is the root Vulkan object for this application.
            // It connects the program to the Vulkan implementation and is
            // used to discover physical devices and create surfaces.
            //

            let app_name = CString::new("Slang Triangle").unwrap();

            let app_info = vk::ApplicationInfo::default()
                .application_name(&app_name)
                .application_version(0)
                .engine_name(&app_name)
                .engine_version(0)
                // Vulkan 1.1 for PhysicalDeviceVulkan11Features (shaderDrawParameters).
                .api_version(vk::API_VERSION_1_1);

            let display = window.display_handle().expect("display handle").as_raw();

            let extension_names = ash_window::enumerate_required_extensions(display)
                .expect("required Vulkan extensions");

            let create_info = vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_extension_names(extension_names);

            let instance = entry
                .create_instance(&create_info, None)
                .expect("failed to create Vulkan instance");

            //
            // ------------------------------------------------------------
            // Window Surface
            // ------------------------------------------------------------
            //
            // A VkSurfaceKHR represents a platform window as a Vulkan
            // presentation target. The surface is owned by the instance and
            // is used later when checking presentation support and creating
            // the swapchain.
            //

            let surface = ash_window::create_surface(
                &entry,
                &instance,
                display,
                window.window_handle().expect("window handle").as_raw(),
                None,
            )
            .expect("failed to create surface");

            let surface_loader = surface::Instance::new(&entry, &instance);

            //
            // ------------------------------------------------------------
            // Physical Device (GPU)
            // ------------------------------------------------------------
            //
            // A physical device describes an actual Vulkan-capable GPU.
            // Nothing is submitted to it directly; first we create a logical
            // device that exposes the queues and features we need.
            //

            let physical_devices = instance
                .enumerate_physical_devices()
                .expect("failed to enumerate physical devices");

            let (physical_device, queue_family_index) = physical_devices
                .iter()
                .find_map(|&device| {
                    let families =
                        instance.get_physical_device_queue_family_properties(device);

                    families.iter().enumerate().find_map(|(index, family)| {
                        let graphics = family.queue_flags.contains(vk::QueueFlags::GRAPHICS);

                        let present = surface_loader
                            .get_physical_device_surface_support(device, index as u32, surface)
                            .ok()?;

                        if graphics && present {
                            Some((device, index as u32))
                        } else {
                            None
                        }
                    })
                })
                .expect("no suitable Vulkan device");

            //
            // ------------------------------------------------------------
            // Logical Device and Queue
            // ------------------------------------------------------------
            //
            // The logical device is this application's interface to the
            // selected GPU. A queue is the execution endpoint to which we
            // submit command buffers.
            //

            // slangc declares an (unused) BuiltIn BaseVertex input for
            // SV_VertexID, which pulls in the DrawParameters SPIR-V
            // capability. That capability is only legal when the
            // shaderDrawParameters device feature (Vulkan 1.1) is enabled.
            assert!(
                instance.get_physical_device_properties(physical_device).api_version
                    >= vk::API_VERSION_1_1,
                "Vulkan 1.1 is required for shaderDrawParameters"
            );

            let mut supported_vulkan11_features = vk::PhysicalDeviceVulkan11Features::default();

            let mut supported_features2 = vk::PhysicalDeviceFeatures2::default()
                .push_next(&mut supported_vulkan11_features);

            instance.get_physical_device_features2(physical_device, &mut supported_features2);

            assert!(
                supported_vulkan11_features.shader_draw_parameters == vk::TRUE,
                "shaderDrawParameters feature is not supported"
            );

            let mut enabled_features = vk::PhysicalDeviceVulkan11Features::default()
                .shader_draw_parameters(true);

            let priorities = [1.0_f32];

            let queue_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);

            let queue_infos = [queue_info];

            let device_extensions = [swapchain::NAME.as_ptr()];

            let device_create_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(&queue_infos)
                .enabled_extension_names(&device_extensions)
                .push_next(&mut enabled_features);

            let device = instance
                .create_device(physical_device, &device_create_info, None)
                .expect("failed to create logical device");

            let queue = device.get_device_queue(queue_family_index, 0);

            //
            // ------------------------------------------------------------
            // Surface Capabilities
            // ------------------------------------------------------------
            //
            // The window system constrains swapchain image count, extent,
            // format, and presentation mode. These queries let us choose a
            // configuration the selected GPU and window surface support.
            //

            let capabilities = surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .expect("surface capabilities");

            let formats = surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .expect("surface formats");

            let surface_format = formats
                .iter()
                .copied()
                .find(|format| format.format == vk::Format::B8G8R8A8_UNORM)
                .unwrap_or(formats[0]);

            let extent = if capabilities.current_extent.width != u32::MAX {
                capabilities.current_extent
            } else {
                vk::Extent2D {
                    width: WIDTH,
                    height: HEIGHT,
                }
            };

            let present_modes = surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .expect("present modes");

            let present_mode = present_modes
                .iter()
                .copied()
                .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
                .unwrap_or(vk::PresentModeKHR::FIFO);

            let image_count = capabilities.min_image_count + 1;

            let image_count = if capabilities.max_image_count > 0 {
                image_count.min(capabilities.max_image_count)
            } else {
                image_count
            };

            //
            // ------------------------------------------------------------
            // Swapchain
            // ------------------------------------------------------------
            //
            // A swapchain is a collection of images used for presentation.
            // The application renders into one acquired image while other
            // images may be displayed or waiting to be presented.
            //

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(surface)
                .min_image_count(image_count)
                .image_format(surface_format.format)
                .image_color_space(surface_format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true);

            let swapchain_loader = swapchain::Device::new(&instance, &device);

            let swapchain = swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("failed to create swapchain");

            let swapchain_images = swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("failed to get swapchain images");

            //
            // ------------------------------------------------------------
            // Swapchain Image Views
            // ------------------------------------------------------------
            //
            // A VkImage is the underlying image resource. An image view tells
            // Vulkan how a shader/render pass should interpret a portion of
            // that image. Render-pass framebuffers use these views.
            //

            let swapchain_image_views = swapchain_images
                .iter()
                .map(|&image| {
                    let components = vk::ComponentMapping::default();

                    let subresource = vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1);

                    let info = vk::ImageViewCreateInfo::default()
                        .image(image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface_format.format)
                        .components(components)
                        .subresource_range(subresource);

                    device.create_image_view(&info, None).expect("image view")
                })
                .collect::<Vec<_>>();

            //
            // ------------------------------------------------------------
            // Render Pass
            // ------------------------------------------------------------
            //
            // This render pass has one color attachment. It is cleared at
            // the beginning of the render pass, used as a color attachment,
            // and transitioned to PRESENT_SRC_KHR when rendering finishes.
            //

            let color_attachment = vk::AttachmentDescription::default()
                .format(surface_format.format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

            let color_ref = vk::AttachmentReference::default()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

            let color_refs = [color_ref];

            let subpass = vk::SubpassDescription::default()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_refs);

            let attachments = [color_attachment];
            let subpasses = [subpass];

            let render_pass_info = vk::RenderPassCreateInfo::default()
                .attachments(&attachments)
                .subpasses(&subpasses);

            let render_pass = device
                .create_render_pass(&render_pass_info, None)
                .expect("render pass");

            //
            // ------------------------------------------------------------
            // Shader Modules: Slang -> SPIR-V -> Vulkan
            // ------------------------------------------------------------
            //
            // The Slang compiler produces SPIR-V binaries during the build.
            // Vulkan consumes SPIR-V through VkShaderModule objects. The
            // shader modules are only needed while creating the pipeline, so
            // they can be destroyed immediately after pipeline creation.
            //

            let vertex_code = include_bytes!(concat!(env!("OUT_DIR"), "/triangle.vert.spv"));

            let fragment_code = include_bytes!(concat!(env!("OUT_DIR"), "/triangle.frag.spv"));

            let vertex_words = std::slice::from_raw_parts(
                vertex_code.as_ptr() as *const u32,
                vertex_code.len() / size_of::<u32>(),
            );

            let fragment_words = std::slice::from_raw_parts(
                fragment_code.as_ptr() as *const u32,
                fragment_code.len() / size_of::<u32>(),
            );

            let vertex_module_info = vk::ShaderModuleCreateInfo::default().code(vertex_words);

            let fragment_module_info = vk::ShaderModuleCreateInfo::default().code(fragment_words);

            let vertex_module = device
                .create_shader_module(&vertex_module_info, None)
                .expect("vertex shader module");

            let fragment_module = device
                .create_shader_module(&fragment_module_info, None)
                .expect("fragment shader module");

            //
            // ------------------------------------------------------------
            // Graphics Pipeline
            // ------------------------------------------------------------
            //
            // The graphics pipeline fixes the rules used to turn submitted
            // vertices into pixels: shader stages, primitive topology,
            // viewport, rasterization, multisampling, and color blending.
            //
            // This example uses no vertex buffer. The vertex shader obtains
            // the vertex number from SV_VertexID and constructs the triangle.
            //

            let main_name = CString::new("vertMain").unwrap();

            let vertex_stage = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_module)
                .name(&main_name);

            let main_name_frag = CString::new("fragMain").unwrap();

            let fragment_stage = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module)
                .name(&main_name_frag);

            let stages = [vertex_stage, fragment_stage];

            //
            // IMPORTANT:
            //
            // There are NO vertex attributes.
            //
            // SV_VertexID supplies the vertex number.
            //

            let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();

            let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false);

            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };

            let scissors = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            };

            let viewports = [viewport];
            let scissors_array = [scissors];

            let viewport_state = vk::PipelineViewportStateCreateInfo::default()
                .viewports(&viewports)
                .scissors(&scissors_array);

            let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0)
                .cull_mode(vk::CullModeFlags::NONE)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

            let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);

            let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .blend_enable(false);

            let color_blend_attachments = [color_blend_attachment];

            let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op_enable(false)
                .attachments(&color_blend_attachments);

            let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();

            let pipeline_layout = device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("pipeline layout");

            let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
                .stages(&stages)
                .vertex_input_state(&vertex_input)
                .input_assembly_state(&input_assembly)
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterizer)
                .multisample_state(&multisampling)
                .color_blend_state(&color_blending)
                .layout(pipeline_layout)
                .render_pass(render_pass)
                .subpass(0);

            let graphics_pipeline = device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .expect("graphics pipeline")[0];

            device.destroy_shader_module(vertex_module, None);

            device.destroy_shader_module(fragment_module, None);

            //
            // ------------------------------------------------------------
            // Framebuffers
            // ------------------------------------------------------------
            //
            // A framebuffer binds the render pass's attachment description
            // to actual image views. There is one framebuffer per swapchain
            // image, so the acquired image index selects the matching
            // framebuffer during command recording.
            //

            let framebuffers = swapchain_image_views
                .iter()
                .map(|&view| {
                    let attachments = [view];

                    let info = vk::FramebufferCreateInfo::default()
                        .render_pass(render_pass)
                        .attachments(&attachments)
                        .width(extent.width)
                        .height(extent.height)
                        .layers(1);

                    device.create_framebuffer(&info, None).expect("framebuffer")
                })
                .collect::<Vec<_>>();

            //
            // ------------------------------------------------------------
            // Command Pool and Command Buffer
            // ------------------------------------------------------------
            //
            // Vulkan rendering is normally submitted through recorded command
            // buffers. The command pool controls allocation/reset of command
            // buffers for a particular queue family.
            //

            // RESET_COMMAND_BUFFER lets draw() reset and re-record the
            // command buffer every frame for the acquired swapchain image.
            let command_pool_info = vk::CommandPoolCreateInfo::default()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index);

            let command_pool = device
                .create_command_pool(&command_pool_info, None)
                .expect("command pool");

            let command_buffer_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            let command_buffer = device
                .allocate_command_buffers(&command_buffer_info)
                .expect("command buffer")[0];

            //
            // ------------------------------------------------------------
            // GPU Synchronization
            // ------------------------------------------------------------
            //
            // Semaphores synchronize GPU operations: one says that the
            // acquired swapchain image is ready, and the other says that
            // rendering has completed before presentation.
            //
            // The fence lets the CPU know that the previous submission has
            // completed before the single reusable command buffer and its
            // synchronization objects are reused.
            //

            let semaphore_info = vk::SemaphoreCreateInfo::default();

            let image_available = device
                .create_semaphore(&semaphore_info, None)
                .expect("semaphore");

            let render_finished = device
                .create_semaphore(&semaphore_info, None)
                .expect("semaphore");

            let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

            let in_flight = device.create_fence(&fence_info, None).expect("fence");

            Self {
                entry,
                instance,
                surface_loader,
                surface,
                physical_device,
                device,
                queue,
                swapchain_loader,
                swapchain,
                swapchain_images,
                swapchain_image_views,
                swapchain_extent: extent,
                render_pass,
                pipeline_layout,
                graphics_pipeline,
                framebuffers,
                command_pool,
                command_buffer,
                image_available,
                render_finished,
                in_flight,
            }
        }
    }

    //
    // Recorded fresh every frame for the swapchain image that was just
    // acquired. The swapchain cycles through several images; recording
    // once against a single framebuffer would present unrendered images
    // and make the triangle blink.
    //

    /// Records commands for the swapchain image selected by `image_index`.
    ///
    /// A command buffer is not a drawing operation by itself. It is a list of
    /// commands that will later be executed by a Vulkan queue. Here the list
    /// is deliberately small:
    ///
    /// `begin -> begin render pass -> bind pipeline -> draw 3 vertices -> end`
    ///
    /// The framebuffer is selected from the acquired swapchain image index.
    unsafe fn record_command_buffer(&self, image_index: u32) {
        unsafe {
            self.device
                .begin_command_buffer(
                    self.command_buffer,
                    &vk::CommandBufferBeginInfo::default(),
                )
                .expect("begin command buffer");

            let clear_value = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.05, 0.05, 0.05, 1.0],
                },
            };

            let clear_values = [clear_value];

            let render_begin = vk::RenderPassBeginInfo::default()
                .render_pass(self.render_pass)
                .framebuffer(self.framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain_extent,
                })
                .clear_values(&clear_values);

            self.device.cmd_begin_render_pass(
                self.command_buffer,
                &render_begin,
                vk::SubpassContents::INLINE,
            );

            self.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline,
            );

            //
            // HERE!
            //
            // No vertex buffer.
            //
            // Draw 3 vertices.
            //

            self.device
                .cmd_draw(self.command_buffer, 3, 1, 0, 0);

            self.device.cmd_end_render_pass(self.command_buffer);

            self.device
                .end_command_buffer(self.command_buffer)
                .expect("end command buffer");
        }
    }

    /// Executes one complete frame.
    ///
    /// The CPU/GPU sequence is:
    ///
    /// 1. Wait until the previous use of our reusable command buffer is done.
    /// 2. Acquire a swapchain image.
    /// 3. Record commands targeting that image's framebuffer.
    /// 4. Submit those commands to the graphics queue.
    /// 5. Present the same swapchain image after rendering finishes.
    ///
    /// The semaphores establish GPU-to-GPU ordering; the fence establishes
    /// CPU-to-GPU reuse ordering.
    unsafe fn draw(&self) {
        unsafe {
            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .expect("wait fence");

            self.device
                .reset_fences(&[self.in_flight])
                .expect("reset fence");

            let (image_index, _) = self
                .swapchain_loader
                .acquire_next_image(
                    self.swapchain,
                    u64::MAX,
                    self.image_available,
                    vk::Fence::null(),
                )
                .expect("acquire image");

            self.record_command_buffer(image_index);

            let wait_semaphores = [self.image_available];

            let signal_semaphores = [self.render_finished];

            // The semaphore wait is consumed before the color-attachment
            // output stage. In other words, the GPU must not start writing
            // the acquired swapchain image until image acquisition signals
            // `image_available`.
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

            let command_buffers = [self.command_buffer];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            self.device
                .queue_submit(self.queue, &[submit_info], self.in_flight)
                .expect("queue submit");

            let swapchains = [self.swapchain];

            let image_indices = [image_index];

            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            self.swapchain_loader
                .queue_present(self.queue, &present_info)
                .expect("queue present");
        }
    }

    /// Releases Vulkan resources in dependency-safe reverse order.
    ///
    /// Vulkan does not automatically destroy handles merely because a Rust
    /// variable goes out of scope. Every created Vulkan object must be
    /// explicitly destroyed (or wrapped in an RAII abstraction that performs
    /// the same operation).
    ///
    /// Destruction must respect dependencies. For example, framebuffers use
    /// image views and a render pass, so they are destroyed before those
    /// objects. The device is destroyed only after device-owned resources are
    /// gone, and the instance is destroyed last.
    unsafe fn destroy(&self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            self.device.destroy_semaphore(self.image_available, None);

            self.device.destroy_semaphore(self.render_finished, None);

            self.device.destroy_fence(self.in_flight, None);

            self.device.destroy_command_pool(self.command_pool, None);

            for &framebuffer in &self.framebuffers {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            self.device.destroy_pipeline(self.graphics_pipeline, None);

            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);

            self.device.destroy_render_pass(self.render_pass, None);

            for &view in &self.swapchain_image_views {
                self.device.destroy_image_view(view, None);
            }

            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);

            self.device.destroy_device(None);

            self.surface_loader.destroy_surface(self.surface, None);

            self.instance.destroy_instance(None);
        }
    }
}

/// Small winit application state.
///
/// `window` must remain alive while the Vulkan surface is being used. The
/// `VulkanApp` is therefore kept alongside the window rather than creating
/// and immediately dropping the window after initialization.
struct App {
    window: Option<Window>,
    vulkan: Option<VulkanApp>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_title("Rust + Slang + Vulkan")
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT));

        let window = event_loop.create_window(attributes).expect("window");

        let vulkan = unsafe { VulkanApp::new(&window) };

        self.window = Some(window);
        self.vulkan = Some(vulkan);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                if let Some(vulkan) = &self.vulkan {
                    unsafe {
                        vulkan.destroy();
                    }
                }

                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if let Some(vulkan) = &self.vulkan {
                    unsafe {
                        vulkan.draw();
                    }
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// Program entry point.
///
/// winit owns the event loop. The application creates its window and Vulkan
/// state from the `resumed` callback, renders whenever a redraw is requested,
/// and explicitly destroys Vulkan resources when the window closes.
fn main() {
    let event_loop = EventLoop::new().expect("event loop");

    let mut app = App {
        window: None,
        vulkan: None,
    };

    event_loop.run_app(&mut app).expect("event loop error");
}

```