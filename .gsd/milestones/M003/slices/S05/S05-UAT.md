# S05: Integration smoke tests — UAT

**Milestone:** M003
**Written:** 2026-04-26T00:24:10.345Z

# S05 UAT: Integration Smoke Tests

## Prerequisites
- Rust toolchain

## Test Cases

### 1. All ui module tests pass
```bash
cargo test --lib ui::
```
Expected: 17 tests pass, 0 fail.

### 2. Full lib test suite (excluding pre-existing pack failure)
```bash
cargo test --lib
```
Expected: only pack::tests::test_pack_export_import_with_sounds fails (pre-existing); all other tests pass.

### 3. No compilation errors in probe/wizard/config_app sources
```bash
cargo check --lib
```
Expected: exits 0 with only pre-existing warnings.

