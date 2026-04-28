---
estimated_steps: 7
estimated_files: 1
skills_used: []
---

# T03: Add wizard-completion transition unit test for setup_just_completed

The `setup_just_completed` boolean in `src/bin/vibe-attack-config.rs` (line 165, 225, 279, 307) is the edge that triggers profile-load and mic-thread spawn after the wizard finishes. It is currently UNTESTED — there is no automated test covering the `was_incomplete && is_setup_complete()` transition.

Add unit tests that exercise the FirstRunState transition logic that drives `setup_just_completed`. We cannot exec egui frames in a unit test, but we CAN test the pure transition predicate. Specifically:

1. Construct a FirstRunState with all-false; assert `is_setup_complete() == false` (was_incomplete = true).
2. Construct a fresh FirstRunState with all-true (simulating the wizard updating the state via probe::run() at the end); assert `is_setup_complete() == true`.
3. Add a helper test asserting that the boolean transition `was_incomplete && now_complete` holds when going from (false,*,*,*) to (true,true,true,true) but does NOT hold when starting already complete (relaunch scenario per S02-RESEARCH Scenario C).

Write the test in `tests/ui_distribution.rs` (or a new `tests/wizard_transition.rs` if size warrants). Use only the FirstRunState public API — do not poke private fields. The tests must compile WITHOUT `--features gui` (FirstRunState has no egui dependency).

Also add a regression test asserting that `FirstRunState::from_checks(true,true,true,true).first_incomplete_step()` is None — proving the relaunch path will not spuriously enter the wizard.

## Inputs

- ``src/ui/first_run.rs` — FirstRunState::from_checks and is_setup_complete are the transition predicates`
- ``src/bin/vibe-attack-config.rs` — lines 293-308 show the was_incomplete/is_setup_complete boolean dance to verify`

## Expected Output

- ``tests/ui_distribution.rs` — new tests `wizard_completion_transition_fires_on_incomplete_to_complete`, `wizard_completion_transition_does_not_fire_on_relaunch`, `relaunch_state_has_no_first_incomplete_step``

## Verification

cargo test --test ui_distribution -- --test-threads=1 wizard_completion_transition && cargo test --test ui_distribution -- --test-threads=1
