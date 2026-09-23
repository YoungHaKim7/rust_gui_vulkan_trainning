use ash::Entry;

fn main() {
    let entry = unsafe { Entry::load().unwrap() };

    let extensions = unsafe { entry.enumerate_instance_extension_properties(None).unwrap() };

    for extension in extensions {
        let name = unsafe { std::ffi::CStr::from_ptr(extension.extension_name.as_ptr()) };

        println!("{}", name.to_string_lossy());
    }
}
