# M002: Rebrand to vibe-attack

**Gathered:** 2026-04-25
**Status:** Ready for planning

## Project Description

Rename the project from `hd-linux-voice` to `vibe-attack` across every surface that carries the old identifier: Cargo package and binary names, the XDG config directory prefix, the AppImage and Arch (PKGBUILD) packaging, the desktop entry, the README/CONTRIBUTING/`docs/` text, the regression-guard test in `tests/documentation.rs` (currently *enforces* `hd-linux-voice` and *forbids* `vibe-attack` — the polarity must invert), and any internal log strings or constants that include the name.

This work was originally scoped as Phase 07 in the legacy `.planning/` tree (`07-rebrand-from-hd-linux-voice-to-vibe-attack/`) but the directory was never populated, the task was never executed, and it was lost when M001 migrated into `.gsd/`.

## Why This Milestone

The project name is wrong. It's currently named after the platform (`hd-linux-voice`) rather than what it does. `vibe-attack` is the chosen identity — the name signals the actual product (voice-driven action triggers) instead of describing the runtime environment. The rename was always intended; it just didn't happen.

Doing it now, before any external users or distribution channels (AUR, AppImage releases, GitHub stars on the current slug) accumulate, keeps the cost low. Every additional release tagged with the old name multiplies the migration debt.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Run `cargo install --path .` and invoke `vibe-attack` (the daemon) and `vibe-attack-config` (the GUI) — the old binary names no longer exist on `PATH`.
- See `~/.config/vibe-attack/` as the on-disk config home for new installs (the `xdg::BaseDirectories::with_prefix` value), with no fallback or shim to the old path.
- Read `README.md`, `CONTRIBUTING.md`, and `docs/*.md` and find `vibe-attack` everywhere; `hd-linux-voice` survives only in `CHANGELOG`-style historical notes if any.
- Build the AppImage (`packaging/appimage/build.sh`) and get a `vibe-attack-x86_64.AppImage` artifact whose desktop entry, icon name, and `Exec=` line all reference `vibe-attack`.
- Run `cargo test` and have the documentation regression guard pass against the new name (with `hd-linux-voice` as the *forbidden* string).

### Entry point / environment

- Entry point: `cargo run --bin vibe-attack` (daemon), `cargo run --bin vibe-attack-config --features gui` (GUI), `packaging/appimage/build.sh` (AppImage build)
- Environment: local dev shell + Linux desktop, plus the AppImage build environment
- Live dependencies involved: ALSA/PipeWire (audio), `/dev/uinput` (input injection), XDG config dir on disk

## Completion Class

- Contract complete means: every test in `tests/` passes; `cargo build --all-features` succeeds; `tests/documentation.rs::readme_has_correct_project_name` enforces `vibe-attack` and forbids `hd-linux-voice`.
- Integration complete means: the renamed binaries launch, read config from the new XDG dir, accept PTT/wake input, inject keys via uinput, and the GUI config app reads/writes the same on-disk config the daemon consumes.
- Operational complete means: the AppImage build script produces a working `vibe-attack-x86_64.AppImage` that runs end-to-end on a clean shell with no environment leakage from the dev tree; the PKGBUILD targets the new package name and would build cleanly under `makepkg` (best-effort verified — no AUR push).

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- A clean checkout, `cargo build --all-features`, then `target/debug/vibe-attack --help` and `target/debug/vibe-attack-config --help` both run; no `hd-linux-voice` binary is produced anywhere under `target/`.
- A fresh user (no pre-existing config) launches the daemon, it creates `~/.config/vibe-attack/` (and `~/.local/share/vibe-attack/`, `~/.cache/vibe-attack/` as applicable), loads its profile, and runs the wake/PTT pipeline successfully.
- `rg -i 'hd[_-]linux[_-]voice' -- src/ tests/ docs/ packaging/ README.md CONTRIBUTING.md Cargo.toml about.toml` returns zero hits (or only matches inside intentional historical-note blocks that are explicitly listed in the slice summary).
- `cargo test` passes — including the inverted documentation regression guard.
- Cannot be simulated: the on-disk XDG directory transition (the daemon writing to `vibe-attack/` instead of `hd-linux-voice/`) and the AppImage producing a runnable artifact under the new name. Both must be exercised against a real filesystem.

