#![warn(
    clippy::use_self,
    deprecated_in_future,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts
)]

use std::{
    borrow::Cow,
    ffi,
    io::Cursor,
    mem::{self, align_of, size_of, size_of_val},
    os::raw::{c_char, c_void},
};

use ash::util::*;

use ash::{
    Device, Entry, Instance,
    ext::debug_utils,
    khr::{surface, swapchain},
    vk,
};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

pub const MAX_FRAME_LATENCY: usize = 3;

// -----------------------------------------------------------------------------
// Utility
// -----------------------------------------------------------------------------

#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = mem::zeroed();
            std::ptr::addr_of!(b.$field) as isize - std::ptr::addr_of!(b) as isize
        }
    }};
}

#[allow(clippy::too_many_arguments)]
pub fn record_submit_commandbuffer<F: FnOnce(&Device, vk::CommandBuffer)>(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    command_buffer_reuse_fence: vk::Fence,
    submit_queue: vk::Queue,
    wait_mask: &[vk::PipelineStageFlags],
    wait_semaphores: &[vk::Semaphore],
    signal_semaphores: &[vk::Semaphore],
    f: F,
) {
    unsafe {
        device
            .reset_command_buffer(
                command_buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .expect("Reset command buffer failed.");

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("Begin command buffer failed.");

        f(device, command_buffer);

        device
            .end_command_buffer(command_buffer)
            .expect("End command buffer failed.");

        let command_buffers = [command_buffer];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);

        device
            .queue_submit(submit_queue, &[submit_info], command_buffer_reuse_fence)
            .expect("Queue submit failed.");
    }
}

// -----------------------------------------------------------------------------
// Vulkan debug callback
// -----------------------------------------------------------------------------

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let callback_data = unsafe { *p_callback_data };

    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        unsafe { ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy() }
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        unsafe { ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy() }
    };

    println!(
        "{message_severity:?}:\n\
         {message_type:?} \
         [{message_id_name} ({message_id_number})] : \
         {message}\n"
    );

    vk::FALSE
}

// -----------------------------------------------------------------------------
// Memory helper
// -----------------------------------------------------------------------------

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as usize]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1u32 << *index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _)| index as u32)
}

// -----------------------------------------------------------------------------
// Vulkan base
// -----------------------------------------------------------------------------

pub struct ExampleBase {
    pub entry: Entry,
    pub instance: Instance,
    pub device: Device,

    pub surface_loader: surface::Instance,
    pub swapchain_loader: swapchain::Device,
    pub debug_utils_loader: debug_utils::Instance,

    pub window: Window,

    pub frame_index: usize,

    pub debug_call_back: vk::DebugUtilsMessengerEXT,

    pub pdevice: vk::PhysicalDevice,
    pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_index: u32,
    pub present_queue: vk::Queue,

    pub surface: vk::SurfaceKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_resolution: vk::Extent2D,

    pub swapchain: vk::SwapchainKHR,
    pub present_images: Vec<vk::Image>,
    pub present_image_views: Vec<vk::ImageView>,

    pub pool: vk::CommandPool,

    pub draw_command_buffers: [vk::CommandBuffer; MAX_FRAME_LATENCY],

    pub setup_command_buffer: vk::CommandBuffer,
    pub app_setup_command_buffer: vk::CommandBuffer,

    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_image_memory: vk::DeviceMemory,

    pub present_complete_semaphores: [vk::Semaphore; MAX_FRAME_LATENCY],

    pub rendering_complete_semaphores: Vec<vk::Semaphore>,

    pub draw_commands_reuse_fences: [vk::Fence; MAX_FRAME_LATENCY],
}

impl ExampleBase {
    // -------------------------------------------------------------------------
    // Create Vulkan
    // -------------------------------------------------------------------------

    pub fn new(
        window: Window,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            // -------------------------------------------------------------
            // Vulkan loader
            //
            // Ash 0.38:
            //
            //     Entry::linked()
            //
            // became:
            //
            //     Entry::load()
            // -------------------------------------------------------------

            let entry = Entry::load()?;

            let app_name = c"VulkanTexture";

            // -------------------------------------------------------------
            // Validation layer
            // -------------------------------------------------------------

            let available_layers = entry.enumerate_instance_layer_properties()?;

            let validation_layer_name = c"VK_LAYER_KHRONOS_validation";

            let validation_available = available_layers.iter().any(|layer| {
                let name = std::ffi::CStr::from_ptr(layer.layer_name.as_ptr());

                name == validation_layer_name
            });

            let layer_names_raw: Vec<*const c_char> = if validation_available {
                vec![validation_layer_name.as_ptr()]
            } else {
                Vec::new()
            };

            if validation_available {
                println!("Validation layer: enabled");
            } else {
                println!(
                    "Validation layer: VK_LAYER_KHRONOS_validation \
                     not found; continuing without it."
                );
            }

            // -------------------------------------------------------------
            // Required window-system extensions
            //
            // ash-window 0.13:
            //
            //     enumerate_required_extensions(display_handle)
            // -------------------------------------------------------------

            let display_handle = window.display_handle()?.as_raw();

            let window_handle = window.window_handle()?.as_raw();

            let mut extension_names =
                ash_window::enumerate_required_extensions(display_handle)?.to_vec();

            // Debug utils is required for our debug messenger.
            extension_names.push(debug_utils::NAME.as_ptr());

            // -------------------------------------------------------------
            // macOS / iOS portability
            // -------------------------------------------------------------

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            {
                extension_names.push(ash::khr::portability_enumeration::NAME.as_ptr());

                extension_names.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
            }

            let appinfo = vk::ApplicationInfo::default()
                .application_name(app_name)
                .application_version(0)
                .engine_name(app_name)
                .engine_version(0)
                .api_version(vk::make_api_version(0, 1, 0, 0));

            let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            } else {
                vk::InstanceCreateFlags::empty()
            };

