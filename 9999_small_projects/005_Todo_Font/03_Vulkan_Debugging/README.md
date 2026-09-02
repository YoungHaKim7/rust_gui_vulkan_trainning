```bash
● Update(src/renderer.rs)
  ⎿  Added 2 lines
      725
      726          match future.map_err(Validated::unwrap) {
      727              Ok(future) => {
      728 +                eprintln!("DBG frame ok image_index={image_index} buf={:p}",  // TEMP-DEBUG
      729 +                    std::sync::Arc::as_ptr(vertex_buffer.buffer()));
      730                  rcx.previous_frame_end = Some(future.boxed());
      731              }
      732              Err(VulkanError::OutOfDate) => {
```
