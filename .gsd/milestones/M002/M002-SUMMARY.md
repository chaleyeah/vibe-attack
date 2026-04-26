---
milestone: M002
title: Rebrand to vibe-attack
status: complete
completed: 2026-04-25
---

# M002 Summary — Rebrand to vibe-attack

## What was done

Complete rename of the project from `hd-linux-voice` to `vibe-attack` across every surface:

- **Source (S01):** Cargo package name, binary targets, XDG prefix (`vibe-attack`), log strings, `src/bin/hd-linux-voice-config.rs` → `src/bin/vibe-attack-config.rs`, `examples/audio_probe.rs`.
- **Tests (S02):** XDG fixture paths updated to `vibe-attack/` prefix; `tests/documentation.rs` regression guard inverted — now asserts README *contains* `vibe-attack` and *does not contain* `hd-linux-voice`.
- **Docs & config (S03):** `docs/*.md` (excluding latency-proofs historical artifacts), `CONTRIBUTING.md`, `config.example.yaml`, `config.yaml`, `demo_hd2.yaml`, `about.toml`, `about.hbs` all updated.
- **Packaging (S04):** `packaging/PKGBUILD` → `pkgname=vibe-attack`; `packaging/appimage/build.sh` updated; `packaging/appimage/hd-linux-voice.desktop` renamed to `vibe-attack.desktop` with `Exec=`/`Icon=` updated.
- **Cleanup (S05):** Deleted `.planning/phases/07-rebrand-from-hd-linux-voice-to-vibe-attack/.gitkeep` and `.planning/phases/08-fix-compilation-errors-from-dependency-updates-and-api-misma/.gitkeep`.

## Verification evidence

```
# Final grep sweep — zero matches except intentional regression-guard assertions
$ rg -i 'hd[_-]linux[_-]voice' -- src/ tests/ docs/ packaging/ Cargo.toml README.md \
    CONTRIBUTING.md config.example.yaml config.yaml demo_hd2.yaml about.toml about.hbs examples/ \
  | grep -v 'latency-proofs' | grep -v '!contains'
(no output)

# Build succeeds with correct binary names
$ cargo build 2>&1 | tail -3
   Compiling vibe-attack v0.8.0 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s)
$ ls target/debug/vibe-attack target/debug/vibe-attack-config
target/debug/vibe-attack  target/debug/vibe-attack-config
```

## Migration note for existing installs

```sh
mv ~/.config/hd-linux-voice ~/.config/vibe-attack
```

## Pending decisions

- GitHub repository slug: currently `hd-linux-voice`. Once the repo is renamed, update `repository` in `Cargo.toml`.
