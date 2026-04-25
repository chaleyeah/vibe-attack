# M001: M001: Migration

**Vision:** An **open source** Linux desktop application in the spirit of [VoiceAttack](https://voiceattack.

## Slices

- [x] **S01: Foundation** `risk:medium` `depends:[]`
  > After this: Install the Rust toolchain and create a compilable project skeleton: Cargo.

- [x] **S02: Pipeline Core** `risk:medium` `depends:[S01]`
  > After this: Establish Phase 2’s dependency + configuration + test scaffolding foundation: local-only model paths for VAD/STT/wake word, and opt-in heavy tests that can be run on developer machines without breaking default `cargo test`.

- [x] **S03: S03** `risk:medium` `depends:[]`
  > After this: unit tests prove phrase-matching-dispatch works

- [x] **S04: S04** `risk:medium` `depends:[]`
  > After this: unit tests prove pack-system-hd2-bundle works

- [ ] **S05: S05** `risk:medium` `depends:[]`
  > After this: unit tests prove UI + Distribution — egui config window, system tray, first-run wizard, AppImage, AUR/PKGBUILD works

- [ ] **S06: Documentation — Usage docs, troubleshooting, and contributor guides** `risk:medium` `depends:[S05]`
  > After this: unit tests prove Documentation — Usage docs, troubleshooting, and contributor guides works

- [ ] **S07: Wake Word Activation (DEFERRED from Phase 2) — Resolve dual ONNX Runtime conflict between sherpa Onnx (statically Linked ORT) and `ort` crate (dynamically Loaded ORT) so the wake Word path runs without heap corruption** `risk:medium` `depends:[S06]`
  > After this: unit tests prove Wake-word Activation (DEFERRED from Phase 2) — Resolve dual-ONNX-Runtime conflict between sherpa-onnx (statically-linked ORT) and `ort` crate (dynamically-loaded ORT) so the wake-word path runs without heap corruption works
