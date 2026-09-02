# Linux에서 WindowsOS테스트 하기

```bash
# target install
rustup target add x86_64-pc-windows-gnu 2>&1 | tail -2
```

```bash
 cargo r --release
   Compiling todo_app_vulkan v0.1.0 (C:\Users\ytok1\OneDrive\Desktop\Finacial_working\todo_app_vulkan)
error[E0433]: cannot find `wayland_clipboard` in `copypasta`
   --> src\app.rs:237:24
    |
237 |             copypasta::wayland_clipboard::create_clipboards_from_external(display.display.as_ptr())
    |                        ^^^^^^^^^^^^^^^^^ could not find `wayland_clipboard` in `copypasta`
    |
note: found an item that was configured out
   --> C:\Users\ytok1\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\copypasta-0.10.2\src\lib.rs:29:9
    |
 20 |     unix,
    |     ---- the item is gated here
...
 29 | pub mod wayland_clipboard;
    |         ^^^^^^^^^^^^^^^^^

For more information about this error, try `rustc --explain E0433`.
error: could not compile `todo_app_vulkan` (bin "todo_app_vulkan") due to 1 previous error
PS C:\Users\ytok1\OneDrive\Desktop\Finacial_working\todo_app_vulkan>
error[E0433]: cannot find `wayland_clipboard` in `copypasta`
   --> src\app.rs:237:24
    |
237 |             copypasta::wayland_clipboard::create_clipboards_from_external(display.display.as_ptr())
    |                        ^^^^^^^^^^^^^^^^^ could not find `wayland_clipboard` in `copypasta`
    |
note: found an item that was configured out
   --> C:\Users\ytok1\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\copypasta-0.10.2\src\lib.rs:29:9
    |
 20 |     unix,
    |     ---- the item is gated here
...
 29 | pub mod wayland_clipboard;
    |         ^^^^^^^^^^^^^^^^^

For more information about this error, try `rustc --explain E0433`.
error: could not compile `todo_app_vulkan` (bin "todo_app_vulkan") due to 1 previous error
```
