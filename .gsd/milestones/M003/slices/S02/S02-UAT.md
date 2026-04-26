# S02: Environment probe — UAT

**Milestone:** M003
**Written:** 2026-04-26T00:15:47.398Z

# S02 UAT: Environment Probe

## Prerequisites
- Rust toolchain with `cargo test`

## Test Cases

### 1. Unit tests pass
```bash
cargo test --lib ui::probe
```
Expected: 8 tests pass, 0 fail.

### 2. No stub in production code
```bash
grep -rn "from_checks" src/bin/
```
Expected: no output (stub only exists inside probe::run() in src/ui/probe.rs).

### 3. Probe reflects real environment
Launch `vibe-attack-config --features gui` on a machine with config missing:
Expected: wizard shows setup steps (probe returned false for at least one check).

Launch with a valid config, model, and uinput access:
Expected: wizard is skipped, main config app shown.

