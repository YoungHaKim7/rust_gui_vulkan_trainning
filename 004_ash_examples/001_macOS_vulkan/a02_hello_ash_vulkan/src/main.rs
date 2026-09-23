use ash::{Entry, vk};
use std::ffi::{CStr, CString};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entry = unsafe { Entry::load()? };

    // 1. Check what layers/extensions actually exist
    let available_layers = unsafe { entry.enumerate_instance_layer_properties()? };
    let has_validation = available_layers.iter().any(|l| {
        let name = unsafe { CStr::from_ptr(l.layer_name.as_ptr()) };
        name == c"VK_LAYER_KHRONOS_validation"
    });

    let app_name = CString::new("Ash Practice")?;
    let engine_name = CString::new("No Engine")?;

    // FIX 1: Use 1.0 - universally supported. 1.3 causes INCOMPATIBLE_DRIVER on many systems
    let app_info = vk::ApplicationInfo::default()
        .application_name(&app_name)
        .application_version(0)
        .engine_name(&engine_name)
        .engine_version(0)
        .api_version(vk::API_VERSION_1_0);

    let layer_names = [c"VK_LAYER_KHRONOS_validation".as_ptr()];
    let mut extension_names = vec![];

    // Only enable debug_utils if we actually have it
    if has_validation {
        extension_names.push(ash::ext::debug_utils::NAME.as_ptr());
    }

    let mut create_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names);

    if has_validation {
        create_info = create_info.enabled_layer_names(&layer_names);
        println!("Enabling validation layers");
    } else {
        println!("Validation layers not found, continuing without them");
    }

    let instance = unsafe { entry.create_instance(&create_info, None)? };
    println!("Instance created!");

    let pdevices = unsafe { instance.enumerate_physical_devices()? };
    println!("Found {} physical devices", pdevices.len());

    let (pdevice, queue_family_index) = pdevices
        .iter()
        .find_map(|pdevice| {
            let props = unsafe { instance.get_physical_device_queue_family_properties(*pdevice) };
            props.iter().enumerate().find_map(|(index, info)| {
                if info.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    Some((*pdevice, index as u32))
                } else {
                    None
                }
            })
        })
        .expect("No suitable GPU found");

    let props = unsafe { instance.get_physical_device_properties(pdevice) };
    println!(
        "Selected GPU: {} (API {}.{}.{} / Driver {})",
        unsafe { CStr::from_ptr(props.device_name.as_ptr()).to_string_lossy() },
        vk::api_version_major(props.api_version),
        vk::api_version_minor(props.api_version),
        vk::api_version_patch(props.api_version),
        props.driver_version
    );

    let queue_priority = [1.0f32];
    let queue_info = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_family_index)
        .queue_priorities(&queue_priority);

    let device_create_info =
        vk::DeviceCreateInfo::default().queue_create_infos(std::slice::from_ref(&queue_info));

    let device = unsafe { instance.create_device(pdevice, &device_create_info, None)? };
    let _graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };
    println!("Logical device and graphics queue created!");

    unsafe {
        device.destroy_device(None);
        instance.destroy_instance(None);
    }

    Ok(())
}
