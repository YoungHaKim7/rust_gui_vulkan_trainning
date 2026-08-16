# Result


```bash


```


# fix

- Fixed and running clean — all asserts pass (build → assemble → parse → disassemble round-trip matches).
  - The code was written against an older rspirv layout (the old README example). In 0.13, three things changed:

1. **SPIR-V enums moved to the `rspirv::spirv` module** — `Capability`, `AddressingModel`, `MemoryModel`, `FunctionControl`, `ExecutionModel` are no longer re-exported at the crate root. Added a `use rspirv::spirv::{...}` import.
2. **`MAGIC_NUMBER` is `rspirv::spirv::MAGIC_NUMBER`**, not a root export.
3. **`begin_basic_block` was renamed to `begin_block`** (same `Option<Word>` label arg, still returns `BuildResult`).
