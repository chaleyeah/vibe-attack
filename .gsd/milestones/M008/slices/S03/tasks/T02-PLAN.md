---
estimated_steps: 13
estimated_files: 1
skills_used: []
---

# T02: Map every DaemonState to a distinct tray icon name with unit tests

Extract the icon-name match in `src/ui/tray.rs::VibeTray::icon_name` into a free function `pub(crate) fn icon_name_for_state(state: Option<&DaemonState>) -> &'static str` so the mapping is unit-testable without spawning a tray. Mapping:
- None → "audio-input-microphone-muted"
- Some(Muted) → "audio-input-microphone-muted"
- Some(Idle) → "audio-input-microphone"
- Some(Listening) → "audio-input-microphone-high"
- Some(Recording) → "audio-input-microphone"

Update `VibeTray::icon_name` to call the new function with `self.current_state().daemon_state.as_ref()`. Add a `#[cfg(test)] mod tests` block in `tray.rs` with five unit tests:
- `icon_name_for_none_is_muted`
- `icon_name_for_idle`
- `icon_name_for_listening`
- `icon_name_for_recording`
- `icon_name_for_muted`

No D-Bus, no async, no ksni in tests — they call the free function directly. Per project convention (MEM002) the new free function gets a `///` doc comment summarising the mapping rationale.

## Inputs

- ``src/ui/tray.rs``
- ``src/control/protocol.rs``

## Expected Output

- ``src/ui/tray.rs` — extracts icon_name_for_state free function, rewires VibeTray::icon_name to call it, adds #[cfg(test)] mod tests with 5 mapping tests`

## Verification

cargo test --features gui icon_name_for && cargo build && cargo build --features gui
