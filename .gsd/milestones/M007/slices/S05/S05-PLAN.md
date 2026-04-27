# S05: README, CONTRIBUTING, and docs/ accuracy pass

**Goal:** Verify every external doc file accurately reflects the current codebase. Update any drift, especially around config field names, binary names, uinput setup commands, and the build/run/configure flow.
**Demo:** README.md describes vibe-attack accurately, including the audio → keypress pipeline, build/run/configure steps, and the feature flags (default vs gui); CONTRIBUTING.md reflects the current dev setup; docs/configuration.md fields match the current Config struct; docs/troubleshooting.md references current binary names and uinput group conventions; docs/uinput-setup.md references the correct group name and udev rule; cargo test passes; cargo clippy -D warnings clean

## Must-Haves

- README.md, CONTRIBUTING.md, docs/configuration.md, docs/troubleshooting.md, and docs/uinput-setup.md are all reviewed line-by-line against the current src/; any drift is corrected; final M007 audit script run reports 0 undocumented public items in src/; success-criteria grep returns 0 unjustified hits in src/; cargo test passes; cargo clippy --all-targets -- -D warnings clean.

## Proof Level

- This slice proves: manual review — cross-referenced against current src/ contents and command output.

## Integration Closure

External documentation surface aligned with current code. No runtime boundaries touched.

## Verification

- None.

## Tasks

- [x] **T01: Verify README.md accuracy** `est:45m`
  Read README.md line-by-line. Verify: (a) project name is vibe-attack throughout, (b) the architecture description matches src/lib.rs //! crate doc from S03, (c) build/run instructions work against current Cargo.toml (cargo build, cargo build --features gui, cargo run --bin vibe-attack), (d) configuration section references the actual fields in src/config.rs, (e) feature flags described match what's in Cargo.toml [features]. Update any drift.
  - Files: `README.md`
  - Verify: Manual review confirms README matches current code; running the documented build/run commands actually works

- [x] **T02: Verify CONTRIBUTING.md accuracy** `est:20m`
  Read CONTRIBUTING.md line-by-line. Verify dev setup commands, test invocations (including hardware-gated ones), code style references, and PR workflow notes match current reality (CI workflow, clippy enforcement). Update any drift.
  - Files: `CONTRIBUTING.md`
  - Verify: Manual review; running the documented dev setup steps works on a fresh clone

- [ ] **T03: Verify docs/configuration.md accuracy** `est:45m`
  Read docs/configuration.md line-by-line. For every config field documented, confirm it exists in src/config.rs with the same name, type, and default value. Add documentation for any field present in code but missing from the doc; remove documentation for any field no longer in code. Confirm example YAML snippets parse against the current Config struct.
  - Files: `docs/configuration.md`
  - Verify: Every field in docs/configuration.md exists in src/config.rs; every pub field in src/config.rs is documented in docs/configuration.md; example YAML snippets are valid

- [ ] **T04: Verify docs/troubleshooting.md and docs/uinput-setup.md accuracy** `est:45m`
  Read both files line-by-line. Confirm: (a) binary names match (vibe-attack, vibe-attack-config), (b) uinput group name and udev rules match current install instructions and PKGBUILD/.spec/debian/, (c) command examples (e.g. journalctl filters, systemctl status) reference correct service names, (d) error messages quoted in troubleshooting.md still match current src/error.rs Display impls. Update any drift.
  - Files: `docs/troubleshooting.md`, `docs/uinput-setup.md`
  - Verify: Manual cross-reference confirms binary names, group names, error messages, and commands all match current state

- [ ] **T05: Run final M007 milestone verification** `est:15m`
  Run the complete M007 verification gate: cargo test, cargo test --features gui, cargo clippy --all-targets -- -D warnings, cargo clippy --all-targets --features gui -- -D warnings, cargo doc --no-deps, the M007-RESEARCH.md Python audit script (must report 0), and the success-criteria grep `grep -rn 'hd.linux.voice\|hd_linux_voice\|hd2_linux\|TODO\|FIXME\|HACK\|dead_code\|allow(unused' src/` (must return 0 unjustified hits; the control/mod.rs CancellationToken TODO is the only acceptable remaining hit if not yet addressed). Capture all output for the milestone summary.
  - Verify: All cargo invocations exit 0; audit script reports 0; grep returns 0 or only the documented control/mod.rs TODO

## Files Likely Touched

- README.md
- CONTRIBUTING.md
- docs/configuration.md
- docs/troubleshooting.md
- docs/uinput-setup.md
