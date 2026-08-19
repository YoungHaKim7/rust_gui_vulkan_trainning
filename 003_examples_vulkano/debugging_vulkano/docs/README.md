Fixed and verified — it builds with no warnings and runs, with the validation layer actively printing debug messages through your callback.

The code was written for the old (≤0.31) vulkano API. In 0.35 the constructors take `Arc`s and `*CreateInfo` structs **by value** instead of by reference. Changes made:

**`Instance::new`** (`src/main.rs:52`)
- `&library` → `library`, `&InstanceCreateInfo {..}` → `InstanceCreateInfo {..}`
- `enabled_layers` is now `Vec<String>`: `["VK_LAYER_KHRONOS_validation"]` → `vec!["VK_LAYER_KHRONOS_validation".to_owned()]`
- `enabled_extensions: &extensions` → `extensions`

**`DebugUtilsMessenger`** (`src/main.rs:67`)
- `&instance` → `instance.clone()` (kept alive since you use `instance` again later), create-info by value
- `DebugUtilsMessengerCreateInfo::new(&callback)` no longer exists → use `..DebugUtilsMessengerCreateInfo::user_callback(DebugUtilsMessengerCallback::new(...))` as the struct-update base. Note `DebugUtilsMessengerCallback::new` already returns `Arc<Self>`, so no extra `Arc::new` — and it's still `unsafe`, so that block stays

**`Device::new`** (`src/main.rs:142`)
- `&physical_device` → `physical_device`, create-info by value
- `queue_create_infos: &[..]` → `vec![..]`, `enabled_extensions: &device_extensions` → `device_extensions`

**Bonus cleanup**: `VulkanLibrary::new()` is no longer `unsafe` in 0.35, so that block was removed too.
