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

    let adapters = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
        power_preference: wgpu::PowerPreference::HighPerformance,
        // power_preference: wgpu::PowerPreference::LowPower,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: true,
    }))
    .expect("No suitable GPU found on this machine.");

    println!("Choose:\t\t{:?}", adapters.get_info().name);

    println!();
}
