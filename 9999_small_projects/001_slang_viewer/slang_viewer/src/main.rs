use ash::{util::read_spv, vk};
use std::{ffi::CString, fs::File, io::BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entry = unsafe { ash::Entry::load()? };

    let app_name = CString::new("Slang Triangle")?;

    let app_info = vk::ApplicationInfo::default()
        .application_name(&app_name)
        .application_version(0)
        .engine_name(&app_name)
        .engine_version(0)
        .api_version(vk::API_VERSION_1_3);

    let create_info = vk::InstanceCreateInfo::default().application_info(&app_info);

    let instance = unsafe { entry.create_instance(&create_info, None)? };

    println!("Vulkan instance created!");

    let vertex_spv = read_spv(&mut BufReader::new(File::open("assets/vertex.spv")?))?;

    let fragment_spv = read_spv(&mut BufReader::new(File::open("assets/fragment.spv")?))?;

    println!("Loaded vertex SPIR-V: {} words", vertex_spv.len());
    println!("Loaded fragment SPIR-V: {} words", fragment_spv.len());

    unsafe {
        instance.destroy_instance(None);
    }

    Ok(())
}
