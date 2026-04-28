# S06: Final distribution UAT

**Goal:** Scaffold `docs/distribution-proofs/final/` with three per-distro transcripts (Debian 12, Fedora 39, Arch) and a README, plus structural tests in `tests/distribution_proofs.rs` asserting all required fields are present. Transcripts seed at `STATUS: pending VM run` per MEM079 policy; real VM runs are a human operator deliverable that converts pending → ok and is out of code-execution scope on this host.
**Demo:** Three transcripts under docs/distribution-proofs/final/ — one per distro — each showing stratagem fired by voice from a clean AppImage install

## Must-Haves

- `docs/distribution-proofs/final/README.md` exists and documents the merged transcript format; `docs/distribution-proofs/final/{debian12,fedora39,arch}/transcript.md` exist and contain all 8 required fields with `STATUS: pending VM run`; three new `*_final_transcript_has_required_fields` tests in `tests/distribution_proofs.rs` pass under `--test-threads=1`; full `cargo test --test distribution_proofs --test packaging --test wizard_proofs --test ui_distribution -- --test-threads=1` regression guard remains green.

## Proof Level

- This slice proves: contract — structural scaffolding only. The slice proves the proof-directory contract (fields present, STATUS values valid). Real-runtime proof (`STATUS: ok` from a clean VM install end-to-end) is a follow-up human operator task, not a code deliverable. This matches the established M010 pattern: appimage and wizard slices also seeded `pending VM run` transcripts and passed tests against them.

## Integration Closure

Upstream consumed: `docs/distribution-proofs/appimage/README.md` field structure (7 AppImage fields), `docs/distribution-proofs/wizard/README.md` field structure (10 wizard fields), `tests/distribution_proofs.rs` `assert_transcript` pattern. New wiring: none — this slice adds artifacts that are read by tests, but no runtime composition. What remains for milestone usability: the actual VM runs to convert all three transcripts from `pending VM run` → `ok`. That step requires real Debian 12, Fedora 39, and Arch VMs with linuxdeploy/appimagetool installed and a working AppImage download from a real release tag — none of which can run on this host. Documented in S06 SUMMARY follow-ups for human operator.

## Verification

- No runtime observability surfaces are introduced. Transcripts themselves are the operational evidence artifact: a future agent inspecting `docs/distribution-proofs/final/{distro}/transcript.md` can read the STATUS field to know whether the full UAT loop has run, and the per-field values (APPIMAGE_VERSION, WIZARD_COMPLETED, STRATAGEM_FIRED) localize which step failed. No log/metric/state changes elsewhere.

## Tasks

- [x] **T01: Scaffold docs/distribution-proofs/final/ with three pending transcripts and structural tests** `est:45m`
  Create the final-UAT proof directory mirroring the existing `appimage/` and `wizard/` patterns, then add three structural tests to `tests/distribution_proofs.rs` asserting field presence.

**Context (executor reads this — no other docs available):**

M010 produces a vibe-attack AppImage and AUR package. Slices S01–S05 already shipped per-distro proof transcripts under `docs/distribution-proofs/appimage/{debian12,fedora39,arch}/transcript.md` (AppImage build/run proof) and `docs/distribution-proofs/wizard/{debian12,fedora39,arch}/transcript.md` (first-run wizard UAT proof). S06 closes the milestone by adding a `final/` directory whose three transcripts prove the *full* end-user loop on each distro: download AppImage → run → wizard completes → stratagem fires by voice.

Real VM runs cannot execute on this host (linuxdeploy/appimagetool absent — MEM078). The deliverable here is the **structural scaffolding** that passes tests with `STATUS: pending VM run` placeholders (MEM079 policy). Human operators convert pending → ok later.

**Step 1 — Create the final/ directory README.**

Write `docs/distribution-proofs/final/README.md`. Mirror the layout of `docs/distribution-proofs/appimage/README.md`. Document:
- Directory layout (debian12/, fedora39/, arch/ subdirs)
- Transcript field format (the 8 fields listed below)
- STATUS values: `ok`, `pending VM run`, `failed:<reason>`
- Reproduction steps per distro: download AppImage from Releases page → chmod +x → run → step through wizard → fire stratagem by voice
- A note that `pending VM run` transcripts are acceptable per MEM079 policy and are converted to `ok` by human operators with real VMs.

**Step 2 — Seed the three per-distro transcripts.**

