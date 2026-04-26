# M002: Rebrand to vibe-attack

**Vision:** Rename the project from hd-linux-voice to vibe-attack across every surface — source, tests, XDG config prefix, packaging, docs — with a clean break (no migration shim) so the codebase consistently reflects its true identity before any external distribution channels accumulate.

## Success Criteria

- Zero `hd[_-]linux[_-]voice` matches (case-insensitive) under src/, tests/, docs/ (excluding latency-proofs/), packaging/, Cargo.toml, README.md, CONTRIBUTING.md, config*.yaml, demo_hd2.yaml, about.toml, about.hbs, examples/.
- `cargo build --all-features` succeeds and produces `target/debug/vibe-attack` and `target/debug/vibe-attack-config`.
- Daemon launched against an empty home creates `~/.config/vibe-attack/` and reads configuration from that path.
- `packaging/appimage/build.sh` produces a `vibe-attack-x86_64.AppImage` artifact and `--help` runs through it.
- `tests/documentation.rs::readme_has_correct_project_name` is inverted (requires `vibe-attack`, forbids `hd-linux-voice`) and passes.
- The empty `.planning/phases/07-rebrand-from-hd-linux-voice-to-vibe-attack/` marker directory is deleted.

## Slices

- [x] **S01: Source rename — Cargo package, binaries, XDG prefix, log strings** `risk:high` `depends:[]`
  > After this: `cargo build --all-features` succeeds; `target/debug/vibe-attack --help` and `target/debug/vibe-attack-config --help` both run; `rg -i 'hd[_-]linux[_-]voice' -- src/ Cargo.toml examples/` returns zero matches.

- [x] **S02: Test suite update — XDG fixture paths and documentation regression-guard inversion** `risk:high` `depends:[S01]`
  > After this: every file under `tests/` is updated; the README is rewritten in the same task as the test polarity inversion; the test suite builds; `rg -i 'hd[_-]linux[_-]voice' -- tests/ README.md` returns zero.

- [x] **S03: Documentation and config-template rewrite** `risk:medium` `depends:[S02]`
  > After this: `docs/*.md` (excluding `docs/latency-proofs/`), `CONTRIBUTING.md`, `config.example.yaml`, `config.yaml`, `demo_hd2.yaml`, `about.toml`, `about.hbs` all reference `vibe-attack`; `rg -i 'hd[_-]linux[_-]voice' -- docs/configuration.md docs/troubleshooting.md docs/uinput-setup.md docs/latency-baseline.md CONTRIBUTING.md config.example.yaml config.yaml demo_hd2.yaml about.toml about.hbs` returns zero.

- [x] **S04: Packaging — PKGBUILD, AppImage build script, desktop entry** `risk:medium` `depends:[S01]`
  > After this: `packaging/appimage/build.sh` produces a runnable `vibe-attack-x86_64.AppImage`; the desktop file is renamed and updated; PKGBUILD references `pkgname=vibe-attack`; invoking the AppImage with `--help` exits zero.

- [x] **S05: Cleanup — delete legacy marker directory and final grep sweep** `risk:low` `depends:[S03,S04]`
  > After this: `.planning/phases/07-rebrand-from-hd-linux-voice-to-vibe-attack/` is deleted; the milestone-wide grep sweep (`rg -i 'hd[_-]linux[_-]voice' -- src/ tests/ docs/ packaging/ Cargo.toml README.md CONTRIBUTING.md config*.yaml demo_hd2.yaml about.toml about.hbs examples/`) returns zero, excluding `docs/latency-proofs/` historical artifacts.

- [x] **S06: Final integration — end-to-end smoke and milestone summary** `risk:low` `depends:[S05]`
  > After this: the renamed daemon launches against an empty home and creates `~/.config/vibe-attack/`; the renamed GUI binary's config app reads/writes the same on-disk config; the milestone summary documents the manual `mv` migration command for the developer's existing config and the pending GitHub-slug decision; M002 closes.

## Boundary Map

### S01 → S02

Produces:
- Cargo package and `[[bin]]` targets renamed to `vibe-attack` and `vibe-attack-config`.
- `src/bin/hd-linux-voice-config.rs` renamed on disk to `src/bin/vibe-attack-config.rs`.
- `xdg::BaseDirectories::with_prefix("vibe-attack")` in every source-side call site.
- All source-level log/error strings, doc comments, and constants reference `vibe-attack`.
- `examples/audio_probe.rs` updated.
- `cargo build --all-features` succeeds.

Consumes:
- nothing (first slice).

### S01 → S03

Produces:
- A buildable codebase under the new name. S03 (docs) consumes "the binary names users will type" from S01.

### S02 → S03

Produces:
- All `tests/` files updated, including XDG fixture paths under the new prefix and the inverted `readme_has_correct_project_name` regression guard.
- README.md fully rewritten in the same task as the test inversion (S02-T01).

Consumes:
- S01's renamed source.

### S02 → S04

Produces:
- A test suite that compiles. S04 (packaging) consumes the binary names from S01 but doesn't depend on test changes; declared dependency is for sequencing, not data.

### S03 → S05

Produces:
- All `docs/*.md` (excluding `docs/latency-proofs/`), `CONTRIBUTING.md`, `config.example.yaml`, `config.yaml`, `demo_hd2.yaml`, `about.toml`, `about.hbs` updated to reference `vibe-attack`.

Consumes:
- S02's already-rewritten README (avoid double-touching).

### S04 → S06

Produces:
- `packaging/PKGBUILD` updated to `pkgname=vibe-attack`.
- `packaging/appimage/build.sh` updated.
- `packaging/appimage/hd-linux-voice.desktop` renamed to `packaging/appimage/vibe-attack.desktop` with internal `Exec=`/`Icon=` updated.
- A successful AppImage build run producing `vibe-attack-x86_64.AppImage`.

Consumes:
- S01's renamed binaries.

### S05 → S06

Produces:
- The empty `.planning/phases/07-rebrand-from-hd-linux-voice-to-vibe-attack/` directory deleted.
- Any final stragglers from the milestone-wide grep sweep cleaned up.

Consumes:
- All prior slices' renamed surfaces.

### S06 (terminal)

Produces:
- A milestone summary documenting:
  - The manual `mv ~/.config/hd-linux-voice ~/.config/vibe-attack` command for the developer's existing config.
  - The pending GitHub-repo-slug decision and how to update `Cargo.toml`'s `repository` URL once chosen.
  - Verification evidence: build output, grep sweep zero-match proof, AppImage `--help` output.

Consumes:
- All prior slices.
