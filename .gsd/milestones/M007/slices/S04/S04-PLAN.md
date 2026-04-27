# S04: Config and error type cleanup

**Goal:** Final pass on the two load-bearing types — Config and DaemonError — that every other module touches. Ensure every pub item in config.rs and error.rs has a doc comment that a new engineer can read and understand without diving into call sites.
**Demo:** cargo test passes; cargo clippy -D warnings clean; src/config.rs and src/error.rs have full doc coverage on every public item; the duplicate default_config_path doc is gone (already done in S02 — verify); DaemonError variant docs explain what each variant represents and where it originates

## Must-Haves

- Every pub item in src/config.rs and src/error.rs has a doc comment; PipelineVerbosity and validate_model_paths in particular have explanatory docs; DaemonError variants describe origin and recovery hint where applicable; cargo test passes; cargo clippy --all-targets -- -D warnings clean on default and gui features; cargo doc --no-deps renders cleanly.

## Proof Level

- This slice proves: static — covered by S03 audit script output, with extra manual review focused on these two files.

## Integration Closure

No integration boundaries touched.

## Verification

- None.

## Tasks

- [ ] **T01: Audit and document all public items in src/config.rs** `est:1h`
  Read src/config.rs in full. For every pub struct, enum, fn, const, and method, ensure there is a /// doc comment explaining what it represents and why it exists. Particular focus: validate_model_paths (what does it check, what errors does it return, when is it called?), PipelineVerbosity (what do the variants control, what's the default behavior?), default_config_path (already cleaned in S02 — verify), and any pub method on Config or its sub-structs (AudioConfig, VadConfig, etc.). Field-level docs on pub struct fields where the field name alone is ambiguous.
  - Files: `src/config.rs`
  - Verify: S03 audit script reports 0 undocumented pub items in src/config.rs; manual reading of the file gives a clear picture of the config schema and validation behavior

- [ ] **T02: Audit and document all public items in src/error.rs** `est:30m`
  Read src/error.rs in full. For every variant of DaemonError (and any other public error types), ensure the /// doc comment explains: (a) what condition produces this variant, (b) where in the codebase it originates (which module/operation), (c) what a caller can do about it (recovery hint, retry, fail). Particular attention to DaemonError::Config(String) — clarify whether this means config parse failure, validation failure, or both. If error messages reference docs URLs (e.g. configuration.md), verify those URLs are correct.
  - Files: `src/error.rs`
  - Verify: S03 audit script reports 0 undocumented pub items in src/error.rs; each DaemonError variant doc explains origin and recovery hint

- [ ] **T03: Run final verification — test, clippy, doc, audit script** `est:10m`
  Run cargo test, cargo test --features gui, cargo clippy --all-targets -- -D warnings, cargo clippy --all-targets --features gui -- -D warnings, cargo doc --no-deps, and the S03 audit script. All must pass and audit must report 0 undocumented public items in src/.
  - Verify: All cargo invocations exit 0; audit script reports 0

## Files Likely Touched

- src/config.rs
- src/error.rs
