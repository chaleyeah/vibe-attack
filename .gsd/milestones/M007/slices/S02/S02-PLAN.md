# S02: Internal consistency — safety comments, alias notes, lint annotations

**Goal:** Make the codebase internally consistent: every unsafe block, type alias, and lint allow has an explanation; duplicate or stale comments are removed; the dual get_socket_path functions in control are documented as intentional.
**Demo:** cargo test passes; cargo clippy -D warnings clean; grep for `unsafe impl` in src/pipeline/dispatcher.rs shows a // SAFETY: comment immediately above each impl; the SegCfg alias in coordinator.rs has an explanatory comment; the dual get_socket_path functions in control/mod.rs and control/client.rs have a comment in each describing the intentional split; the duplicate doc comment on default_config_path in config.rs is collapsed; the #[allow(clippy::too_many_arguments)] on jsonl.rs has a justification comment

## Must-Haves

- Every `unsafe impl` and `unsafe fn` in src/ has a `// SAFETY:` comment explaining the invariant; every `#[allow(...)]` annotation in src/ has a one-line justification comment; duplicate doc comment on default_config_path collapsed; SegCfg alias in coordinator.rs has an explanatory comment; both get_socket_path functions in control/ describe the intentional split; cargo test and cargo clippy --all-targets -- -D warnings remain clean on both default and gui features.

## Proof Level

- This slice proves: static — verified by grep audits and cargo clippy passing -D warnings. No runtime behavior changes.

## Integration Closure

No integration boundaries touched.

## Verification

- None.

## Tasks

- [x] **T01: Add SAFETY comments to unsafe impl Send/Sync on Dispatcher** `est:15m`
  In src/pipeline/dispatcher.rs, add a `// SAFETY:` comment immediately above each `unsafe impl Send for Dispatcher {}` and `unsafe impl Sync for Dispatcher {}`. The comment must explain that rodio's OutputStream (held by SoundPlayer) is not Send, but Dispatcher only ever invokes SoundPlayer from its single owning thread, making the manual Send/Sync impls sound. Run cargo clippy -D warnings to verify.
  - Files: `src/pipeline/dispatcher.rs`
  - Verify: grep -B1 'unsafe impl' src/pipeline/dispatcher.rs shows a // SAFETY: line immediately preceding each unsafe impl; cargo clippy --all-targets -- -D warnings clean

- [x] **T02: Annotate the SegCfg alias and #[allow] annotations** `est:30m`
  In src/pipeline/coordinator.rs, add a comment explaining why VadConfig is aliased as SegCfg (likely historical naming or to avoid conflict with another type — read git blame or the surrounding code to confirm; if the alias has no real reason, remove it instead). In src/pipeline/jsonl.rs, add a one-line comment above `#[allow(clippy::too_many_arguments)]` justifying why the function legitimately needs that many arguments. Audit `grep -rn '#\[allow(' src/` for any other unjustified allows and add justification comments to each.
  - Files: `src/pipeline/coordinator.rs`, `src/pipeline/jsonl.rs`
  - Verify: grep -B1 '#\[allow(' src/ shows a justifying comment above each allow; cargo clippy -D warnings clean

- [x] **T03: Document the dual get_socket_path functions in control/** `est:15m`
  In src/control/mod.rs and src/control/client.rs, add a comment on each private get_socket_path function explaining the intentional difference: mod.rs uses place_runtime_file (creating the parent dir), client.rs uses find_runtime_file (read-only lookup). The comment should reference its counterpart so a reader searching for one finds the other.
  - Files: `src/control/mod.rs`, `src/control/client.rs`
  - Verify: grep -B3 'fn get_socket_path' src/control/ shows an explanatory comment in each location; cargo check passes

- [ ] **T04: Collapse duplicate doc comment on default_config_path** `est:5m`
  In src/config.rs around lines 258–260, two consecutive /// lines both say 'Return the XDG config file path'. Collapse into a single accurate doc comment. Verify against current line numbers (file may have shifted).
  - Files: `src/config.rs`
  - Verify: grep -A1 'fn default_config_path' src/config.rs shows a single coherent /// doc block; cargo check passes

- [ ] **T05: Run full verification — test, clippy, audit grep** `est:10m`
  Run cargo test, cargo test --features gui, cargo clippy --all-targets -- -D warnings, cargo clippy --all-targets --features gui -- -D warnings, and `grep -rn '#\[allow(\|unsafe impl\|unsafe fn' src/` to confirm every match has an adjacent justifying comment. All cargo invocations must exit 0.
  - Verify: All four cargo invocations exit 0; manual review of grep output confirms each unsafe/allow has a comment

## Files Likely Touched

- src/pipeline/dispatcher.rs
- src/pipeline/coordinator.rs
- src/pipeline/jsonl.rs
- src/control/mod.rs
- src/control/client.rs
- src/config.rs