Write each of:
- `docs/distribution-proofs/final/debian12/transcript.md`
- `docs/distribution-proofs/final/fedora39/transcript.md`
- `docs/distribution-proofs/final/arch/transcript.md`

Each file MUST contain exactly these 8 structured fields, one per line, in this order, all values set to `pending` except STATUS which must be the literal string `pending VM run`:

```
STATUS: pending VM run
DISTRO: pending
KERNEL: pending
APPIMAGE_VERSION: pending
APPIMAGE_SIZE_BYTES: pending
WIZARD_COMPLETED: pending
STRATAGEM_FIRED: pending
INSTALL_METHOD: appimage
```

Follow the fields with a `## Reproduction Notes` section documenting the per-distro reproduction recipe: install libfuse2 (Debian) / fuse-libs (Fedora) / fuse2 (Arch), download AppImage from Releases page, chmod +x, run, step through wizard end-to-end, fire a stratagem by voice, fill in the field values from observed output. Use `os-release` PRETTY_NAME for DISTRO, `uname -r` for KERNEL, `./vibe-attack-x86_64.AppImage --version` for APPIMAGE_VERSION, `stat -c %s` for APPIMAGE_SIZE_BYTES.

**Step 3 — Append three structural tests to tests/distribution_proofs.rs.**

The existing file (read in your inputs) has `assert_transcript()` for AppImage transcripts hard-coded to 7 specific fields. Do NOT modify the existing helper or existing tests. Instead:

1. Add a NEW helper `assert_final_transcript(rel: &str)` that checks the 8 final-UAT fields (`STATUS:`, `DISTRO:`, `KERNEL:`, `APPIMAGE_VERSION:`, `APPIMAGE_SIZE_BYTES:`, `WIZARD_COMPLETED:`, `STRATAGEM_FIRED:`, `INSTALL_METHOD:`). The helper accepts STATUS values: `STATUS: ok`, `STATUS: pending VM run`, and any line beginning with `STATUS: failed:`.
2. Add three `#[test]` functions: `debian12_final_transcript_has_required_fields`, `fedora39_final_transcript_has_required_fields`, `arch_final_transcript_has_required_fields`. Each calls `assert_final_transcript("docs/distribution-proofs/final/<distro>/transcript.md")`.

Reuse the existing `project_root()` and `read_file()` helpers — do not duplicate them.

**Step 4 — Verify.**

Run `cargo test --test distribution_proofs -- --test-threads=1` (serial is mandatory — MEM080). All tests in the file (existing 6 + 3 new = 9) must pass.

Then run the broader regression guard: `cargo test --test distribution_proofs --test packaging --test wizard_proofs --test ui_distribution -- --test-threads=1`. All tests must pass.

**Constraints / pitfalls:**
- STATUS must be the exact string `pending VM run` (lowercase, with the space) — anything else fails MEM079.
- All 8 fields must be present in each transcript or the test panics.
- Do NOT attempt to build the AppImage on this host — linuxdeploy/appimagetool are absent.
- Do NOT modify existing assert_transcript() or existing tests in tests/distribution_proofs.rs — only add new helper + tests.
- Do NOT commit — .gsd/ planning docs are managed externally; the README/transcripts/tests are tracked in git but commit decisions are made elsewhere.
  - Files: `docs/distribution-proofs/final/README.md`, `docs/distribution-proofs/final/debian12/transcript.md`, `docs/distribution-proofs/final/fedora39/transcript.md`, `docs/distribution-proofs/final/arch/transcript.md`, `tests/distribution_proofs.rs`
  - Verify: cargo test --test distribution_proofs --test packaging --test wizard_proofs --test ui_distribution -- --test-threads=1 && grep -q 'STATUS: pending VM run' docs/distribution-proofs/final/debian12/transcript.md && grep -q 'STATUS: pending VM run' docs/distribution-proofs/final/fedora39/transcript.md && grep -q 'STATUS: pending VM run' docs/distribution-proofs/final/arch/transcript.md && grep -q 'STRATAGEM_FIRED' docs/distribution-proofs/final/debian12/transcript.md && grep -q 'INSTALL_METHOD: appimage' docs/distribution-proofs/final/arch/transcript.md

## Files Likely Touched

- docs/distribution-proofs/final/README.md
- docs/distribution-proofs/final/debian12/transcript.md
- docs/distribution-proofs/final/fedora39/transcript.md
- docs/distribution-proofs/final/arch/transcript.md
- tests/distribution_proofs.rs
