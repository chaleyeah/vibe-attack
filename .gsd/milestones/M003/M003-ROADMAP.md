# M003: First-Run GUI Wizard + Setup Script

**Vision:** Replace the manual copy-paste setup process with a guided experience: a bash setup script for CLI/headless users, and an egui wizard that walks GUI users through every prerequisite — config file, whisper model, uinput access, and PTT key — before handing off to the live config app.

## Success Criteria

- A new user can run `scripts/setup.sh` on a fresh machine and reach a working vibe-attack config without consulting the docs
- The `vibe-attack-config` binary detects incomplete setup on launch and shows the wizard; once all steps are satisfied it transitions to the main config app
- Each wizard step performs the real action (copy config, check model path, validate uinput access, capture PTT key) rather than showing copy-paste instructions
- The main config app shows real profiles from disk, a live mic level bar, and a scrollable log feed
- All probe logic is covered by hermetic unit tests; wizard state transitions are exercised without a display server

## Slices

- [x] **S01: S01** `risk:medium` `depends:[]`
  > After this: Running `scripts/setup.sh` on a fresh system (or with --yes in a temp dir) completes all steps and exits 0; re-running is idempotent

- [x] **S02: S02** `risk:medium` `depends:[]`
  > After this: Unit tests (no display server) pass: probe returns correct booleans for each check under hermetic XDG temp dirs; /dev/uinput open failure is correctly classified as inaccessible

- [x] **S03: S03** `risk:high` `depends:[]`
  > After this: Launching vibe-attack-config on a system missing prerequisites shows the wizard; clicking through each step performs the real action; after all steps pass, the app transitions to the main config view

- [x] **S04: S04** `risk:medium` `depends:[]`
  > After this: After wizard completes, the main config view shows real profile names, a live mic level bar that responds to audio input, and log lines appearing as the daemon runs

- [x] **S05: S05** `risk:low` `depends:[]`
  > After this: cargo test covers probe module, FirstRunState transitions, and binary smoke; all pass in CI without a display server

## Boundary Map

### S01 → S02\n\nProduces:\n- `scripts/setup.sh` with documented step names matching probe check names\n\nConsumes:\n- nothing (first slice)\n\n### S02 → S03\n\nProduces:\n- `src/ui/probe.rs` with `probe::run() -> FirstRunState` and per-check reason strings\n- `FirstRunState` fields publicly readable for wizard panel decisions\n\nConsumes:\n- `scripts/setup.sh` step names (for consistent messaging)\n\n### S03 → S04\n\nProduces:\n- Wizard panels that transition to `ConfigApp` when `is_setup_complete()` is true\n- PTT key written to `config.yaml` ptt.key field\n\nConsumes:\n- `probe::run()` from S02\n\n### S04 → S05\n\nProduces:\n- `ConfigApp` populated from real disk, CPAL, and log channel\n- Full end-to-end launch path from wizard to live config view\n\nConsumes:\n- Wizard transition from S03
