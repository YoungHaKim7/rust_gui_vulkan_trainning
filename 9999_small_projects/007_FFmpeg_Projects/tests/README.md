# grep이랑 test랑 섞어서 쓰기

```bash
cargo test --lib 2>&1 | grep -E "^test .* FAILED|^failures:" -A 20 | grep -E "panicked|FAILED|assert|left|right|Error|at src" | head -50)
  ⎿  test filter::filter::tests::push_path_queues_and_counts ... FAILED
     test filter::filter::tests::ff_request_frame_pull_and_eof ... FAILED
     test filter::filter::tests::inlink_set_status_discards_queued_frames ... FAILED
```
