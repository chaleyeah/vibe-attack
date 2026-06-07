# GSD State

**Active Milestone:** M013: CI Build Revamp & Package Distribution
**Active Slice:** None
**Phase:** validating-milestone
**Requirements Status:** 0 active · 6 validated · 0 deferred · 0 out of scope

## Milestone Registry
- ✅ **M001:** Migration
- ✅ **M002:** Rebrand to vibe-attack
- ✅ **M003:** First-Run GUI Wizard + Setup Script
- ✅ **M004:** Runtime UX — System Tray + Daemon Control
- ✅ **M005:** Distribution Packaging
- ✅ **M006:** CI test pipeline
- ✅ **M007:** Codebase Cleanup & Documentation
- ✅ **M008:** UI / Tray Runtime Control
- ✅ **M009:** Pack UX — Editor, Import/Export, Full HD2 Coverage
- ✅ **M010:** Distribution — AppImage, AUR, First-Run Wizard
- ✅ **M011:** M011
- ✅ **M012:** GUI Redesign — Tactical Field Equipment Aesthetic
- 🔄 **M013:** CI Build Revamp & Package Distribution
- ⬜ **M014:** v1.1 - STT Accuracy, VAD Overhaul, Sound Feedback UI

## Recent Decisions
- D001 (M007 / S01 — discovered during research as the only real bug surfaced by the cleanup pass.): Pin profiles directory canonical format to {name}/pack.yaml subdirectories (not flat *.yaml files) -> load_profiles in src/ui/config_app.rs is fixed in M007/S01 to read only subdirectory profiles. Pack::load_from_dir and handle_switch_profile already use this format; the UI is now aligned. Flat *.yaml profiles are no longer recognized.
- D002 (M007 — established as part of the documentation cleanup milestone; intended to persist as a project-wide convention going forward.): Every public item in src/ requires a /// doc comment -> M007 establishes a project convention: every pub fn/struct/enum/trait/type/const/mod in src/ must carry a /// doc comment. Enforced by the Python audit script in M007-RESEARCH.md, which is run at the end of S03, S04, and milestone close.
- D003 (M007 / S02 — applied to all existing instances in src/; intended to persist as a project convention.): Every unsafe block and #[allow(...)] in src/ requires an adjacent justifying comment -> M007/S02 enforces: every `unsafe impl`, `unsafe fn`, and `#[allow(...)]` annotation in src/ must be preceded by a comment explaining the safety invariant or the lint justification. The unsafe impl Send/Sync on Dispatcher is the canonical example — it is correct (rodio OutputStream is non-Send but only ever invoked from the dispatcher's owning thread) and now carries a // SAFETY: comment.
- D004 (M007 — set during planning as the primary scope guardrail.): M007 explicitly excludes behavior changes; the only functional change is the load_profiles bug fix -> M007 ships zero behavior changes other than the load_profiles UI bug fix. No new features, no error-handling refactors, no API changes, no packaging changes, no CI changes. Visibility narrowing (DispatcherState pub→pub(crate)) is permitted because it is verifiably internal-only.

## Blockers
- None

## Next Action
Validate milestone M013 before completion.
