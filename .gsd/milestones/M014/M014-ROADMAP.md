# M014: v1.1 - STT Accuracy, VAD Overhaul, Sound Feedback UI

**Vision:** Make Vibe Attack actually usable in PTT mode during HD2 gameplay by fixing the two root causes of recognition failure: Whisper hallucinates without vocabulary context, and the VAD onset gate drops phrases on the floor due to single-frame score dips. Deliver: auto-populated initial_prompt from pack phrases, a more robust sliding-window VAD onset algorithm, tuned defaults, exposed config sliders, and per-macro sound feedback UI. Ship as v1.1.0.

## Success Criteria

- PTT mode recognises HD2 stratagem names reliably when initial_prompt is populated from the active pack
- VAD onset survives a single low-confidence frame dip without resetting (N-of-M sliding window)
- Config UI exposes VAD start/stop threshold and silence duration sliders
- Per-macro sound file can be set and saved from the pack editor
- DEVICES nav icon renders without a broken-glyph box
- cargo build --features gui succeeds at version 1.1.0

## Slices

- [x] **S01: Fix broken DEVICES nav icon** `risk:low` `depends:[]`
  > After this: Launch vibe-attack-config and observe the DEVICES nav item renders a recognisable icon.

- [x] **S02: VAD overhaul - robust onset and tuned defaults** `risk:high` `depends:[S01]`
  > After this: cargo test in vad module passes including new onset tests. Config UI sliders visible in ADVANCED pane.

- [x] **S03: STT accuracy - initial prompt from active pack** `risk:medium` `depends:[S02]`
  > After this: cargo test passes. Log shows initial_prompt set line on daemon start.

- [x] **S04: Sound feedback UI - per-macro sound file picker** `risk:low` `depends:[S03]`
  > After this: Open pack editor, edit a macro, click Browse, select a .wav file, save. Reload confirms path retained.

- [x] **S05: Tracker cleanup and v1.1.0 release** `risk:low` `depends:[S04]`
  > After this: cargo build --features gui and version string in UI footer shows 1.1.0.

## Boundary Map

Not provided.
