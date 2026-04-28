# S04: Import / Export dialogs — UAT

**Milestone:** M009
**Written:** 2026-04-28T03:09:51.084Z

# S04 UAT — Import / Export dialogs

## Preconditions
- `vibe-attack-config` binary built with `--features gui` (`cargo build --features gui --bin vibe-attack-config`)
- At least one existing profile in `$XDG_CONFIG_HOME/vibe-attack/profiles/` (e.g. the hd2 pack from S01)
- A writable temp directory available for export targets

---

## TC-01: Export Pack produces a .hdpack file

**Steps:**
1. Launch `vibe-attack-config`.
2. Open the Pack Editor panel.
3. Select an existing pack in the profile list.
4. Click **Export Pack**.
5. In the native file dialog, choose a destination path ending in `.hdpack` (e.g. `/tmp/my_export.hdpack`).
6. Click Save in the dialog.

**Expected:**
- Dialog closes immediately.
- No red error text appears in the editor toolbar.
- `/tmp/my_export.hdpack` exists on disk and is a valid ZIP file (`unzip -l /tmp/my_export.hdpack` lists `pack.yaml`).
- `tracing::info!` log line `"Export Pack: succeeded"` appears in stdout with `dest_path` and `pack_name` fields.

---

## TC-02: Import Pack from exported .hdpack refreshes the profile list

**Steps:**
1. Export a pack to `/tmp/test_import.hdpack` (see TC-01).
2. Click **Import Pack**.
3. In the native file dialog, select `/tmp/test_import.hdpack`.
4. Click Open.

**Expected:**
- Dialog closes immediately.
- No red error text appears.
- The profile list (left panel of the main config window) now includes the imported pack name.
- The editor immediately shows the imported pack's categories and macros.
- `tracing::info!` log line `"Import Pack: succeeded"` appears with `zip_path`, `pack_name`, and `macro_count`.

---

## TC-03: Import → Export → Re-import round-trip preserves macro content

**Steps:**
1. Select a pack with at least 3 macros in different categories.
2. Note the macro names and key sequences.
3. Export to `/tmp/roundtrip.hdpack`.
4. Import `/tmp/roundtrip.hdpack`.
5. Browse the imported pack in the editor.

**Expected:**
- All category names are present and in the same order.
- All macro names are present with identical key sequences.
- No macros are missing or duplicated.

---

## TC-04: Cancel on Import dialog is a no-op

**Steps:**
1. Click **Import Pack**.
2. In the native file dialog, click **Cancel** (or press Escape).

**Expected:**
- Editor state is unchanged (same pack selected, same error state).
- No red error text appears.
- No crash or freeze.

---

## TC-05: Cancel on Export dialog is a no-op

**Steps:**
1. Click **Export Pack**.
2. In the native file dialog, click **Cancel**.

**Expected:**
- Editor state is unchanged.
- No red error text appears.
- No crash or freeze.

---

## TC-06: Import of malformed .hdpack surfaces error inline

**Steps:**
1. Create a corrupt file: `echo "not a zip" > /tmp/bad.hdpack`.
2. Click **Import Pack** and select `/tmp/bad.hdpack`.

**Expected:**
- Red error text appears in the editor toolbar describing the failure (e.g. "invalid zip archive").
- Profile list is unchanged — no partial or empty profile was added.
- `tracing::warn!` log line `"Import Pack: failed"` appears with the `reason` field.

---

## TC-07: Import overwrites existing pack with same name without crashing

**Steps:**
1. Export a pack to `/tmp/overwrite_test.hdpack`.
2. Import the same `.hdpack` again (the profile directory already exists).

**Expected:**
- Import completes successfully (import_to removes the existing dir before extracting).
- No error text appears.
- Profile list reflects the refreshed pack.

---

## Automated Regression Baseline
All UAT cases above have automated coverage at the backend level via `tests/pack_lifecycle.rs` (TC-03 field-equivalence, TC-06 corruption path via existing pack_hd2_bundle tests). TC-01, TC-02, TC-04, TC-05, TC-07 require the rfd dialog which cannot be driven headlessly — manual smoke is required before S06 milestone UAT sign-off.
