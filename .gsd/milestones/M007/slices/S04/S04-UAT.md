# S04: Config and error type cleanup — UAT

**Milestone:** M007
**Written:** 2026-04-27T12:21:44.821Z

# S04: Config and error type cleanup — UAT

**Milestone:** M007
**Written:** 2026-04-27

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: S04 is a pure documentation slice — no runtime behavior changed. Correctness is fully verifiable by static analysis (audit script, cargo doc) and the existing test suite.

## Preconditions

- Working directory: `/home/chadmin/Github/hd-linux-voice`
- Rust toolchain present (`cargo`, `rustdoc`)
- Python 3 available for audit script

## Smoke Test

Run the canonical audit script against src/. It must report PASS with 0 undocumented pub items:

```bash
python3 -c "
import re, sys, os
src_dir = 'src'
undocumented = []
for root, dirs, files in os.walk(src_dir):
    for fname in files:
        if not fname.endswith('.rs'): continue
        fpath = os.path.join(root, fname)
        lines = open(fpath).readlines()
        for i, line in enumerate(lines):
            stripped = line.strip()
            if re.match(r'^pub\s+(fn|struct|enum|trait|type|const|mod)\s+', stripped):
                preceding = ''.join(lines[max(0,i-3):i])
                if '///' not in preceding and '//!' not in preceding:
                    undocumented.append(f'{fpath}:{i+1} — {stripped[:80]}')
if undocumented:
    print('UNDOCUMENTED:'); [print(' ', u) for u in undocumented]; sys.exit(1)
else:
    print('PASS: 0 undocumented pub items in src/')
"
```

**Expected:** `PASS: 0 undocumented pub items in src/`

## Test Cases

### 1. cargo test passes clean

```bash
cargo test
```

**Expected:** All tests pass. Output ends with `test result: ok. N passed; 0 failed`. Ignored tests (privileged/KWS/stress) are acceptable.

### 2. cargo check is clean on both feature sets

```bash
cargo check --all-targets
cargo check --all-targets --features gui
```

**Expected:** Both commands finish with `Finished 'dev' profile` and no errors.

### 3. cargo doc produces zero warnings

```bash
cargo doc --no-deps 2>&1 | grep -i warning
```

**Expected:** No output (zero warnings). In particular, no broken intra-doc link warnings from `src/error.rs`.

### 4. DaemonError variant docs cover condition, origin, and recovery

```bash
grep -A 8 'UinputPermissionDenied' src/error.rs | head -12
grep -A 8 'InputGroupMissing' src/error.rs | head -12
grep -A 8 'NoPttDevice' src/error.rs | head -12
grep -A 8 'Config(' src/error.rs | head -12
```

**Expected:** Each variant is preceded by a `///` doc block that mentions: (a) what condition produces the error, (b) which module/operation originates it, (c) what a user or caller can do to recover.

### 5. config.rs pub struct fields are all documented

```bash
grep -n 'pub ' src/config.rs | head -60
```

Spot-check any 5 pub fields from the output against their preceding lines:

**Expected:** Every `pub` field has a `///` doc comment on the immediately preceding line(s) explaining its semantics, default value (if fixed), or required-when conditions.

### 6. stt/mod.rs result_rx field is documented

```bash
grep -B2 'pub result_rx' src/stt/mod.rs
```

**Expected:** The two lines before `pub result_rx` include a `///` doc comment.

## Edge Cases

### Audit script handles pub(crate) items correctly

```bash
python3 -c "
import re
test_lines = ['    pub(crate) fn new() -> Self {\n', '    pub fn do_thing() -> () {\n']
for line in test_lines:
    stripped = line.strip()
    m = re.match(r'^pub\s+(fn|struct|enum|trait|type|const|mod)\s+', stripped)
    print(repr(stripped[:40]), '->', 'MATCH' if m else 'skip')
"
```

**Expected:** `pub(crate) fn new()` → `skip`; `pub fn do_thing()` → `MATCH`. Confirms the audit script does not flag restricted-visibility items.

### cargo doc intra-doc link for std::fmt::Display resolves

```bash
cargo doc --no-deps 2>&1 | grep 'Display'
```

**Expected:** No output. The `[std::fmt::Display]` link in `src/error.rs` resolves cleanly.

## Failure Signals

- Audit script exits non-zero and lists undocumented items → doc coverage gap remains
- `cargo doc` emits `warning: unresolved link` → intra-doc link regression
- `cargo test` shows unexpected failures (beyond pre-existing pack flake) → code regression introduced during doc editing
- `cargo check` errors → structural code change accidentally introduced

## Not Proven By This UAT

- Runtime behavior of config loading or error display — S04 made no behavior changes; these paths are covered by existing integration tests
- Accuracy of cross-references to docs/uinput-setup.md and docs/configuration.md — those docs are audited in S05
- The yourusername/vibe-attack placeholder URL in UinputPermissionDenied's Display output — behavior change deferred beyond M007

## Notes for Tester

The `pack::tests::test_pack_export_import_with_sounds` test exhibits transient parallel-test ordering flakiness under `--features gui`. It is pre-existing and unrelated to S04 work. If seen, run `cargo test --features gui test_pack_export_import_with_sounds` in isolation — it passes consistently alone. Do not treat it as a regression introduced by S04.
