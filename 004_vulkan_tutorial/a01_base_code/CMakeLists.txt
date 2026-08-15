cmake_minimum_required(VERSION 3.16)

get_filename_component(ProjectId ${CMAKE_CURRENT_SOURCE_DIR} NAME)
string(REPLACE " " "_" ProjectId ${ProjectId})
project(${ProjectId} LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# find Vulkan before creating targets, so Vulkan::Vulkan carries include paths
find_package(Vulkan REQUIRED)

# GLFW: prefer the CMake config shipped by GLFW, fall back to pkg-config
find_package(glfw3 QUIET)
if(NOT glfw3_FOUND)
    find_package(PkgConfig REQUIRED)
    pkg_check_modules(GLFW REQUIRED IMPORTED_TARGET glfw3)
endif()

# GLM is header-only; newer versions ship a CMake config, fall back to find_path
find_package(glm QUIET)
if(NOT glm_FOUND)
    find_path(GLM_INCLUDE_DIR glm/glm.hpp REQUIRED)
endif()

# Source files
add_executable(${ProjectId} src/main.cpp)

# Link libraries
target_link_libraries(${ProjectId} Vulkan::Vulkan dl pthread)

if(glfw3_FOUND)
    target_link_libraries(${ProjectId} glfw)
else()
    target_link_libraries(${ProjectId} PkgConfig::GLFW)
endif()

if(${CMAKE_SYSTEM_NAME} STREQUAL "Darwin")
    target_link_libraries(${ProjectId}
        "-framework Cocoa"
        "-framework IOKit"
        "-framework CoreVideo"
    )
endif()

# GLM is header-only
if(NOT glm_FOUND)
    target_include_directories(${ProjectId} PRIVATE ${GLM_INCLUDE_DIR})
endif()

if(Vulkan_FOUND)
    if(NOT Vulkan_dxc_exe_FOUND)
        find_program(Vulkan_dxc_EXECUTABLE
                    NAMES dxc
                    HINTS
                    "$ENV{VULKAN_SDK}/Bin"
                    "$ENV{VULKAN_SDK}/bin"
                    )
    endif()
    if(Vulkan_dxc_EXECUTABLE)
        message(STATUS "Found DirectX Shader Compiler under ${Vulkan_dxc_EXECUTABLE}")
    else()
        message(STATUS "Couldn't find DirectX Shader Compiler executable, make sure it is present in Vulkan SDK or add it manually via Vulkan_dxc_EXECUTABLE cmake variable. HLSL shaders won't be compiled.")
    endif()
    # slang compiler
    if(NOT Vulkan_slang_exe_FOUND)
        find_program(Vulkan_slang_EXECUTABLE
                    NAMES slangc
                    HINTS
                    "$ENV{VULKAN_SDK}/Bin"
                    "$ENV{VULKAN_SDK}/bin"
                    )
    endif()
    if(Vulkan_slang_EXECUTABLE)
        message(STATUS "Found slang Shader Compiler under ${Vulkan_slang_EXECUTABLE}")
    else()
        message(STATUS "Couldn't find slang Shader Compiler executable, make sure it is present in Vulkan SDK or add it manually via Vulkan_slang_EXECUTABLE cmake variable. Slang shaders won't be compiled.")
    endif()
    # glsl compiler
    if(NOT Vulkan_glslc_exe_FOUND)
        find_program(Vulkan_glslc_EXECUTABLE
                    NAMES glslc
                    HINTS
                    "$ENV{VULKAN_SDK}/Bin"
                    "$ENV{VULKAN_SDK}/bin"
                    )
    endif()
    if(Vulkan_glslc_EXECUTABLE)
        message(STATUS "Found glslc Shader Compiler under ${Vulkan_glslc_EXECUTABLE}")
    else()
        message(STATUS "Couldn't find glslc Shader Compiler executable, make sure it is present in Vulkan SDK or add it manually via Vulkan_glslc_EXECUTABLE cmake variable. GLSL shaders won't be compiled.")
    endif()
    # spvasm compiler
    if(NOT Vulkan_spirvas_exe_FOUND)
        find_program(Vulkan_spirvas_EXECUTABLE
                    NAMES spirv-as
                    HINTS
                    "$ENV{VULKAN_SDK}/Bin"
                    "$ENV{VULKAN_SDK}/bin"
                    )
    endif()
    if(Vulkan_spirvas_EXECUTABLE)
        message(STATUS "Found spirv-as Shader Compiler under ${Vulkan_spirvas_EXECUTABLE}")
    else()
        message(STATUS "Couldn't find spirv-as Shader Compiler executable, make sure it is present in Vulkan SDK or add it manually via Vulkan_spirvas_EXECUTABLE cmake variable. SPIR-V assembly shaders won't be compiled.")
    endif()

endif()

# globally add VKB_DEBUG for the debug build
add_compile_definitions($<$<CONFIG:DEBUG>:VKB_DEBUG>)

#globally define VULKAN_HPP_NO_STRUCT_CONSTRUCTORS to have designated initializers for the vk-structs
add_compile_definitions(VULKAN_HPP_NO_STRUCT_CONSTRUCTORS)

# globally set -fno-strict-aliasing, needed due to using reinterpret_cast
if (NOT MSVC)
  add_compile_options(-fno-strict-aliasing)
endif()

if(MSVC AND (DEFINED CMAKE_C_COMPILER_LAUNCHER))
    message(DEBUG "Setting MSVC flags to /Z7 for ccache compatibility.  Current flags: ${CMAKE_CXX_FLAGS_DEBUG}")
    string(REPLACE "/Zi" "/Z7" CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG}")
    string(REPLACE "/Zi" "/Z7" CMAKE_C_FLAGS_DEBUG "${CMAKE_C_FLAGS_DEBUG}")
    string(REPLACE "/Zi" "/Z7" CMAKE_CXX_FLAGS_RELWITHDEBINFO "${CMAKE_CXX_FLAGS_RELWITHDEBINFO}")
    string(REPLACE "/Zi" "/Z7" CMAKE_C_FLAGS_RELWITHDEBINFO "${CMAKE_C_FLAGS_RELWITHDEBINFO}")
    message(DEBUG "New flags: ${CMAKE_CXX_FLAGS_DEBUG}")
endif()
 
# Set output directory
set_target_properties(${ProjectId} PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY 
        "${CMAKE_BINARY_DIR}/$<LOWER_CASE:$<CONFIG>>"
)

# Optimization flags for Release build (via add_compile_options, keeps CMake's defaults)
if(NOT MSVC)
    add_compile_options($<$<CONFIG:Release>:-O2>)
endif()
