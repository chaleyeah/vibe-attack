---
id: T01
parent: S03
milestone: M011
key_files:
  - src/ui/wizard.rs
key_decisions:
  - Added manual_key to PttCaptureState (not as a local) so egui text_edit binding survives frame repaints
  - Auto-advance on DownloadStatus::Done uses idempotent probe::run() per-frame — no new state needed
  - Uinput warning frame reuses Frame::NONE + fill pattern from copy_command_row — no new egui APIs
duration: 
verification_result: passed
completed_at: 2026-04-29T01:40:42.668Z
blocker_discovered: false
---

# T01: Fixed four wizard UX bugs: manual PTT key persists across frames, install-model auto-advances on Done, uinput note wrapped in dark-amber frame, download failure prepends HuggingFace CDN redirect hint

**Fixed four wizard UX bugs: manual PTT key persists across frames, install-model auto-advances on Done, uinput note wrapped in dark-amber frame, download failure prepends HuggingFace CDN redirect hint**

## What Happened

Four wizard-side bugs from S03-RESEARCH were addressed in `src/ui/wizard.rs`:

**1. manual_key frame-local reset (wizard.rs:653):** Added `manual_key: String` field to `PttCaptureState` (alongside existing `listening`, `captured_key`, `handle`, `error`). Updated `PttCaptureState::new()` to initialize it to `String::new()`. In `show_configure_ptt`, replaced `let mut manual_key = String::new()` with `&mut ptt.manual_key` so the text edit widget binds to persistent state. On successful save, `ptt.manual_key.clear()` is called before `probe::run()` so the field resets cleanly.

**2. Install-model auto-advance (wizard.rs:384–390):** In the `DownloadStatus::Done` arm of `show_install_model`, added `*state = probe::run()` before the Re-check button render. This re-probes the filesystem every frame while Done, so the wizard advances automatically when re-entered after the download handle was reaped. The manual Re-check button is kept as a fallback.

**3. Uinput note legibility (wizard.rs:570–573):** Replaced the bare `egui::Color32::YELLOW` `colored_label` with an `egui::Frame::NONE` block using `fill(Color32::from_rgb(64, 50, 0))` (dark amber) and `inner_margin(Margin::same(6))`. The label inside uses `Color32::from_rgb(255, 200, 60)`. Pattern matches the existing `copy_command_row` Frame usage.

**4. HuggingFace redirect hint (download_model failure path):** On `ureq::get(MODEL_URL).call()` failure, the `DownloadStatus::Failed` message now prepends: "HuggingFace serves a 302 redirect to a CDN — if your network blocks the CDN this will fail.\n" followed by the raw error string.

## Verification

Ran `cargo test --features gui --lib ui::wizard:: -- --test-threads=1`: 5 tests passed (3 pre-existing + 2 new: `manual_key_persists_in_state`, `manual_key_default_empty`). Ran `cargo build --release --features gui --bin vibe-attack-config`: compiled successfully in 6.19s.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features gui --lib ui::wizard:: -- --test-threads=1` | 0 | ✅ pass | 4740ms |
| 2 | `cargo build --release --features gui --bin vibe-attack-config` | 0 | ✅ pass | 6190ms |

## Deviations

None — all four changes implemented exactly as specified in the task plan.

## Known Issues

None.

## Files Created/Modified

- `src/ui/wizard.rs`