## Architectural Decisions

### Clean-break XDG rename — no migration shim

**Decision:** The XDG prefix moves from `hd-linux-voice` to `vibe-attack` with no read-fallback to the old path and no automatic copy/migrate at first run.

**Rationale:** The project is pre-1.0, has no shipped distribution (no AUR release, no AppImage on a release page), and the only consumers of `~/.config/hd-linux-voice/` are the developer's own dev environment. A migration shim is dead weight: it adds a code path that would need to live forever (or be removed in a future cleanup milestone), all to spare a one-time `mv` on a single machine.

**Alternatives Considered:**
- One-shot first-run migration that copies `~/.config/hd-linux-voice/` to `~/.config/vibe-attack/` if the new dir is missing — rejected: speculative complexity for a userbase of one; trivially replaceable by a documented `mv` command.
- Read-fallback (try new, fall back to old) — rejected: encodes the legacy name into the source forever and obscures which directory is actually authoritative.

### Rename `hd-linux-voice-config` to `vibe-attack-config`

**Decision:** The GUI binary becomes `vibe-attack-config`. The `-config` suffix is preserved.

**Rationale:** Consistency with the daemon binary name. The `-config` suffix is conventional for "GUI configurator for daemon X" and doesn't carry the old project identity.

**Alternatives Considered:**
- Drop the suffix and call the GUI `vibe-attack-ui` or `vibe-attack-gui` — rejected: breaks the daemon/configurator pairing convention and forces a docs/help-text rewrite that exceeds a rename's scope.
- Collapse into a single `vibe-attack` binary with subcommands — rejected: out of scope for a rename; that's a UX redesign.

### Inversion of `tests/documentation.rs::readme_has_correct_project_name`

**Decision:** The test inverts polarity in the same slice that updates the README — `vibe-attack` becomes the required string and `hd-linux-voice` becomes the forbidden one.

**Rationale:** The test's purpose is "the README reflects the current project identity"; that intent survives the rename, only the strings change. Keeping the test (rather than deleting it) preserves the regression guard against future drift.

**Alternatives Considered:**
- Delete the test — rejected: loses a useful guard.
- Update the test before the README — rejected: would leave the test failing through the slice and break TDD ordering. The test and the README must change in the same commit/task.

---

> See `.gsd/DECISIONS.md` for the full append-only register of all project decisions.

## Error Handling Strategy

A rename is a mechanical transformation; the failure modes are:

1. **Missed reference** — `rg` sweep at the end of each slice catches stragglers. Each slice's verification includes a `rg -i 'hd[_-]linux[_-]voice'` against its scope (source / docs / packaging) and must return zero.
2. **Build break from a missed Cargo `[[bin]]` reference or import** — caught by `cargo build --all-features` at the end of S01 (the source-rename slice). Fix in place, do not bypass.
3. **AppImage build failure from path mismatch (desktop file, icon name, AppRun)** — caught by running `packaging/appimage/build.sh` to completion in the packaging slice. Failure means a slice-internal fix, not a deferral.
4. **Test failure from the inverted documentation guard** — expected to pass on the same commit that flips it; if the README change lands without the test inversion (or vice versa), the slice is incomplete.

No retries, no fallbacks, no graceful degradation — a rename is either complete or it isn't.

## Risks and Unknowns

