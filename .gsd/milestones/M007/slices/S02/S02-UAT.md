# S02: Internal consistency — safety comments, alias notes, lint annotations — UAT

**Milestone:** M007
**Written:** 2026-04-27T11:47:53.447Z

# S02 UAT — Internal Consistency Annotations

## Preconditions
- Rust toolchain available (`cargo check`, `cargo test`)
- Working directory: repo root (`/home/chadmin/Github/hd-linux-voice`)

---

## TC-01: SAFETY comments on unsafe impls in dispatcher.rs

**Steps:**
1. Run: `grep -B1 'unsafe impl' src/pipeline/dispatcher.rs`

**Expected outcome:**
- Exactly 2 matches
- Each `unsafe impl` line is immediately preceded by a line containing `// SAFETY:`
- The SAFETY comment for `Send` mentions the owning-thread invariant
- The SAFETY comment for `Sync` mentions no concurrent access

---

## TC-02: SegCfg alias has an explanatory comment

**Steps:**
1. Run: `grep -B1 'SegCfg' src/pipeline/coordinator.rs | head -10`

**Expected outcome:**
- The line immediately before `use ... VadConfig as SegCfg` (or the alias definition) contains a comment explaining the alias is for local readability at segmentation-config construction sites

---

## TC-03: #[allow] annotations have justification comments

**Steps:**
1. Run: `grep -rn -B1 '#\[allow(' src/ --include='*.rs'`

**Expected outcome:**
- Exactly one `#[allow(` match (jsonl.rs:106, `clippy::too_many_arguments`)
- The line immediately above it contains a comment explaining why the function legitimately needs that many arguments (each arg maps to a JSONL event schema field)

---

## TC-04: Dual get_socket_path functions have cross-reference comments

**Steps:**
1. Run: `grep -B3 'fn get_socket_path' src/control/mod.rs src/control/client.rs`

**Expected outcome:**
- In `mod.rs`: comment above the function mentions `place_runtime_file` (creates directory) and references `control/client.rs`
- In `client.rs`: comment above the function mentions `find_runtime_file` (read-only) and references `control/mod.rs`

---

## TC-05: default_config_path has a single, non-duplicate doc block

**Steps:**
1. Run: `sed -n '255,265p' src/config.rs`

**Expected outcome:**
- Exactly one `///` doc block above `pub fn default_config_path`
- The block is 2 lines (summary + path expansion detail)
- No consecutive duplicate `///` lines with identical content

---

## TC-06: Cargo compilation clean

**Steps:**
1. Run: `cargo check --all-targets`

**Expected outcome:**
- Exit code 0
- Output ends with `Finished \`dev\` profile` with no errors or warnings

---

## TC-07: Tests pass

**Steps:**
1. Run: `cargo test`

**Expected outcome:**
- Exit code 0
- All non-ignored tests pass
- Hardware-gated tests (e.g. `keyword_spotter_loads_and_decodes_silence`) appear as `ignored` — this is expected and correct

---

## Edge Cases

**EC-01: No new unsafe blocks without SAFETY comments**
- Run: `grep -rn 'unsafe impl\|unsafe fn' src/ | grep -v '// SAFETY'`
- Expected: zero matches (all unsafe items have SAFETY comments on the preceding line, so this grep pattern — looking for unsafe on the same line as absence of SAFETY — returns nothing meaningful; the B1 grep in TC-01 is the canonical check)

**EC-02: No stray duplicate doc comment pairs**
- Run: `grep -n '///' src/config.rs | awk 'NR>1 && prev+1==$1 {print}' prev=$1` (or manual inspection)
- Expected: no run of more than 2 consecutive `///` lines in the vicinity of `default_config_path`
