# S02: VAD overhaul - robust onset and tuned defaults — UAT

**Milestone:** M014
**Written:** 2026-06-07T21:11:18.712Z

Run vibe-attack daemon in wake-word mode. Say the wake word followed by a stratagem name. Confirm the utterance is captured even if there is a brief pause between wake word detection and speaking the command. Run with RUST_LOG=debug to observe per-frame VAD scores in stderr.