- **External references we can't see** — the GitHub repo slug (`yourusername/hd-linux-voice` placeholder in `Cargo.toml`'s `repository` field), any pushed tags, any CI config in `.github/`. The local rename can't change a GitHub repo slug; that's a manual user action documented in the milestone summary, not a slice. Why it matters: a user reading `Cargo.toml` after the rename will still see a stale `repository = "..."` URL until they decide what the new slug is.
- **AppImage runtime path assumptions** — `packaging/appimage/build.sh` may bake in `hd-linux-voice` as a directory or filename in ways that aren't grep-visible (e.g. via shell variable expansion or hardcoded `${app}` strings). Why it matters: a missed reference means a broken AppImage that builds but doesn't run.
- **Test fixture path assumptions** — integration tests that redirect `XDG_CONFIG_HOME` for hermetic isolation create fixtures at `<dir>/hd-linux-voice/profiles/` (per the project memory note on `with_prefix`). Every such fixture must move to `<dir>/vibe-attack/profiles/`. Why it matters: a missed test fixture renders that test silently meaningless — it would still pass against a stale path while the daemon code looked at a different one. The grep sweep will catch the path strings.
- **`about.toml` / `about.hbs` license-report templates** — these may embed the package name in generated output; need to verify they regenerate cleanly post-rename. Why it matters: stale license-attribution output could ship with packages.

## Existing Codebase / Prior Art

- `Cargo.toml` — package `name`, both `[[bin]]` targets, `description`, and `repository` URL all reference the old name.
- `src/main.rs`, `src/config.rs`, `src/control/{mod,client}.rs`, `src/pack/{mod,manager}.rs`, `src/error.rs`, `src/input/inject.rs`, `src/bin/hd-linux-voice-config.rs` — source files that hardcode the name (mostly via `xdg::BaseDirectories::with_prefix` and log/error strings); the binary file itself must be renamed on disk.
- `tests/documentation.rs` — `readme_has_correct_project_name` enforces `hd-linux-voice` and rejects `vibe-attack`; polarity must invert.
- `tests/{ui_distribution,pack_hd2_bundle,...}.rs` — tests that grep against the project name or write XDG fixtures under the old prefix.
- `README.md`, `CONTRIBUTING.md`, `config.example.yaml`, `config.yaml`, `demo_hd2.yaml`, `about.toml`, `about.hbs`, `docs/{configuration,troubleshooting,uinput-setup,latency-baseline}.md`, `docs/latency-proofs/**` — text-level references.
- `packaging/PKGBUILD`, `packaging/appimage/build.sh`, `packaging/appimage/hd-linux-voice.desktop` — packaging-layer references; the desktop file itself must be renamed.
- `examples/audio_probe.rs` — has at least one stale reference per the grep.
- `.planning/phases/07-rebrand-from-hd-linux-voice-to-vibe-attack/` — empty marker directory; the rebrand was scoped here originally and never populated. Will be deleted as part of M002 cleanup.
- Project memory MEM (xdg::BaseDirectories::with_prefix) — directly relevant: confirms test fixture paths must be updated alongside the source rename.

## Relevant Requirements

None of the existing R### requirements turn on the project name; this milestone touches identity, not behavior. New requirements are not expected.

## Scope

### In Scope

- Rename the Cargo package, both binaries, and the binary source file (`src/bin/hd-linux-voice-config.rs` → `src/bin/vibe-attack-config.rs`).
- Update every source-code reference to the old name (XDG prefix, log/error strings, doc comments).
- Move the on-disk XDG config from `hd-linux-voice/` to `vibe-attack/` for new installs (no migration code).
- Update all tests, including inverting `tests/documentation.rs::readme_has_correct_project_name` and updating any XDG fixture paths.
- Update `README.md`, `CONTRIBUTING.md`, `config.example.yaml`, `config.yaml`, `demo_hd2.yaml`, `about.toml`, `about.hbs`, all `docs/*.md` files (excluding historical latency-proof artifacts which are timestamped records).
- Update `packaging/PKGBUILD` and `packaging/appimage/{build.sh,hd-linux-voice.desktop}` (rename the desktop file too).
- Delete the empty `.planning/phases/07-rebrand-from-hd-linux-voice-to-vibe-attack/` marker directory.
- Verify a build, the renamed binaries launch, and the AppImage script produces a runnable artifact under the new name.

### Out of Scope / Non-Goals

