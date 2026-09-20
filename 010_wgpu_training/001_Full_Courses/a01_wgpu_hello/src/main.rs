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
        let limits = adapter.limits();

        println!("Name:   \t{}", info.name);
        println!("Backend:\t{}", info.backend);
        println!("Type:   \t{:?}", info.device_type);
        println!("Driver: \t{} {}", info.driver, info.driver_info);

        println!(
            "Max texture dimension 2D:\t{}",
            limits.max_texture_dimension_2d
        );

        println!();
    }
}