            let create_info = vk::InstanceCreateInfo::default()
                .application_info(&appinfo)
                .enabled_layer_names(&layer_names_raw)
                .enabled_extension_names(&extension_names)
                .flags(create_flags);

            let instance = entry.create_instance(&create_info, None)?;

            // -------------------------------------------------------------
            // Debug messenger
            // -------------------------------------------------------------

            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vulkan_debug_callback));

            // Ash 0.38:
            //
            // debug_utils::Instance::load(...)
            //
            // became:
            //
            // debug_utils::Instance::new(...)

            let debug_utils_loader = debug_utils::Instance::new(&entry, &instance);

            let debug_call_back =
                debug_utils_loader.create_debug_utils_messenger(&debug_info, None)?;

            // -------------------------------------------------------------
            // Surface
            //
            // ash-window 0.13 no longer has SurfaceFactory.
            // -------------------------------------------------------------

            let surface =
                ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)?;

            // Ash 0.38:
            //
            // surface::Instance::load(...)
            //
            // became:
            //
            // surface::Instance::new(...)

            let surface_loader = surface::Instance::new(&entry, &instance);

            // -------------------------------------------------------------
            // Physical device
            // -------------------------------------------------------------

            let pdevices = instance.enumerate_physical_devices()?;

            let (pdevice, queue_family_index) = pdevices
                .iter()
                .find_map(|pdevice| {
                    instance
                        .get_physical_device_queue_family_properties(*pdevice)
                        .iter()
                        .enumerate()
                        .find_map(|(index, info)| {
                            let supports_graphic =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS);

                            let supports_surface = surface_loader
                                .get_physical_device_surface_support(
                                    *pdevice,
                                    index as u32,
                                    surface,
                                )
                                .ok()?;

                            if supports_graphic && supports_surface {
                                Some((*pdevice, index as u32))
                            } else {
                                None
                            }
                        })
                })
                .ok_or("Couldn't find suitable Vulkan device")?;

            // -------------------------------------------------------------
            // Device extensions
            // -------------------------------------------------------------

            let device_extension_names_raw = [
                swapchain::NAME.as_ptr(),
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                ash::khr::portability_subset::NAME.as_ptr(),
            ];

            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: vk::TRUE,
                ..Default::default()
            };

            let priorities = [1.0f32];

            let queue_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);

            let device_create_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features);

            let device = instance.create_device(pdevice, &device_create_info, None)?;

            let present_queue = device.get_device_queue(queue_family_index, 0);

            // -------------------------------------------------------------
            // Surface format
            // -------------------------------------------------------------

            let surface_formats =
                surface_loader.get_physical_device_surface_formats(pdevice, surface)?;

            let surface_format = surface_formats
                .first()
                .copied()
                .ok_or("No surface formats available")?;

            // -------------------------------------------------------------
            // Surface capabilities
            // -------------------------------------------------------------

            let surface_capabilities =
                surface_loader.get_physical_device_surface_capabilities(pdevice, surface)?;

            let mut desired_image_count = surface_capabilities.min_image_count + 1;

            if surface_capabilities.max_image_count > 0
                && desired_image_count > surface_capabilities.max_image_count
            {
                desired_image_count = surface_capabilities.max_image_count;
            }

            let surface_resolution = match surface_capabilities.current_extent.width {
                u32::MAX => vk::Extent2D {
                    width: window_width,
                    height: window_height,
                },
                _ => surface_capabilities.current_extent,
            };

            let pre_transform = if surface_capabilities
                .supported_transforms
                .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
            {
                vk::SurfaceTransformFlagsKHR::IDENTITY
            } else {
                surface_capabilities.current_transform
            };

            // -------------------------------------------------------------
            // Present mode
            // -------------------------------------------------------------

            let present_modes =
                surface_loader.get_physical_device_surface_present_modes(pdevice, surface)?;

            let present_mode = present_modes
                .iter()
                .copied()
                .find(|mode| *mode == vk::PresentModeKHR::MAILBOX)
                .unwrap_or(vk::PresentModeKHR::FIFO);

            // Ash 0.38:
            //
            // swapchain::Device::load(...)
            //
            // became:
            //
            // swapchain::Device::new(...)

            let swapchain_loader = swapchain::Device::new(&instance, &device);

            // -------------------------------------------------------------
            // Swapchain
            // -------------------------------------------------------------

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(surface)
                .min_image_count(desired_image_count)
                .image_color_space(surface_format.color_space)
                .image_format(surface_format.format)
                .image_extent(surface_resolution)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);

            let swapchain = swapchain_loader.create_swapchain(&swapchain_create_info, None)?;

            // -------------------------------------------------------------
            // Command pool
            // -------------------------------------------------------------

            let pool_create_info = vk::CommandPoolCreateInfo::default()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index);

            let pool = device.create_command_pool(&pool_create_info, None)?;

            // We need:
            //
            // 0 = depth setup
            // 1 = texture setup
            // 2.. = per-frame drawing
            //

            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_buffer_count(2 + MAX_FRAME_LATENCY as u32)
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffers = device.allocate_command_buffers(&command_buffer_allocate_info)?;

            let setup_command_buffer = command_buffers[0];

            let app_setup_command_buffer = command_buffers[1];

            let draw_command_buffers = command_buffers[2..][..MAX_FRAME_LATENCY]
                .try_into()
                .unwrap();

            // -------------------------------------------------------------
            // Swapchain images
            // -------------------------------------------------------------

            let present_images = swapchain_loader.get_swapchain_images(swapchain)?;

            let present_image_views: Vec<vk::ImageView> = present_images
                .iter()
                .map(|&image| {
                    let create_view_info = vk::ImageViewCreateInfo::default()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface_format.format)
                        .components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        })
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .image(image);

                    device.create_image_view(&create_view_info, None).unwrap()
                })
                .collect();

            // -------------------------------------------------------------
            // Device memory properties
            // -------------------------------------------------------------

            let device_memory_properties = instance.get_physical_device_memory_properties(pdevice);

            // -------------------------------------------------------------
            // Depth image
            // -------------------------------------------------------------

            let depth_image_create_info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::D16_UNORM)
                .extent(surface_resolution.into())
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let depth_image = device.create_image(&depth_image_create_info, None)?;

            let depth_image_memory_req = device.get_image_memory_requirements(depth_image);

            let depth_image_memory_index = find_memorytype_index(
                &depth_image_memory_req,
                &device_memory_properties,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .ok_or("Unable to find suitable memory for depth image")?;

            let depth_image_allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(depth_image_memory_req.size)
                .memory_type_index(depth_image_memory_index);

            let depth_image_memory = device.allocate_memory(&depth_image_allocate_info, None)?;

            device.bind_image_memory(depth_image, depth_image_memory, 0)?;

            // -------------------------------------------------------------
            // Transition depth image
            // -------------------------------------------------------------

            record_submit_commandbuffer(
                &device,
                setup_command_buffer,
                vk::Fence::null(),
                present_queue,
                &[],
                &[],
                &[],
                |device, command_buffer| {
                    let barrier = vk::ImageMemoryBarrier::default()
                        .image(depth_image)
                        .dst_access_mask(
                            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        )
                        .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .subresource_range(
                            vk::ImageSubresourceRange::default()
                                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                                .layer_count(1)
                                .level_count(1),
                        );

                    device.cmd_pipeline_barrier(
                        command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[barrier],
                    );
                },
            );

            // -------------------------------------------------------------
            // Depth view
            // -------------------------------------------------------------

            let depth_image_view_info = vk::ImageViewCreateInfo::default()
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::DEPTH)
                        .level_count(1)
                        .layer_count(1),
                )
                .image(depth_image)
                .format(depth_image_create_info.format)
                .view_type(vk::ImageViewType::TYPE_2D);

            let depth_image_view = device.create_image_view(&depth_image_view_info, None)?;

            // -------------------------------------------------------------
            // Synchronization
            // -------------------------------------------------------------

            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let present_complete_semaphores = std::array::from_fn(|_| {
                device
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap()
            });

            let rendering_complete_semaphores = (0..present_images.len())
                .map(|_| {
                    device
                        .create_semaphore(&semaphore_create_info, None)
                        .unwrap()
                })
                .collect();

            let fence_create_info =
                vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

            let draw_commands_reuse_fences = std::array::from_fn(|_| {
                device
                    .create_fence(&fence_create_info, None)
                    .expect("Create fence failed.")
            });

            Ok(Self {
                entry,
                instance,
                device,

                surface_loader,
                swapchain_loader,
                debug_utils_loader,

                window,

                frame_index: 0,

                debug_call_back,

                pdevice,
                device_memory_properties,
                queue_family_index,
                present_queue,

                surface,
                surface_format,
                surface_resolution,

                swapchain,
                present_images,
                present_image_views,

                pool,
                draw_command_buffers,

                setup_command_buffer,
                app_setup_command_buffer,

                depth_image,
                depth_image_view,
                depth_image_memory,

                present_complete_semaphores,
                rendering_complete_semaphores,

                draw_commands_reuse_fences,
            })
        }
    }

    // -------------------------------------------------------------------------
    // Start next frame
    // -------------------------------------------------------------------------

    pub fn begin_frame(&mut self) -> usize {
        let frame_index = self.frame_index;

        let fence = self.draw_commands_reuse_fences[frame_index % MAX_FRAME_LATENCY];

        unsafe {
            self.device
                .wait_for_fences(&[fence], true, u64::MAX)
                .expect("Wait for fence failed.");

            self.device
                .reset_fences(&[fence])
                .expect("Reset fence failed.");
        }

        self.frame_index += 1;

        frame_index
    }
}

