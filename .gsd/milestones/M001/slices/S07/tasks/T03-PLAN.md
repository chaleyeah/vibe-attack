---
estimated_steps: 33
estimated_files: 3
skills_used: []
---

# T03: Update docs to remove ORT conflict warnings and document shared .so deployment

## Description

Now that the dual-ORT conflict is resolved by the shared feature switch, update documentation to remove the stale "enable only one at a time" warnings and add deployment guidance for the shared `.so` files.

## Steps

1. In `docs/troubleshooting.md`, **Models** section (around line 89-91):
   - Remove the paragraph: "**ONNX Runtime errors:** Two concurrent ONNX Runtime instances can conflict. If you see a `bad_alloc` or ORT initialization error, ensure only one feature (STT *or* wake) is enabled at a time."
   - Replace with a note about `ORT_DYLIB_PATH` for custom installs: if `libonnxruntime.so` is not next to the binary, set `ORT_DYLIB_PATH=/path/to/libonnxruntime.so`.

2. In `docs/troubleshooting.md`, **Build** section (at the end):
   - Add a note that `cargo build` copies `libonnxruntime.so` and `libsherpa-onnx-c-api.so` to the target directory, and these must be present at runtime for wake-word and VAD to work.

3. In `docs/configuration.md`, **wake** section (around line 172-173):
   - Remove the blockquote: "> **Note:** Running both STT and wake-word simultaneously may cause ONNX Runtime conflicts. If you see initialization errors, enable only one at a time."
   - Replace with a note that `libonnxruntime.so` and `libsherpa-onnx-c-api.so` are automatically placed next to the binary at build time. For custom installs, set `ORT_DYLIB_PATH`.

4. Check `tests/documentation.rs` for any assertions that grep for the old conflict text (e.g. "only one", "conflict", "bad_alloc"). The current tests check for section headings (uinput, ptt, Installation, Usage) and project name — none reference the ORT conflict text, so no test changes should be needed. Verify this by reading the file.

## Must-Haves

- [ ] ORT conflict "ensure only one feature" warning removed from `docs/troubleshooting.md`
- [ ] ORT conflict "enable only one at a time" note removed from `docs/configuration.md`
- [ ] `ORT_DYLIB_PATH` guidance added to `docs/troubleshooting.md`
- [ ] Shared `.so` deployment note added to docs
- [ ] `tests/documentation.rs` still passes (no assertions reference removed text)

## Verification

- `! grep -qi 'ensure only one feature' docs/troubleshooting.md` exits 0
- `! grep -qi 'enable only one at a time' docs/configuration.md` exits 0
- `grep -qi 'ORT_DYLIB_PATH' docs/troubleshooting.md` exits 0
- `grep -qi 'libonnxruntime' docs/configuration.md` exits 0
- No documentation.rs test assertions reference the removed text (static check)

## Inputs

- `docs/troubleshooting.md` — ORT conflict warning to remove (Models section, lines 89-91)
- `docs/configuration.md` — ORT conflict note to remove (wake section, lines 172-173)
- `tests/documentation.rs` — verify no assertions reference removed text
- `Cargo.toml` — confirms shared feature is now used (from T01)
- `src/pipeline/coordinator.rs` — confirms ORT_DYLIB_PATH auto-discovery is in place (from T01)

## Expected Output

- `docs/troubleshooting.md` — ORT conflict warning replaced with ORT_DYLIB_PATH guidance and shared .so deployment note
- `docs/configuration.md` — ORT conflict note replaced with shared .so deployment note

## Inputs

- ``docs/troubleshooting.md` — ORT conflict warning to remove from Models section`
- ``docs/configuration.md` — ORT conflict note to remove from wake section`
- ``tests/documentation.rs` — verify no assertions reference removed text`
- ``Cargo.toml` — confirms shared feature change from T01`
- ``src/pipeline/coordinator.rs` — confirms ORT_DYLIB_PATH auto-discovery from T01`

## Expected Output

- ``docs/troubleshooting.md` — ORT conflict warning replaced with shared .so deployment guidance`
- ``docs/configuration.md` — ORT conflict note replaced with shared .so deployment note`

## Verification

! grep -qi 'ensure only one feature' docs/troubleshooting.md && ! grep -qi 'enable only one at a time' docs/configuration.md && grep -qi 'ORT_DYLIB_PATH' docs/troubleshooting.md && grep -qi 'libonnxruntime' docs/configuration.md
