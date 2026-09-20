use wgpu;

fn main() {
    // Enable wgpu logging.
    env_logger::init();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: wgpu::InstanceFlags::default(),
        backend_options: wgpu::BackendOptions::default(),

        // Use the default memory budget thresholds.
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),

        // No specific display handle.
        display: None,
    });

    println!("-- GPUs found on this machine --");

    let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));

    for adapter in adapters {
        let info = adapter.get_info();

        println!("Name:   \t{}", info.name);
        println!("Backend:\t{}", info.backend);
        println!("Type:   \t{:?}", info.device_type);
        println!("Driver: \t{} {}", info.driver, info.driver_info);

        let limits = adapter.limits();
        println!(
            "Max texture dimension 2D:\t{}",
            limits.max_texture_dimension_2d
        );
        println!("Max texture size : \t\t{}", limits.max_texture_dimension_2d);
        println!("Max buffer size : \t\t{}", limits.max_buffer_size);
        println!("Max bind groups : \t\t{}", limits.max_bind_groups);
        println!("Max vertex attribs : \t\t{}", limits.max_vertex_attributes);
        println!(
            "Max compute group : \t\t{}",
            limits.max_compute_workgroup_size_x
        );

        println!();
    }
}
