Where Phase 3b stands (all green: 193 lib + 3 integration + 8 golden)

Landed and verified:
- Wave-1 engine + negotiation driver (query_formats rounds with auto-scale insertion, reduce/pick, graph_config_links) — plus three real bugs fixed en route: check_validity side-attachment, config() swallowing fatal negotiation errors, and the 0/0-vs-0/1 "unset" test for link defaults
- buffersrc (1586 lines), buffersink (1238), format/noformat (798) — recovered from implementer transcripts after the worktree cleanup, then repaired + verified
- -vf CLI + transrc → parsed graph→ buffersink, sink-derived output params, EOF drain) — activates the m
- The adversarial review of vf_format came back clean (its one
  finding was refce)

Casualties of the2, now open):
- The buffersrc/buffersink/driver reviews died mid-run — I'll
  fold them into  after the newmodules land
- Both parser + vore writinganything — the fresh workflow now running has both cached specs staged anguard

In flight (workfl
- parser.rs (graphparser.c — av_get_token quoting rules, label parsing, the se
- vf_scale.rs (vf_scale.c + scale_eval.c — delegating to the existing ScaleCtor, honestUnsupported gating for unported color-matrix conversions)

Next when it completes: integrate + wire registry/parse_ptr, flip the missing-
