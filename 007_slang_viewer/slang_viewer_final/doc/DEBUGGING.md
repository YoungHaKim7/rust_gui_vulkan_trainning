```bash
gdb coredumpctl 2>/dev/null; gdb -batch -ex run -ex "bt 12" -ex "info registers rip" --args target/debug/slang_viewer_final 2>&1 | tail -30

gdb -batch -ex run -ex "info proc mappings" -ex "x/8i \$rip" -ex "info symbol \$rip" --args target/debug/slang_viewer_final 2>&1 | tail -50

env VK_LAYER_KHRONOS_VALIDATION=true timeout 12 target/debug/slang_viewer_final > /tmp/sv_log4.txt 2>&1; echo
      "exit: $?"; grep -ci "validation\|error\|warn" /tm…)
  ⎿  /bin/bash: line 1: 27033 Segmentation fault      env VK_LAYER_KHRONOS_VALIDATION=true timeout 12 target/debug/
     slang_viewer_final > /tmp/sv_log4.txt 2>&1

```

```bash
vulkaninfo --summary 2>/dev/null | grep -A 12 "GPU0")
  ⎿  GPU0:
        apiVersion         = 1.4.329
        driverVersion      = 595.84.0.0
```