- Renaming the GitHub repository slug or pushing tags (manual user action).
- Updating the `repository` URL in `Cargo.toml` to the actual new slug — left as `yourusername/vibe-attack` until the user confirms the GitHub slug separately.
- AUR submission, AppImage release uploads, or any external publication.
- Logo/icon redesign — the icon file is renamed but its visual content is not changed.
- Migrating existing developer-machine `~/.config/hd-linux-voice/` data (documented as a one-line `mv` in the milestone summary instead).
- Historical `.planning/` and `docs/latency-proofs/` content — these are timestamped artifacts, not living docs, and remain as-is.
- Renaming any GSD artifacts in `.gsd/` (these are project-management records, not user-facing).

## Technical Constraints

- The cargo package `name` must be valid (lowercase, hyphen-separated, no leading digit) — `vibe-attack` satisfies this.
- The rename must not introduce backward-compatibility code (per the clean-break decision above).
- All work happens on the `main` branch (default isolation: worktree, per `PREFERENCES.md` if applicable; the milestone-branch worktree is fine).
- No new dependencies; this is a pure rename.

## Integration Points

- **Cargo build system** — the `[[bin]]` rename must be reflected anywhere the binary path is referenced (e.g. `[[bin]] required-features`, AppImage build script that invokes `cargo build --bin <name>`).
- **XDG Base Directory** — `xdg::BaseDirectories::with_prefix("vibe-attack")` becomes the canonical config path; tests that redirect `XDG_CONFIG_HOME` must create fixtures under the new prefix.
- **AppImage build pipeline** (`packaging/appimage/build.sh`) — produces an artifact whose name, desktop entry, and AppRun all reference the new identity.
- **Arch PKGBUILD** — `pkgname`, `source`, and any `_pkgname` variables must align with the new package name.
- **License attribution** (`about.toml` / `about.hbs`) — re-generated output must reference `vibe-attack` consistently.

## Testing Requirements

- All existing tests under `tests/` and any inline `#[cfg(test)]` modules must pass after the rename. No tests are deleted as part of this milestone; only updated.
- `tests/documentation.rs::readme_has_correct_project_name` is inverted and continues to enforce identity drift.
- An end-to-end smoke test of the renamed daemon launching and reading config from the new XDG path is exercised manually in the integration slice (no new automated test required — the existing `daemon_headless` and `pack_hd2_bundle` tests cover the path resolution once the prefix is updated).
- The AppImage build is run end-to-end in the packaging slice; success is the artifact existing and `--help` running cleanly when invoked through the AppImage.
- Per memory: `cargo test` requires user approval in auto-mode. Verification in auto-mode is static (`cargo build --all-features`, grep sweeps); runtime test confirmation is deferred to a manual user run after auto-mode completes, captured in the milestone summary.

## Acceptance Criteria

Per-slice acceptance criteria are defined in `M002-ROADMAP.md`. At the milestone level, the criteria are:

- Zero matches for `hd[_-]linux[_-]voice` (case-insensitive) under `src/`, `tests/`, `docs/` (excluding `docs/latency-proofs/`), `packaging/`, `Cargo.toml`, `README.md`, `CONTRIBUTING.md`, `config*.yaml`, `demo_hd2.yaml`, `about.toml`, `about.hbs`, `examples/`.
- `cargo build --all-features` succeeds.
- The renamed binaries (`vibe-attack`, `vibe-attack-config`) exist under `target/{debug,release}/` and `--help` runs.
- The XDG config path resolves to `~/.config/vibe-attack/` at runtime (verified by launching the daemon against an empty home and observing the directory it creates).
- The AppImage build produces a `vibe-attack-x86_64.AppImage` artifact and `--help` runs through it.
- The empty `.planning/phases/07-rebrand-from-hd-linux-voice-to-vibe-attack/` directory is deleted.
- The inverted `readme_has_correct_project_name` test passes.

## Open Questions

- GitHub repo slug — does the user want this rename to coincide with renaming the GitHub repo itself? If yes, who does the rename and when, relative to the local milestone? Current thinking: out of scope for M002; document the manual step in the milestone summary and leave the `Cargo.toml` `repository` URL with the new placeholder until the user confirms.
- Tag history / release naming — does any tag or release draft need a name change? Current thinking: probably none exist; verify in the milestone summary and act only if found.
