# Using `glslc` (Recommended CLI): Run the Google Shader compiler included in the Vulkan SDK via your terminal or a batch script:

```bash
glslc shader.comp -o compute.spv
```

# Using `glslangValidator`: Alternatively, compile via the Khronos validator tool:

```bash
glslangValidator -V shader.comp -o compute.spv
```

- [240707) Vulkan with C++, Stage 11: Runtime Shader Compilation](https://youtu.be/z1QrxFTrO8E?si=n9WGsZEvsB49oMU-)
  - https://github.com/amengede/getIntoGameDev

# `vert` & `frag` compile to spv

- https://vulkan.lunarg.com/doc/view/1.4.304.1/mac/antora/tutorial/latest/03_Drawing_a_triangle/02_Graphics_pipeline_basics/01_Shader_modules.html

```
$ glslc shader.vert -o vert.spv
$ glslc shader.frag -o frag.spv
```