impl Drop for ExampleBase {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();

            for &semaphore in &self.present_complete_semaphores {
                self.device.destroy_semaphore(semaphore, None);
            }

            for &semaphore in &self.rendering_complete_semaphores {
                self.device.destroy_semaphore(semaphore, None);
            }

            for &fence in &self.draw_commands_reuse_fences {
                self.device.destroy_fence(fence, None);
            }

            self.device.free_memory(self.depth_image_memory, None);

            self.device.destroy_image_view(self.depth_image_view, None);

            self.device.destroy_image(self.depth_image, None);

            for &image_view in &self.present_image_views {
                self.device.destroy_image_view(image_view, None);
            }

            self.device.destroy_command_pool(self.pool, None);

            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);

            self.device.destroy_device(None);

            self.surface_loader.destroy_surface(self.surface, None);

            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_call_back, None);

            self.instance.destroy_instance(None);
        }
    }
}

// -----------------------------------------------------------------------------
// Vertex
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct Vertex {
    pos: [f32; 4],
    uv: [f32; 2],
}

#[derive(Clone, Copy, Debug)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub _pad: f32,
}

// -----------------------------------------------------------------------------
// Texture example
// -----------------------------------------------------------------------------

pub struct TextureExample {
    pub base: ExampleBase,

