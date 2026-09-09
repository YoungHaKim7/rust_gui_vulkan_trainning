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
