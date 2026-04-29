---
estimated_steps: 9
estimated_files: 1
skills_used: []
---

# T01: Fix wizard usability bugs (manual_key state, install-model auto-advance, uinput note legibility, download error hint)

Address the four wizard-side bugs identified in S03-RESEARCH that are visible from code inspection of src/ui/wizard.rs.

(1) manual_key frame-local reset — show_configure_ptt currently uses `let mut manual_key = String::new()` inside the panel function (wizard.rs:653), resetting to empty every frame so the user cannot type. Add a `manual_key: String` field to PttCaptureState (alongside listening, captured_key, handle, error). Update PttCaptureState::new() and Default to initialize it to String::new(). In show_configure_ptt, replace the local with `&mut ptt.manual_key`. After a successful manual save, clear `ptt.manual_key`.

(2) Install-model auto-advance — when the wizard re-renders show_install_model with DownloadStatus::Done but the wizard step is still InstallModel (because the user re-entered the wizard after the download handle was reaped on a previous run), the panel sits on a 'Re-check' button. Inside the DownloadStatus::Done arm of show_install_model (currently wizard.rs:384–390), call `*state = probe::run()` once before rendering the button. This is idempotent — probe::run() reads the filesystem so re-entering the panel re-detects the file and advances steps automatically. Keep the manual 'Re-check' button as a fallback.

(3) Uinput note legibility — replace the bare `egui::Color32::YELLOW` colored_label at wizard.rs:570–573 with a visually-bounded warning: wrap the note text in an `egui::Frame` with a subtle background fill (egui::Color32::from_rgb(64, 50, 0) or similar dark-amber) and use `egui::Color32::from_rgb(255, 200, 60)` for the text. Pattern matches the existing copy_command_row Frame usage (wizard.rs:604) so no new egui APIs are needed.

(4) HuggingFace redirect hint — in download_model's failure path (wizard.rs ~423), when ureq returns an error, prepend a one-line hint to the failure message: 'HuggingFace serves a 302 redirect to a CDN — if your network blocks the CDN this will fail.' Concatenate before the raw error so both are visible.

Unit tests:
- Add `tests::manual_key_persists_in_state` to wizard.rs `#[cfg(test)] mod tests`: construct PttCaptureState::new(), mutate ptt.manual_key.push_str('KEY_F13'), assert ptt.manual_key == 'KEY_F13'. This is a state round-trip test (matches MEM083 pattern of testing wizard predicates without GUI).
- Add `tests::manual_key_default_empty`: PttCaptureState::default().manual_key.is_empty().

Must stay inside the `mod inner` block (wizard.rs:14) since PttCaptureState is exported via `pub use inner::*` (line 11). Do not add fields outside `inner`.

## Inputs

- `src/ui/wizard.rs`
- `.gsd/milestones/M011/slices/S03/S03-RESEARCH.md`

## Expected Output

- `src/ui/wizard.rs`

## Verification

cargo test --features gui --lib ui::wizard:: -- --test-threads=1 && cargo build --release --features gui --bin vibe-attack-config

## Observability Impact

PTT manual entry now persists across frames — a future user-reported 'I typed but nothing happened' can be debugged by reading PttCaptureState.manual_key in test fixtures without booting a GUI. Download failure hint string makes the HuggingFace-redirect pitfall self-explanatory in logs and UI.