    renderpass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,

    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,

    vertex_input_buffer: vk::Buffer,
    vertex_input_buffer_memory: vk::DeviceMemory,

    uniform_color_buffer: vk::Buffer,
    uniform_color_buffer_memory: vk::DeviceMemory,

    image_buffer: vk::Buffer,
    image_buffer_memory: vk::DeviceMemory,

    texture_image: vk::Image,
    texture_memory: vk::DeviceMemory,
    tex_image_view: vk::ImageView,
    sampler: vk::Sampler,

    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,

    vertex_shader_module: vk::ShaderModule,
    fragment_shader_module: vk::ShaderModule,

    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,
}

impl TextureExample {
    pub fn new(
        window: Window,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let base = ExampleBase::new(window, width, height)?;

            // -------------------------------------------------------------
            // Render pass
            // -------------------------------------------------------------

            let renderpass_attachments = [
                vk::AttachmentDescription::default()
                    .format(base.surface_format.format)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
                vk::AttachmentDescription::default()
                    .format(vk::Format::D16_UNORM)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
            ];

            let color_attachment_refs = [vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            }];

            let depth_attachment_ref = vk::AttachmentReference {
                attachment: 1,
                layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            };

            let dependencies = [vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,

                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,

                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,

                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                    | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,

                ..Default::default()
            }];

            let subpass = vk::SubpassDescription::default()
                .color_attachments(&color_attachment_refs)
                .depth_stencil_attachment(&depth_attachment_ref)
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

            let renderpass_create_info = vk::RenderPassCreateInfo::default()
                .attachments(&renderpass_attachments)
                .subpasses(std::slice::from_ref(&subpass))
                .dependencies(&dependencies);

            let renderpass = base
                .device
                .create_render_pass(&renderpass_create_info, None)?;

            // -------------------------------------------------------------
            // Framebuffers
            // -------------------------------------------------------------

            let framebuffers: Vec<vk::Framebuffer> = base
                .present_image_views
                .iter()
                .map(|&present_image_view| {
                    let attachments = [present_image_view, base.depth_image_view];

                    let info = vk::FramebufferCreateInfo::default()
                        .render_pass(renderpass)
                        .attachments(&attachments)
                        .width(base.surface_resolution.width)
                        .height(base.surface_resolution.height)
                        .layers(1);

                    base.device.create_framebuffer(&info, None).unwrap()
                })
                .collect();

            // -------------------------------------------------------------
            // Index buffer
            // -------------------------------------------------------------

            let index_buffer_data = [0u32, 1, 2, 2, 3, 0];

            let index_buffer_info = vk::BufferCreateInfo::default()
                .size(size_of_val(&index_buffer_data) as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let index_buffer = base.device.create_buffer(&index_buffer_info, None)?;

            let index_buffer_memory_req = base.device.get_buffer_memory_requirements(index_buffer);

            let index_buffer_memory_index = find_memorytype_index(
                &index_buffer_memory_req,
                &base.device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .ok_or("Unable to find memory for index buffer")?;

            let index_allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(index_buffer_memory_req.size)
                .memory_type_index(index_buffer_memory_index);

            let index_buffer_memory = base.device.allocate_memory(&index_allocate_info, None)?;

            let index_ptr = base.device.map_memory(
                index_buffer_memory,
                0,
                index_buffer_memory_req.size,
                vk::MemoryMapFlags::empty(),
            )?;

            let mut index_slice = Align::new(
                index_ptr,
                align_of::<u32>() as u64,
                index_buffer_memory_req.size,
            );

            index_slice.copy_from_slice(&index_buffer_data);

            base.device.unmap_memory(index_buffer_memory);

            base.device
                .bind_buffer_memory(index_buffer, index_buffer_memory, 0)?;

            // -------------------------------------------------------------
            // Vertex buffer
            // -------------------------------------------------------------

            let vertices = [
                Vertex {
                    pos: [-1.0, -1.0, 0.0, 1.0],
                    uv: [0.0, 0.0],
                },
                Vertex {
                    pos: [-1.0, 1.0, 0.0, 1.0],
                    uv: [0.0, 1.0],
                },
                Vertex {
                    pos: [1.0, 1.0, 0.0, 1.0],
                    uv: [1.0, 1.0],
                },
                Vertex {
                    pos: [1.0, -1.0, 0.0, 1.0],
                    uv: [1.0, 0.0],
                },
            ];

            let vertex_buffer_info = vk::BufferCreateInfo::default()
                .size(size_of_val(&vertices) as u64)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let vertex_input_buffer = base.device.create_buffer(&vertex_buffer_info, None)?;

            let vertex_memory_req = base
                .device
                .get_buffer_memory_requirements(vertex_input_buffer);

            let vertex_memory_index = find_memorytype_index(
                &vertex_memory_req,
                &base.device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .ok_or("Unable to find vertex buffer memory")?;

            let vertex_allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(vertex_memory_req.size)
                .memory_type_index(vertex_memory_index);

            let vertex_input_buffer_memory =
                base.device.allocate_memory(&vertex_allocate_info, None)?;

            let vert_ptr = base.device.map_memory(
                vertex_input_buffer_memory,
                0,
                vertex_memory_req.size,
                vk::MemoryMapFlags::empty(),
            )?;

            let mut slice = Align::new(
                vert_ptr,
                align_of::<Vertex>() as u64,
                vertex_memory_req.size,
            );

            slice.copy_from_slice(&vertices);

            base.device.unmap_memory(vertex_input_buffer_memory);

            base.device
                .bind_buffer_memory(vertex_input_buffer, vertex_input_buffer_memory, 0)?;

            // -------------------------------------------------------------
            // Uniform buffer
            // -------------------------------------------------------------

            let uniform_color_buffer_data = Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
                _pad: 0.0,
            };

            let uniform_buffer_info = vk::BufferCreateInfo::default()
                .size(size_of_val(&uniform_color_buffer_data) as u64)
                .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let uniform_color_buffer = base.device.create_buffer(&uniform_buffer_info, None)?;

            let uniform_memory_req = base
                .device
                .get_buffer_memory_requirements(uniform_color_buffer);

            let uniform_memory_index = find_memorytype_index(
                &uniform_memory_req,
                &base.device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .ok_or("Unable to find uniform buffer memory")?;

            let uniform_allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(uniform_memory_req.size)
                .memory_type_index(uniform_memory_index);

            let uniform_color_buffer_memory =
                base.device.allocate_memory(&uniform_allocate_info, None)?;

            let uniform_ptr = base.device.map_memory(
                uniform_color_buffer_memory,
                0,
                uniform_memory_req.size,
                vk::MemoryMapFlags::empty(),
            )?;

            let mut uniform_slice = Align::new(
                uniform_ptr,
                align_of::<Vector3>() as u64,
                uniform_memory_req.size,
            );

            uniform_slice.copy_from_slice(&[uniform_color_buffer_data]);

            base.device.unmap_memory(uniform_color_buffer_memory);

            base.device
                .bind_buffer_memory(uniform_color_buffer, uniform_color_buffer_memory, 0)?;

            // -------------------------------------------------------------
            // Load texture
            // -------------------------------------------------------------

            let image = image::load_from_memory(include_bytes!("../assets/rust.png"))?.to_rgba8();

            let (width, height) = image.dimensions();

            let image_extent = vk::Extent2D { width, height };

            let image_data = image.into_raw();

            // -------------------------------------------------------------
            // Staging buffer
            // -------------------------------------------------------------

            let image_buffer_info = vk::BufferCreateInfo::default()
                .size(image_data.len() as u64)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let image_buffer = base.device.create_buffer(&image_buffer_info, None)?;

            let image_memory_req = base.device.get_buffer_memory_requirements(image_buffer);

            let image_memory_index = find_memorytype_index(
                &image_memory_req,
                &base.device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .ok_or("Unable to find staging buffer memory")?;

            let image_allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(image_memory_req.size)
                .memory_type_index(image_memory_index);

            let image_buffer_memory = base.device.allocate_memory(&image_allocate_info, None)?;

            let image_ptr = base.device.map_memory(
                image_buffer_memory,
                0,
                image_memory_req.size,
                vk::MemoryMapFlags::empty(),
            )?;

            let mut image_slice =
                Align::new(image_ptr, align_of::<u8>() as u64, image_memory_req.size);

            image_slice.copy_from_slice(&image_data);

            base.device.unmap_memory(image_buffer_memory);

            base.device
                .bind_buffer_memory(image_buffer, image_buffer_memory, 0)?;

            // -------------------------------------------------------------
            // Texture image
            // -------------------------------------------------------------

            let texture_create_info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .extent(image_extent.into())
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let texture_image = base.device.create_image(&texture_create_info, None)?;

            let texture_memory_req = base.device.get_image_memory_requirements(texture_image);

            let texture_memory_index = find_memorytype_index(
                &texture_memory_req,
                &base.device_memory_properties,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .ok_or("Unable to find texture memory")?;

            let texture_allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(texture_memory_req.size)
                .memory_type_index(texture_memory_index);

            let texture_memory = base.device.allocate_memory(&texture_allocate_info, None)?;

            base.device
                .bind_image_memory(texture_image, texture_memory, 0)?;

            // -------------------------------------------------------------
            // Upload texture
            // -------------------------------------------------------------

            record_submit_commandbuffer(
                &base.device,
                base.app_setup_command_buffer,
                vk::Fence::null(),
                base.present_queue,
                &[],
                &[],
                &[],
                |device, command_buffer| {
                    let texture_barrier = vk::ImageMemoryBarrier::default()
                        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .image(texture_image)
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            level_count: 1,
                            layer_count: 1,
                            ..Default::default()
                        });

                    device.cmd_pipeline_barrier(
                        command_buffer,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[texture_barrier],
                    );

                    let buffer_copy_region = vk::BufferImageCopy::default()
                        .image_subresource(
                            vk::ImageSubresourceLayers::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .layer_count(1),
                        )
                        .image_extent(image_extent.into());

                    device.cmd_copy_buffer_to_image(
                        command_buffer,
                        image_buffer,
                        texture_image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[buffer_copy_region],
                    );

                    let texture_barrier_end = vk::ImageMemoryBarrier::default()
                        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .dst_access_mask(vk::AccessFlags::SHADER_READ)
                        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image(texture_image)
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            level_count: 1,
                            layer_count: 1,
                            ..Default::default()
                        });

                    device.cmd_pipeline_barrier(
                        command_buffer,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[texture_barrier_end],
                    );
                },
            );

            // -------------------------------------------------------------
            // Sampler
            // -------------------------------------------------------------

            let sampler_info = vk::SamplerCreateInfo::default()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                .address_mode_u(vk::SamplerAddressMode::MIRRORED_REPEAT)
                .address_mode_v(vk::SamplerAddressMode::MIRRORED_REPEAT)
                .address_mode_w(vk::SamplerAddressMode::MIRRORED_REPEAT)
                .max_anisotropy(1.0)
                .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
                .compare_op(vk::CompareOp::NEVER);

            let sampler = base.device.create_sampler(&sampler_info, None)?;

            // -------------------------------------------------------------
            // Texture image view
            // -------------------------------------------------------------

            let tex_image_view_info = vk::ImageViewCreateInfo::default()
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(texture_create_info.format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    level_count: 1,
                    layer_count: 1,
                    ..Default::default()
                })
                .image(texture_image);

            let tex_image_view = base.device.create_image_view(&tex_image_view_info, None)?;

            // -------------------------------------------------------------
            // Descriptor pool
            // -------------------------------------------------------------

            let descriptor_sizes = [
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                },
            ];

            let descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(&descriptor_sizes)
                .max_sets(1);

            let descriptor_pool = base
                .device
                .create_descriptor_pool(&descriptor_pool_info, None)?;

            // -------------------------------------------------------------
            // Descriptor set layout
            // -------------------------------------------------------------

            let desc_layout_bindings = [
                vk::DescriptorSetLayoutBinding::default()
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            ];

            let descriptor_info =
                vk::DescriptorSetLayoutCreateInfo::default().bindings(&desc_layout_bindings);

            let descriptor_set_layout = base
                .device
                .create_descriptor_set_layout(&descriptor_info, None)?;

            // -------------------------------------------------------------
            // Descriptor set
            // -------------------------------------------------------------

            let desc_alloc_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(descriptor_pool)
                .set_layouts(std::slice::from_ref(&descriptor_set_layout));

            let descriptor_set = base.device.allocate_descriptor_sets(&desc_alloc_info)?[0];

            let uniform_descriptor = vk::DescriptorBufferInfo {
                buffer: uniform_color_buffer,
                offset: 0,
                range: size_of_val(&uniform_color_buffer_data) as u64,
            };

            let texture_descriptor = vk::DescriptorImageInfo {
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                image_view: tex_image_view,
                sampler,
            };

            let write_desc_sets = [
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_set)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(std::slice::from_ref(&uniform_descriptor)),
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_set)
                    .dst_binding(1)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&texture_descriptor)),
            ];

            base.device.update_descriptor_sets(&write_desc_sets, &[]);

            // -------------------------------------------------------------
            // Shaders
            // -------------------------------------------------------------

            let mut vertex_spv_file =
                Cursor::new(&include_bytes!("../assets/texture/vert.spv")[..]);

            let mut frag_spv_file = Cursor::new(&include_bytes!("../assets/texture/frag.spv")[..]);

            let vertex_code = read_spv(&mut vertex_spv_file)?;

            let fragment_code = read_spv(&mut frag_spv_file)?;

            let vertex_shader_info = vk::ShaderModuleCreateInfo::default().code(&vertex_code);

            let fragment_shader_info = vk::ShaderModuleCreateInfo::default().code(&fragment_code);

            let vertex_shader_module = base
                .device
                .create_shader_module(&vertex_shader_info, None)?;

            let fragment_shader_module = base
                .device
                .create_shader_module(&fragment_shader_info, None)?;

            // -------------------------------------------------------------
            // Pipeline layout
            // -------------------------------------------------------------

            let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
                .set_layouts(std::slice::from_ref(&descriptor_set_layout));

            let pipeline_layout = base
                .device
                .create_pipeline_layout(&pipeline_layout_info, None)?;

            // -------------------------------------------------------------
            // Shader stages
            // -------------------------------------------------------------

            let shader_entry_name = c"main";

            let shader_stage_create_infos = [
                vk::PipelineShaderStageCreateInfo::default()
                    .module(vertex_shader_module)
                    .name(shader_entry_name)
                    .stage(vk::ShaderStageFlags::VERTEX),
                vk::PipelineShaderStageCreateInfo::default()
                    .module(fragment_shader_module)
                    .name(shader_entry_name)
                    .stage(vk::ShaderStageFlags::FRAGMENT),
            ];

            // -------------------------------------------------------------
            // Vertex input
            // -------------------------------------------------------------

            let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
                binding: 0,
                stride: size_of::<Vertex>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            }];

            let vertex_input_attribute_descriptions = [
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT,
                    offset: offset_of!(Vertex, pos) as u32,
                },
                vk::VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: vk::Format::R32G32_SFLOAT,
                    offset: offset_of!(Vertex, uv) as u32,
                },
            ];

            let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
                .vertex_binding_descriptions(&vertex_input_binding_descriptions);

            // -------------------------------------------------------------
            // Input assembly
            // -------------------------------------------------------------

            let vertex_input_assembly_state_info =
                vk::PipelineInputAssemblyStateCreateInfo::default()
                    .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

            // -------------------------------------------------------------
            // Viewport
            // -------------------------------------------------------------

            let viewports = [vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: base.surface_resolution.width as f32,
                height: base.surface_resolution.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }];

            let scissors = [base.surface_resolution.into()];

            let viewport_state_info = vk::PipelineViewportStateCreateInfo::default()
                .scissors(&scissors)
                .viewports(&viewports);

            // -------------------------------------------------------------
            // Rasterizer
            // -------------------------------------------------------------

            let rasterization_info = vk::PipelineRasterizationStateCreateInfo::default()
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .line_width(1.0)
                .polygon_mode(vk::PolygonMode::FILL);

            // -------------------------------------------------------------
            // Multisampling
            // -------------------------------------------------------------

            let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);

            // -------------------------------------------------------------
            // Depth
            // -------------------------------------------------------------

            let noop_stencil_state = vk::StencilOpState::default()
                .fail_op(vk::StencilOp::KEEP)
                .pass_op(vk::StencilOp::KEEP)
                .depth_fail_op(vk::StencilOp::KEEP)
                .compare_op(vk::CompareOp::ALWAYS);

            let depth_state_info = vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
                .front(noop_stencil_state)
                .back(noop_stencil_state)
                .max_depth_bounds(1.0);

            // -------------------------------------------------------------
            // Color blending
            // -------------------------------------------------------------

            let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState::default()
                .blend_enable(false)
                .src_color_blend_factor(vk::BlendFactor::SRC_COLOR)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_DST_COLOR)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ZERO)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD)
                .color_write_mask(vk::ColorComponentFlags::RGBA)];

            let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op(vk::LogicOp::CLEAR)
                .attachments(&color_blend_attachment_states);

            // -------------------------------------------------------------
            // Dynamic state
            // -------------------------------------------------------------

            let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

            let dynamic_state_info =
                vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_state);

            // -------------------------------------------------------------
            // Graphics pipeline
            // -------------------------------------------------------------

            let graphics_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
                .stages(&shader_stage_create_infos)
                .vertex_input_state(&vertex_input_state_info)
                .input_assembly_state(&vertex_input_assembly_state_info)
                .viewport_state(&viewport_state_info)
                .rasterization_state(&rasterization_info)
                .multisample_state(&multisample_state_info)
                .depth_stencil_state(&depth_state_info)
                .color_blend_state(&color_blend_state)
                .dynamic_state(&dynamic_state_info)
                .layout(pipeline_layout)
                .render_pass(renderpass);

            let graphics_pipelines = base
                .device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphics_pipeline_info],
                    None,
                )
                .map_err(|(_, error)| error)?;

            let graphics_pipeline = graphics_pipelines[0];

            Ok(Self {
                base,

                renderpass,
                framebuffers,

                index_buffer,
                index_buffer_memory,

                vertex_input_buffer,
                vertex_input_buffer_memory,

                uniform_color_buffer,
                uniform_color_buffer_memory,

                image_buffer,
                image_buffer_memory,

                texture_image,
                texture_memory,
                tex_image_view,
                sampler,

                descriptor_pool,
                descriptor_set_layout,
                descriptor_set,

                vertex_shader_module,
                fragment_shader_module,

                pipeline_layout,
                graphics_pipeline,
            })
        }
    }

    // -------------------------------------------------------------------------
    // Render
    // -------------------------------------------------------------------------

    pub fn render(&mut self) {
        unsafe {
            let frame_index = self.base.begin_frame();

            let present_complete_semaphore =
                self.base.present_complete_semaphores[frame_index % MAX_FRAME_LATENCY];

            let draw_commands_reuse_fence =
                self.base.draw_commands_reuse_fences[frame_index % MAX_FRAME_LATENCY];

            let draw_command_buffer =
                self.base.draw_command_buffers[frame_index % MAX_FRAME_LATENCY];

            let acquired = self.base.swapchain_loader.acquire_next_image(
                self.base.swapchain,
                u64::MAX,
                present_complete_semaphore,
                vk::Fence::null(),
            );

            let (present_index, _) = match acquired {
                Ok(value) => value,

                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    return;
                }

                Err(error) => {
                    panic!("acquire_next_image failed: {error:?}");
                }
            };

            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 0.0],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ];

            let rendering_complete_semaphore =
                self.base.rendering_complete_semaphores[present_index as usize];

            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.renderpass)
                .framebuffer(self.framebuffers[present_index as usize])
                .render_area(self.base.surface_resolution.into())
                .clear_values(&clear_values);

            record_submit_commandbuffer(
                &self.base.device,
                draw_command_buffer,
                draw_commands_reuse_fence,
                self.base.present_queue,
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[present_complete_semaphore],
                &[rendering_complete_semaphore],
                |device, command_buffer| {
                    device.cmd_begin_render_pass(
                        command_buffer,
                        &render_pass_begin_info,
                        vk::SubpassContents::INLINE,
                    );

                    device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_layout,
                        0,
                        &[self.descriptor_set],
                        &[],
                    );

                    device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.graphics_pipeline,
                    );

                    let viewport = vk::Viewport {
                        x: 0.0,
                        y: 0.0,
                        width: self.base.surface_resolution.width as f32,
                        height: self.base.surface_resolution.height as f32,
                        min_depth: 0.0,
                        max_depth: 1.0,
                    };

                    let scissor = vk::Rect2D::default()
                        .offset(vk::Offset2D { x: 0, y: 0 })
                        .extent(self.base.surface_resolution);

                    device.cmd_set_viewport(command_buffer, 0, &[viewport]);

                    device.cmd_set_scissor(command_buffer, 0, &[scissor]);

                    device.cmd_bind_vertex_buffers(
                        command_buffer,
                        0,
                        &[self.vertex_input_buffer],
                        &[0],
                    );

                    device.cmd_bind_index_buffer(
                        command_buffer,
                        self.index_buffer,
                        0,
                        vk::IndexType::UINT32,
                    );

                    device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 1);

                    device.cmd_end_render_pass(command_buffer);
                },
            );

            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(std::slice::from_ref(&rendering_complete_semaphore))
                .swapchains(std::slice::from_ref(&self.base.swapchain))
                .image_indices(std::slice::from_ref(&present_index));

            match self
                .base
                .swapchain_loader
                .queue_present(self.base.present_queue, &present_info)
            {
                Ok(_) => {}

                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {}

                Err(error) => {
                    panic!("queue_present failed: {error:?}");
                }
            }
        }
    }
}

