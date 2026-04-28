---
estimated_steps: 12
estimated_files: 1
skills_used: []
---

# T02: Add Pack::import_to(zip_path, dest_dir) and make Pack::import delegate

Refactor `Pack::import` so the destination directory is injectable. Add a new public `Pack::import_to(zip_path: &Path, dest_dir: &Path) -> Result<Pack>` that contains the actual extraction logic, then make the existing `Pack::import(zip_path)` a thin wrapper that resolves `get_profiles_dir()?.join(&pack.name)` and delegates to `import_to`. Also add a unit test for `import_to` against a tempdir to lock the behaviour in.

Why: the existing `Pack::import` writes unconditionally to `get_profiles_dir()` (XDG_CONFIG_HOME). MEM005 records that the existing parallel test for export/import is already flaky under shared XDG mutation. To write a reliable round-trip integration test in T03, the import side must accept a destination path so the test can pass a tempdir. Refactoring `import` to delegate keeps backwards compatibility — all existing call sites (and existing tests using `XDG_CONFIG_HOME` env override) remain green.

Key constraints:
- Public API: `pub fn import_to(zip_path: &Path, dest_dir: &Path) -> Result<Pack>`. Signature must accept `&Path` for both args (matches existing `import` and `export` style).
- `import_to` MUST extract into `dest_dir.join(&pack.name)` — i.e. the caller passes the *parent* profiles directory, and the function appends the pack name itself. This matches the existing `import()` semantics (extracts to `get_profiles_dir()?.join(&pack.name)`).
- Preserve the path-traversal protection (`file.enclosed_name()`).
- Preserve the existing collision behaviour: if `dest_dir.join(&pack.name)` already exists, `remove_dir_all` it first. This is what callers (UI and tests) expect.
- Existing `pub fn import(zip_path: &Path) -> Result<Self>` becomes a 3-line wrapper: `let profiles_dir = get_profiles_dir()?; Self::import_to(zip_path, &profiles_dir)`.
- Add tracing::info! at the start of `import_to` with `zip_path` and `dest_dir` fields; add tracing::info! at the end with `macro_count` after successful extraction.
- Add one inline unit test in `src/pack/mod.rs` `#[cfg(test)] mod tests` (or a fresh test module) that builds a small pack, exports to tempdir, calls `import_to(&zip, &tempdir2)`, and asserts the imported `Pack` name and one macro name match. This ensures `import_to` is exercised even before T03 lands.

Failure modes (Q5): zip not found → wraps existing io::Error path; missing pack.yaml inside zip → existing context message; dest_dir not writable → propagated io::Error.
Negative tests (Q7): the unit test above covers the happy path; negative cases (malformed zip, missing pack.yaml) are already covered by existing tests that exercise `import` and will exercise the same code path through delegation.

## Inputs

- ``src/pack/mod.rs``

## Expected Output

- ``src/pack/mod.rs``

## Verification

cargo test --lib pack:: -- --test-threads=1 — the new import_to unit test plus all existing pack tests must pass; cargo test --test pack_hd2_bundle -- --test-threads=1 — existing serial export/import tests must stay green (they exercise import() via the wrapper path).
