# Vulkan개념잡기(잘 정리됨)
- https://namu.wiki/w/Vulkan(API)

# vulkano examples
- https://github.com/vulkano-rs/vulkano/tree/master/examples

# vulkan tutorial rust code
- https://github.com/bwasty/vulkan-tutorial-rs
  - https://vulkan-tutorial.com/code/
  - https://github.com/Overv/VulkanTutorial

- https://vulkan-tutorial.com/
  - https://github.com/Overv/VulkanTutorial
- 첫시작은 Vlukano구만
  - https://vulkano.rs/
  - https://github.com/vulkano-rs/vulkano/tree/master/examples


# Vulkano-Shaders
- https://docs.rs/vulkano-shaders/latest/vulkano_shaders/

# (Unity 지만 잘 정리됨)버텍스 및 프래그먼트 셰이더 예제(Vertex and fragment shader examples)
https://docs.unity3d.com/kr/2017.1/Manual/SL-VertexFragmentShaderExamples.html

# frag, vert 예시
- https://gist.github.com/vicrucann/3214c150012c6816c487a451a12590fe
- https://docs.unity3d.com/450/Documentation/Manual/SL-VertexFragmentShaderExamples.html


# rust Vulkan
- https://github.com/gfx-rs/wgpu-native

```bash
Fixed. The problem was that the git submodules weren't initialized — `build.rs` needs `ffi/webgpu-headers/webgpu.h` (bindgen input), but the directory was empty.

Ran `git submodule update --init --recursive`, which cloned both `ffi/webgpu-headers` and `examples/vendor/glfw`, and `cargo build` now succeeds.
```

- wgpu-native is a Rust C-ABI library built with Cargo (MSRV 1.87), with a Makefile wrapping the common flows:

## Build the library

```sh
cargo build                # debug → target/debug/libwgpu_native.so
cargo build --release      # release → target/release/libwgpu_native.so
```

or via the Makefile (`make lib-native` / `make lib-native-release`). For a specific target triple: `make lib-native TARGET=x86_64-unknown-linux-gnu`.

## Build & run the C examples

Examples need CMake + Ninja and link against the built library:

```sh
make run-example-triangle            # debug
make run-example-triangle-release    # release
```

Other examples: `compute`, `capture`, `enumerate_adapters`, `texture_arrays`, `immediates`, `metal_interop` (macOS only).

## Other targets

- `make check` — `cargo check --all`
- `make test` — `cargo test --all`
- `make package` — builds both profiles and zips `libwgpu_native.so`/`.a` + headers (`ffi/webgpu.h`, `ffi/webgpu-headers/webgpu.h`) into `dist/`
- `make clean`

Headers for consumers live in `ffi/webgpu-headers/webgpu.h` and `ffi/wgpu.h`. Note the `rust-toolchain.toml` pins a specific Rust version, so rustup will pick it up automatically.


# 외국사람이 만든거
- Exprimental web-based idler game in rust
  - https://github.com/gregouar/webidler
