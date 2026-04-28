# S03: Egui editor panel — UAT

**Milestone:** M009
**Written:** 2026-04-28T02:55:10.549Z

# S03 UAT — Egui Editor Panel

**Preconditions:**
- Built with `cargo build --features gui`
- At least one profile directory exists under `$XDG_CONFIG_HOME/vibe-attack/profiles/` (e.g. `hd2/`) containing a valid `pack.yaml`
- `vibe-attack-config` binary is available at `target/debug/vibe-attack-config`

---

## TC-01: Editor panel opens on profile click

1. Launch `target/debug/vibe-attack-config`
2. Observe the profiles list in the main config window
3. Click a profile name (e.g. `hd2`)

**Expected:** Below the profiles list, the egui editor panel appears showing a heading with the pack name. No crash, no error label visible.

---

## TC-02: Category list populates correctly

1. With the editor panel open (TC-01 complete)
2. Observe the left column of the editor panel

**Expected:** Each category from `pack.yaml` appears as a selectable label. Clicking a category name highlights it and populates the middle column with that category's macros.

---

## TC-03: Macro list populates on category selection

1. Click a category with at least one macro
2. Observe the middle column

**Expected:** Macro names appear as selectable labels. Clicking a macro name highlights it and populates the right-column edit form with that macro's name, phrase, if_flag, set_flag, and key sequence.

---

## TC-04: Add Macro

1. Select a category in the left column
2. In the edit form (right column), type a new macro name (e.g. `TestMacro`), phrase (e.g. `test phrase`), and key sequence (e.g. `KEY_A,KEY_B`)
3. Click **Add Macro**

**Expected:** New macro appears in the middle column under the selected category. No red error label. `last_error` is cleared.

**Edge case — empty name:** Leave the name field blank and click Add Macro. Expected: Red error label appears with a message about empty name. Macro is NOT added.

**Edge case — bad key:** Enter `KEY_A,,KEY_B` (double comma) and click Add Macro. Expected: Red error label about empty key token. Macro is NOT added.

---

## TC-05: Update Macro

1. Select an existing macro
2. Change the phrase field to a new value
3. Click **Update Macro**

**Expected:** The macro's phrase is updated (verify by deselecting and reselecting the macro — form repopulates with the new value). No error label.

---

## TC-06: Remove Macro (two-click confirmation)

1. Select an existing macro
2. Click **Remove Macro** once

**Expected:** A confirm button appears inline. The macro is NOT removed yet.

3. Click the confirm button

**Expected:** Macro disappears from the middle column. No error label.

---

## TC-07: Add Category

1. In the toolbar above the category list, type a new category name (e.g. `NewCat`) in the text field
2. Click **Add Category**

**Expected:** `NewCat` appears in the category list. Clicking it shows an empty macro list.

---

## TC-08: Remove Category — non-empty refused

1. Select a category that has at least one macro
2. Click **Remove Category**

**Expected:** Red error label appears indicating the category is non-empty. Category is NOT removed.

---

## TC-09: Remove Category — empty succeeds

1. Remove all macros from a category (TC-06 repeated)
2. Click **Remove Category**

**Expected:** Category disappears from the left column. No error label.

---

## TC-10: Rename Category — warning + two-click confirm

1. Select a category
2. Click **Rename Category** (or enter a new name in the rename field)

**Expected:** A yellow warning label appears indicating that `if_flag`/`set_flag` references are NOT cascaded and must be manually updated. The rename is NOT applied yet.

3. Click the confirm button

**Expected:** Category name updated in the left column. Warning label clears.

---

## TC-11: Save — disk write

1. Make any CRUD change (e.g. TC-04: add a macro)
2. Click **Save**

**Expected:**
- No error label appears
- `$XDG_CONFIG_HOME/vibe-attack/profiles/<profilename>/pack.yaml` is updated on disk (verify with `cat` — new macro should appear in YAML)
- Tracing log (if visible in terminal) shows `tracing::info!` event with path and macro count

---

## TC-12: Save — daemon absent is not an error

1. Ensure vibe-attack daemon is NOT running
2. Perform TC-11

**Expected:** Save completes successfully (exit 0 from disk write). No red error label. Terminal log may show an info/warn message about SwitchProfile failing to reach daemon, but the UI does not block or show error.

---

## TC-13: Reload profile after save

1. After TC-11, close and reopen `vibe-attack-config`
2. Click the same profile name

**Expected:** Editor panel populates with the updated pack data (the macro added in TC-11 is present). Round-trip integrity confirmed.

---

## TC-14: Regression — default build unaffected

1. Run `cargo build` (no `--features gui`)

**Expected:** Exit 0, zero warnings. The `parse_key_sequence` and `build_macro_config_from_form` helpers compile; no egui imports leak outside the feature gate.