impl Drop for TextureExample {
    fn drop(&mut self) {
        unsafe {
            let device = &self.base.device;

            let _ = device.device_wait_idle();

            device.destroy_pipeline(self.graphics_pipeline, None);

            device.destroy_pipeline_layout(self.pipeline_layout, None);

            device.destroy_shader_module(self.vertex_shader_module, None);

            device.destroy_shader_module(self.fragment_shader_module, None);

            device.destroy_descriptor_pool(self.descriptor_pool, None);

            device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);

            device.destroy_sampler(self.sampler, None);

            device.destroy_image_view(self.tex_image_view, None);

            device.destroy_image(self.texture_image, None);

            device.free_memory(self.texture_memory, None);

            device.destroy_buffer(self.image_buffer, None);

            device.free_memory(self.image_buffer_memory, None);

            device.destroy_buffer(self.index_buffer, None);

            device.free_memory(self.index_buffer_memory, None);

            device.destroy_buffer(self.vertex_input_buffer, None);

            device.free_memory(self.vertex_input_buffer_memory, None);

            device.destroy_buffer(self.uniform_color_buffer, None);

            device.free_memory(self.uniform_color_buffer_memory, None);

            for framebuffer in self.framebuffers.drain(..) {
                device.destroy_framebuffer(framebuffer, None);
            }

            device.destroy_render_pass(self.renderpass, None);
        }
    }
}

