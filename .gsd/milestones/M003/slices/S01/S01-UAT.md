# S01: Setup script — UAT

**Milestone:** M003
**Written:** 2026-04-26T00:11:33.773Z

# S01 UAT: Setup Script

## Prerequisites
- Fresh environment (or temp XDG dirs to isolate)
- vibe-attack repo checked out

## Test Cases

### 1. Help flag
```bash
bash scripts/setup.sh --help
```
Expected: prints OPTIONS and STEPS sections, exits 0.

### 2. Syntax check
```bash
bash -n scripts/setup.sh
```
Expected: no output, exits 0.

### 3. Full dry-run
```bash
bash scripts/setup.sh --yes --dry-run
```
Expected: prints `dry-run: ...` lines for each action, no files created, exits 0.

### 4. copy_config idempotency
```bash
tmpdir=$(mktemp -d)
XDG_CONFIG_HOME=$tmpdir bash scripts/setup.sh --yes --step=copy_config
# should create file
XDG_CONFIG_HOME=$tmpdir bash scripts/setup.sh --yes --step=copy_config
# should print 'already exists — skipping'
```
Expected: first run creates `$tmpdir/vibe-attack/config.yaml`; second run skips.

### 5. validate all-fail
```bash
XDG_CONFIG_HOME=$(mktemp -d) XDG_DATA_HOME=$(mktemp -d) bash scripts/setup.sh --step=validate
```
Expected: config and model checks fail with path in message; exits 1.

### 6. validate SETUP COMPLETE
Run full `--yes` setup in a temp env (or manually satisfy all four checks), then run validate.
Expected: all four checks pass; prints SETUP COMPLETE; exits 0.

### 7. uinput already-satisfied
```bash
bash scripts/setup.sh --step=setup_uinput
```
On a machine where /dev/uinput exists and user is in input group.
Expected: both checks print 'already satisfied'; no sudo commands run.

