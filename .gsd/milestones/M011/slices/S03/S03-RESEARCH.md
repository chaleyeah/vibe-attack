# S03: UI polish from proof-run findings — Research

**Date:** 2026-04-28

## Summary

S03 is structurally blocked: all four wizard UAT transcripts are `STATUS: pending VM run` with empty `## Findings` blocks. No real VM runs have been executed, so there is no empirical basis for UI fixes. The S02 summary explicitly warns: "S03 should be held until at least some wizard VM runs complete so it has real findings to act on."

Despite the absence of VM-run findings, code inspection reveals several **pre-existing UX issues** that are independently fixable from reading the source — no VM required to identify them. These fall into three categories: wizard panel issues visible from reading `src/ui/wizard.rs`, config screen issues visible from `src/bin/vibe-attack-config.rs`, and tray menu issues visible from `src/ui/tray.rs`. These are documented below for the planner to schedule as concrete tasks.

S03 execution should proceed in two phases: (1) fix the code-inspection issues immediately, and (2) add a task that populates findings from the `## Findings` blocks of wizard transcripts once VM runs complete. Phase 2 tasks are human-gated and should be planned as a single task with a PRECONDITION gate.

## Recommendation

**Implement the code-visible polish items now; gate the transcript-driven items on VM run completion.** The code-inspection issues are real UX problems independent of any VM run. The planner should scope tasks to the specific files and patterns identified below. Do not block the entire slice on VM runs — the code-visible fixes are worth shipping regardless.

The HuggingFace download URL (`MODEL_URL` in wizard.rs) uses a redirect-prone CDN path. The wizard currently falls through silently if `ureq` cannot follow the 302 redirect — the `Failed` state shows but the wording is not actionable. This is a known pitfall from the README and worth hardening.

## Implementation Landscape

### Key Files

- `src/ui/wizard.rs:330–401` — `show_install_model` panel: download size label says "~75 MB" but actual model is ~75 MB (ggml-tiny.en.bin). This is accurate. However, the `DownloadStatus::Done` branch (line 384) shows "Download complete." and a "Re-check" button but does NOT auto-advance — the wizard stays on step 2 until the user manually clicks "Re-check". The re-probe inside `show_wizard` (line 207) only fires when `dl.handle` is `Some` and finished. If the download completes and the handle is reaped, but the `DownloadStatus::Done` arm is entered next frame, the re-probe at line 207 is already gone — the user is stuck. This is a real stuck-state bug. Fix: auto-invoke `*state = probe::run()` inside `DownloadStatus::Done` arm when state has not advanced (i.e., when model step is still incomplete).

- `src/ui/wizard.rs:621–669` — `show_configure_ptt` panel: the manual key entry `text_edit_singleline` uses a local `let mut manual_key = String::new()` (line 653) — this resets to empty on every frame. The user cannot type across frames; the text is lost immediately. This is a significant usability bug. Fix: the manual_key state needs to live in `PttCaptureState` (add a `manual_key: String` field), not as a frame-local variable.

- `src/ui/wizard.rs:601–619` — `copy_command_row`: uses `egui::Frame::NONE` (line 604) which may not exist in older egui versions. The `TextEdit` widget is fed `&mut cmd.to_string().as_str()` (line 609) — this creates a temporary `String` from `&str`, then borrows `&mut &str`. This pattern compiles but the edit is discarded every frame. For a read-only code block this is fine functionally, but the `desired_width(f32::INFINITY)` call on a read-only `TextEdit` inside a `Frame` inside a `horizontal` layout can cause layout issues on narrow windows. This is low-priority cosmetic.

- `src/ui/wizard.rs:508–579` — `show_setup_uinput` panel: line 570 shows a `YELLOW` note about systemd v258+/CachyOS using the 'input' group. This note uses `egui::Color32::YELLOW` which renders poorly on light themes. The note is important but visually fragile. Fix: use a warning `Frame` with a colored background, or a standard colored label with a more legible color.

- `src/bin/vibe-attack-config.rs:388–390` — PTT key row: `ui.label(format!("PTT key: {}", app.config.ptt_binding))` shows the raw key string (e.g. "KEY_F13") with no affordance to change it from the main config screen. The wizard sets it once; there is no way to re-configure PTT from the main UI post-wizard. This is a scope question (deferred in M008) but the label alone is confusing — users don't know they can't change it. Fix: add a short note "(configured in wizard)" or a "Reconfigure…" button that re-enters the wizard PTT step.

- `src/bin/vibe-attack-config.rs:295–296` — `ui.heading("Vibe Attack"); ui.separator()` — the window has no visual indication of which wizard step the user is on beyond the step heading inside the panel. A step indicator (e.g. "Step X of 4") is already rendered per-panel but there is no top-level progress indicator. Low priority; already addressed per-step in each panel heading.

- `src/ui/tray.rs:199–343` — `menu()`: the "Mode" submenu (lines 288–329) has no icon on the `SubMenu` parent item (unlike "Profiles" which has `icon_name: "folder".into()`). The "Quit" item uses `std::process::exit(0)` (line 338) which bypasses normal eframe shutdown and will not flush pending writes. Fix: use `ctx.send_viewport_cmd(egui::ViewportCommand::Close)` instead — but the tray doesn't have access to the egui context. The correct fix is to set an `AtomicBool` shutdown flag (same pattern as `open_window`) and have the eframe loop exit cleanly. This is a correctness issue, not cosmetic.

