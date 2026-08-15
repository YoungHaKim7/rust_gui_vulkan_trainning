# Native WebGPU implementation based on wgpu-core
- https://github.com/gfx-rs/wgpu-native


# `submodule`로 파일을 다 받아줘야함.

```bash
$ git clone https://github.com/gfx-rs/wgpu-native.git

$ git submodule update --init --recursive

$ make run-example-triangle-release
```

# openSUSE(260815)
- error(CMakefile) 수정해줌

# fix

- Fixed. The `triangle` target defines `GLFW_EXPOSE_NATIVE_WAYLAND`/`GLFW_EXPOSE_NATIVE_X11` and includes `GLFW/glfw3native.h`, which requires `wayland-client.h` and Xlib headers — but only GLFW's own build had discovered those include paths.


The fix in `examples/triangle/CMakeLists.txt:19-26` uses pkg-config to find `x11` and `wayland-client`, adds their include dirs (on this system wayland headers live in `/usr/include/wayland`), and links their libs:

```cmake
find_package(PkgConfig REQUIRED)
pkg_check_modules(GLFW_NATIVE REQUIRED x11 wayland-client)
include_directories(${GLFW_NATIVE_INCLUDE_DIRS})
set(OS_LIBRARIES "-lm -ldl" ${GLFW_NATIVE_LIBRARIES})
```

Build now succeeds and the example runs (window opens, picks up the RTX 3060 Ti adapter). The remaining `-Wdiscarded-qualifiers` warnings (casting `const WGPUChainedStruct *` from compound literals) are pre-existing and harmless — happy to clean those up too if you want.

- `examples/triangle/CMakeLists.txt`
```cmake
cmake_minimum_required(VERSION 3.20)
project(triangle LANGUAGES C)

add_executable(triangle main.c)

if (MSVC)
    add_compile_options(/W4)
else()
    add_compile_options(-Wall -Wextra -Wpedantic)
endif()

include_directories(${CMAKE_SOURCE_DIR}/../ffi)
include_directories(${CMAKE_SOURCE_DIR}/../ffi/webgpu-headers)
include_directories(${CMAKE_SOURCE_DIR}/framework)

if (WIN32)
    add_definitions(-DGLFW_EXPOSE_NATIVE_WIN32)
    set(OS_LIBRARIES d3dcompiler ws2_32 userenv bcrypt ntdll opengl32 Propsys RuntimeObject)
elseif(UNIX AND NOT APPLE)
    add_definitions(-DGLFW_EXPOSE_NATIVE_X11)
    add_definitions(-DGLFW_EXPOSE_NATIVE_WAYLAND)
    find_package(PkgConfig REQUIRED)
    pkg_check_modules(GLFW_NATIVE REQUIRED x11 wayland-client)
    include_directories(${GLFW_NATIVE_INCLUDE_DIRS})
    set(OS_LIBRARIES "-lm -ldl" ${GLFW_NATIVE_LIBRARIES})
elseif(APPLE)
    add_definitions(-DGLFW_EXPOSE_NATIVE_COCOA)
    set(OS_LIBRARIES "-framework CoreFoundation -framework QuartzCore -framework Metal")
endif()

target_link_libraries(triangle framework glfw ${WGPU_LIBRARY} ${OS_LIBRARIES})
```