// -----------------------------------------------------------------------------
// Winit 0.30 application
// -----------------------------------------------------------------------------

struct App {
    example: Option<TextureExample>,
}

impl App {
    fn new() -> Self {
        Self { example: None }
    }
}

impl ApplicationHandler for App {
    // -------------------------------------------------------------------------
    // Window creation
    //
    // Winit 0.30 creates windows from `resumed()`.
    // -------------------------------------------------------------------------

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);

        if self.example.is_some() {
            return;
        }

        let attributes: WindowAttributes = Window::default_attributes()
            .with_title("Ash - Texture Example")
            .with_inner_size(winit::dpi::LogicalSize::new(1920.0, 1080.0));

        let window = event_loop
            .create_window(attributes)
            .expect("Failed to create window");

        let example = TextureExample::new(window, 1920, 1080)
            .expect("Failed to initialize Vulkan texture example");

        example.base.window.request_redraw();

        self.example = Some(example);
    }

    // -------------------------------------------------------------------------
    // Window events
    // -------------------------------------------------------------------------

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: ElementState::Pressed,
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if let Some(example) = self.example.as_mut() {
                    example.render();

                    example.base.window.request_redraw();
                }
            }

            _ => {}
        }
    }

    // -------------------------------------------------------------------------
    // Event loop idle point
    // -------------------------------------------------------------------------

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(example) = self.example.as_ref() {
            example.base.window.request_redraw();
        }
    }
}

// -----------------------------------------------------------------------------
// main
// -----------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;

    let mut app = App::new();

    event_loop.run_app(&mut app)?;

    Ok(())
}