- `src/ui/tray.rs:185–196` — `tool_tip()`: `DaemonState::Idle` tooltip says "Idle — listening for wake word" — this is only accurate in Wake mode. In PTT mode, "Idle" means waiting for the PTT key, not a wake word. The tooltip should reflect the active mode. Fix: include `active_mode` in the tooltip description.

- `src/ui/wizard.rs:404–492` — `download_model`: the model URL (`MODEL_URL = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin"`) uses `ureq::get(...).call()` with no explicit redirect following. `ureq` 2.x follows redirects by default, but HuggingFace serves a 302 → CDN URL. The README already documents this as a common pitfall. If the CDN is restricted, the `Failed` message will be a raw HTTP error. Fix: surface a friendlier message in the `DownloadStatus::Failed` arm that mentions the HuggingFace CDN redirect as a possible cause.

### Build Order

1. **Fix the `manual_key` frame-local bug** (wizard.rs PTT step) — this is the highest-impact usability bug. Add `manual_key: String` to `PttCaptureState`, update `new()` and references in `show_configure_ptt`.

2. **Fix the `DownloadStatus::Done` auto-advance bug** (wizard.rs install model step) — prevents users getting stuck on step 2 after download. Add the re-probe call inside the `Done` arm of `show_wizard`'s main body (before `match dl.current()`).

3. **Fix the tray Quit behavior** (tray.rs) — add a `shutdown: Arc<AtomicBool>` to `TrayHandle`, wire it through the `Quit` menu item, and poll it in the eframe loop alongside `take_open_request()`. Remove `std::process::exit(0)`.

4. **Fix the tray tooltip** (tray.rs) — include active_mode in the description string.

5. **Add uinput step warning legibility** (wizard.rs) — change the `YELLOW` label to a `Frame`-backed warning or use `egui::Color32::from_rgb(230, 180, 0)` with a `RichText`.

6. **Populate findings-driven tasks after VM runs** — a single task with PRECONDITION: at least one wizard transcript has `SCENARIO_A: ok` or `SCENARIO_A: failed:*` (not `pending`). The task reads `## Findings` blocks from all four wizard transcripts and implements the reported issues.

### Verification Approach

- `cargo test --test distribution_proofs -- --test-threads=1` — must continue to pass (these tests don't exercise wizard UI, but they validate the proof scaffolding isn't broken)
- `cargo test --test wizard_proofs -- --test-threads=1` — must continue to pass
- `cargo build --release --features gui --bin vibe-attack-config` — GUI build must compile cleanly
- `cargo test --lib` — unit tests for `rewrite_ptt_key` and probe module must pass
- Manual smoke test on dev host: launch `vibe-attack-config --skip-wizard`, confirm main config screen loads; confirm tray appears; quit via tray and verify clean exit (no abrupt termination)

## Constraints

- All wizard UI code is behind `#[cfg(feature = "gui")]` — changes to wizard.rs must stay in the `inner` module. The `PttCaptureState` struct is exported from `inner` (via `pub use inner::*` at line 11), so adding a field to it is safe.
- `ksni` D-Bus callbacks must never block (MEM042 pattern) — tray fix must use an `AtomicBool` flag, not a direct eframe call.
- The tray doesn't own an egui `Context` — any tray→eframe signalling must go through the existing `Arc<AtomicBool>` pattern in `TrayHandle`.
- `egui::Frame::NONE` availability: verify against the egui version in Cargo.toml before using it; the `copy_command_row` function (line 604) already uses it, so it is available.
- `manual_key` state must be added to `PttCaptureState` in `wizard.rs`, not in the calling binary (`vibe-attack-config.rs`), to keep wizard state self-contained.

## Common Pitfalls

- **`manual_key` frame-local variable** — the existing bug (reset each frame) means the current text_edit_singleline does nothing useful. Any fix must move state into `PttCaptureState`. The `lost_focus() && key_pressed(Enter)` guard (line 655) is correct but unreachable because the value is always empty.
- **Auto-advance after download** — adding `*state = probe::run()` inside the `Done` arm of `show_wizard` must happen in the main harvesting block (the `if let Some(h) = &dl.handle` block at lines 206–214), not inside `show_install_model`. The harvesting block already calls `*state = probe::run()` at line 210 — the issue is when a user re-enters the wizard after the handle has been reaped. In that case, `dl.current()` returns `Done` but no re-probe happens. The `show_install_model` panel can handle this by also calling probe when it sees `Done` and the state is still on InstallModel.
- **Tray `process::exit`** — search for all callers. There is exactly one: line 338. Replacing it with an `AtomicBool` flag requires threading the new shutdown flag through `TrayHandle` (add a `shutdown` field alongside `open_window`) and polling it in `VibeAttackConfigApp::ui` alongside `take_open_request()`.

## Open Risks

- **No VM wizard findings yet** — The primary intended input for this slice (empirical UI bugs found during VM runs) does not exist. If VM runs are not completed before S03 executes, the slice produces only the code-visible fixes. The planner should structure the findings-driven task as a distinct, human-gated step.
- **egui API churn** — `egui::Frame::NONE` and `request_repaint_after` signatures may differ across egui minor versions. Check `Cargo.toml` for the pinned version before using any API not already present in the codebase.
