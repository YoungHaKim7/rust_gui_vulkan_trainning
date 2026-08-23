//! Compiles the GLSL shaders in `src/shaders/` to SPIR-V at build time using `glslc`
//! (part of the Vulkan SDK). The resulting `.spv` files are embedded into the binary
//! with `include_bytes!`, so no shader files need to be shipped alongside the program.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn find_glslc() -> Option<PathBuf> {
    // 1. Inside the Vulkan SDK, if VULKAN_SDK is set.
    if let Ok(sdk) = env::var("VULKAN_SDK") {
        let candidate = Path::new(&sdk).join("bin").join("glslc");
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    // 2. Somewhere on PATH.
    if let Some(path_var) = env::var_os("PATH") {
        for dir in env::split_paths(&path_var) {
            let candidate = dir.join("glslc");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let glslc = find_glslc().unwrap_or_else(|| {
        panic!(
            "glslc not found: install the Vulkan SDK or make sure \
             VULKAN_SDK points to it"
        )
    });

    let out_dir = env::var("OUT_DIR").unwrap();

    let shaders = [
        ("src/shaders/circle.vert", "circle.vert.spv"),
        ("src/shaders/circle.frag", "circle.frag.spv"),
    ];

    for (source, compiled) in shaders {
        println!("cargo:rerun-if-changed={source}");

        let output_path = Path::new(&out_dir).join(compiled);

        // Target Vulkan 1.0 / SPIR-V 1.0 for maximum driver compatibility.
        let status = Command::new(&glslc)
            .arg("--target-env=vulkan1.0")
            .arg("-O")
            .arg(source)
            .arg("-o")
            .arg(&output_path)
            .status()
            .unwrap_or_else(|err| panic!("failed to run {}: {err}", glslc.display()));

        assert!(status.success(), "shader compilation failed: {source}");

        assert!(fs::metadata(&output_path).is_ok());
    }
}
