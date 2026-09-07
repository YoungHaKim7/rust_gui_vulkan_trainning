# grep이랑 test랑 섞어서 쓰기

```bash
cargo test --lib 2>&1 | grep -E "^test .* FAILED|^failures:" -A 20 | grep -E "panicked|FAILED|assert|left|right|Error|at src" | head -50)
  ⎿  test filter::filter::tests::push_path_queues_and_counts ... FAILED
     test filter::filter::tests::ff_request_frame_pull_and_eof ... FAILED
     test filter::filter::tests::inlink_set_status_discards_queued_frames ... FAILED
```


# test 2
- All five reviewers are reading C sources. Status while the final verify runs:

- This session's progress

Your test-move request — done: all 22 inline #[cfg(test)] modules extracted from src/ into tests/unit/<module-path>.rs (mirroring the src tree), wired back with #[cfg(test)] #[path] includes — so they still compile inside the crate (private-field access and use super::* unchanged), cargo doesn't double-compile them as intargets, and cargo test --lib runs exactly as before. Behavior-identical             before/after; README documents the layout (KR).

vf_scale landed (recovered from the rate-limit casualty's worktree):
- All 6 failing test expectations corrected against C — verified with the C sources and live system ffmpeg: h=oh/2 really is rejected at init (check_exprs), "Cannot parse expression…" has no trailing period while the self-referencing errors do, trailing whitespace in WxH is extraneous data, frame SAR comes from the decoded  frame (never the link), the unconditional in_chroma_loc stamp makes chroma      "mismatches" a noop pass-through in C too, and packed rgb24 gives repeating [R,G,B] patterns-scale_algo/-scale_engine now propagate into the -vf graph (like -sws_flags flows  in real ffmpeg)

Golden suite now 13 tests: -vf scale=256:192:flags=lanczos is max-1 vs system ffmpeg (the phase-3a bit-faithful table kernels hold through the filtergraph); bicubic rides the established ≤96 phase-1 bound; the spline,format=rgb24 chain is bounded ≤96 with a NOTE — ffmpeg's negotiation fuses scale+convert into one sws pass where ours two-steps (max 61) — the vf_scale reviewer is checking exactly that question now.

Totals: 199 lib + 3 integration + 13 golden, all green, all committed.

In flight: the final 5-area adversarial verify workflow (wf_08b566fc-82b: buffersrc, buffersink, driver, parser, vf_scale — vf_format already reviewed clean). When it reports, I fix whatever survives refutation, and Phase 3b is fully closed out.
