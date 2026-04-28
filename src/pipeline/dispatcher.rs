use crate::config::MacroConfig;
use crate::input::inject::{KeyStep, MacroCmd};
use crate::pipeline::matcher::PhraseMatcher;
use crate::pipeline::sound::SoundPlayer;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

/// Outcome returned by `Dispatcher::process()` so callers can emit JSONL events.
#[derive(Debug)]
pub enum DispatchOutcome {
    /// A macro matched and fired. `macro_id` and `score` identify which one.
    Fired { macro_id: String, score: f32 },
    /// No phrase matched above the confidence threshold.
    NoMatch,
}

pub(crate) struct DispatcherState {
    pub(crate) flags: Arc<RwLock<HashMap<String, bool>>>,
}

impl Default for DispatcherState {
    fn default() -> Self {
        Self::new()
    }
}

impl DispatcherState {
    pub(crate) fn new() -> Self {
        Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub(crate) fn get(&self, flag: &str) -> bool {
        *self.flags.read().unwrap().get(flag).unwrap_or(&false)
    }

    pub(crate) fn set(&self, flag: &str, value: bool) {
        self.flags.write().unwrap().insert(flag.to_string(), value);
    }
}

/// Receives STT transcripts, finds the best-matching macro phrase, and fires the macro.
///
/// The dispatcher is shared across threads via `Arc` (see `SAFETY` comments below).
/// Live macro registry updates arrive through [`Dispatcher::update_macros`] without
/// restarting the pipeline.
pub struct Dispatcher {
    state: DispatcherState,
    matcher: RwLock<PhraseMatcher>,
    macros: Arc<RwLock<Vec<MacroConfig>>>,
    sound_player: Option<SoundPlayer>,
    macro_tx: Sender<MacroCmd>,
    default_dwell_ms: u64,
    default_gap_ms: u64,
}

// SAFETY: rodio's OutputStream (held inside SoundPlayer) is !Send, but Dispatcher only ever
// accesses sound_player from the single thread that owns it. No reference to sound_player is
// shared across threads, so manual Send is sound.
unsafe impl Send for Dispatcher {}
// SAFETY: same invariant as Send — all access to sound_player is serialised through the single
// owning thread; no concurrent access can occur across thread boundaries.
unsafe impl Sync for Dispatcher {}

impl Dispatcher {
    /// Create a new `Dispatcher`.
    ///
    /// `threshold` is the minimum fuzzy-match score (0.0–1.0) for a phrase to fire.
    /// `default_dwell_ms` and `default_gap_ms` are applied to key steps that omit their own timing.
    pub fn new(
        threshold: f32,
        macros: Vec<MacroConfig>,
        macro_tx: Sender<MacroCmd>,
        default_dwell_ms: u64,
        default_gap_ms: u64,
    ) -> Self {
        let sound_player = match SoundPlayer::new() {
            Ok(player) => Some(player),
            Err(e) => {
                tracing::warn!("Sound player disabled: {}", e);
                None
            }
        };

        Self {
            state: DispatcherState::new(),
            matcher: RwLock::new(PhraseMatcher::new(threshold)),
            macros: Arc::new(RwLock::new(macros)),
            sound_player,
            macro_tx,
            default_dwell_ms,
            default_gap_ms,
        }
    }

    /// Replace the live macro registry without restarting the pipeline.
    pub fn update_macros(&self, new_macros: Vec<MacroConfig>) {
        let mut macros = self.macros.write().unwrap();
        *macros = new_macros;
        tracing::info!("Registry updated with {} macros", macros.len());
    }

    /// Return the number of macros currently registered.
    pub fn macro_count(&self) -> usize {
        self.macros.read().unwrap().len()
    }

    /// Return the current confidence threshold.
    pub fn threshold(&self) -> f32 {
        self.matcher.read().unwrap().threshold()
    }

    /// Change the phrase-match confidence threshold without rebuilding the pipeline.
    ///
    /// `threshold` is clamped to `[0.0, 1.0]`; NaN is treated as 0.0.
    pub fn update_threshold(&self, threshold: f32) {
        let clamped = if threshold.is_nan() {
            0.0_f32
        } else {
            threshold.clamp(0.0, 1.0)
        };
        let old = self.matcher.read().unwrap().threshold();
        *self.matcher.write().unwrap() = PhraseMatcher::new(clamped);
        tracing::info!(old, new = clamped, "dispatcher threshold updated");
    }

    fn check_condition(&self, if_flag: &Option<String>) -> bool {
        match if_flag {
            None => true,
            Some(if_flag) => {
                if let Some(stripped) = if_flag.strip_prefix('!') {
                    !self.state.get(stripped)
                } else {
                    self.state.get(if_flag.as_str())
                }
            }
        }
    }

    /// Match `transcript` against the macro registry and fire the best-scoring macro.
    ///
    /// Returns [`DispatchOutcome::Fired`] when a phrase matches above the threshold,
    /// or [`DispatchOutcome::NoMatch`] otherwise. Side effects: plays an optional
    /// sound file and sends a [`MacroCmd`] to the input injector.
    pub fn process(&self, transcript: &str) -> DispatchOutcome {
        let macros = self.macros.read().unwrap();
        let candidates = macros
            .iter()
            .filter(|m| self.check_condition(&m.if_flag))
            .filter_map(|m| m.phrase.as_ref().map(|p| (m.name.as_str(), p.as_str())));

        if let Some((best_match_name, score)) = self.matcher.read().unwrap().find_best_match(transcript, candidates) {
            tracing::info!(macro_name = best_match_name, score, "Firing macro");
            if let Some(mac) = macros.iter().find(|m| m.name == best_match_name) {
                if let Some(sound_path) = &mac.sound {
                    if let Some(player) = &self.sound_player {
                        if let Err(e) = player.play(sound_path) {
                            tracing::error!("Failed to play sound for macro {}: {}", mac.name, e);
                        }
                    }
                }

                let keys = mac.keys.iter().map(KeyStep::from_config).collect();
                let _ = self.macro_tx.send(MacroCmd::Execute {
                    keys,
                    default_dwell_ms: self.default_dwell_ms,
                    default_gap_ms: self.default_gap_ms,
                });

                if let Some(set_flag) = &mac.set_flag {
                    if let Some(stripped) = set_flag.strip_prefix('!') {
                        self.state.set(stripped, false);
                    } else {
                        self.state.set(set_flag.as_str(), true);
                    }
                }

                return DispatchOutcome::Fired {
                    macro_id: best_match_name.to_string(),
                    score,
                };
            }
        }

        tracing::debug!(transcript, "No phrase matched above threshold");
        DispatchOutcome::NoMatch
    }

    /// Fire a macro by exact name, bypassing phrase matching.
    ///
    /// Acquires a read lock on the macro registry, locates the first entry whose
    /// `name` equals `name`, plays its optional sound, sends `MacroCmd::Execute`
    /// to the injection thread, and returns `Ok(DispatchOutcome::Fired)` with
    /// `score: 1.0` (deliberate direct trigger convention).
    ///
    /// Returns `Err` when the macro is not found or the injection channel is closed.
    pub fn fire_named(&self, name: &str) -> Result<DispatchOutcome, String> {
        let macros = self.macros.read().unwrap();
        let mac = macros
            .iter()
            .find(|m| m.name == name)
            .ok_or_else(|| format!("macro not found: {name}"))?;

        tracing::info!(macro_name = name, "Firing macro (direct)");

        if let Some(sound_path) = &mac.sound {
            if let Some(player) = &self.sound_player {
                if let Err(e) = player.play(sound_path) {
                    tracing::error!("Failed to play sound for macro {}: {}", mac.name, e);
                }
            }
        }

        let keys: Vec<KeyStep> = mac.keys.iter().map(KeyStep::from_config).collect();
        self.macro_tx
            .send(MacroCmd::Execute {
                keys,
                default_dwell_ms: self.default_dwell_ms,
                default_gap_ms: self.default_gap_ms,
            })
            .map_err(|e| format!("injection channel closed: {e}"))?;

        Ok(DispatchOutcome::Fired {
            macro_id: name.into(),
            score: 1.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MacroConfig;

    fn make_dispatcher(threshold: f32, phrase: &str) -> Dispatcher {
        let (tx, _rx) = std::sync::mpsc::channel();
        let macro_cfg = MacroConfig {
            name: "eagle_airstrike".to_string(),
            phrase: Some(phrase.to_string()),
            if_flag: None,
            set_flag: None,
            sound: None,
            keys: vec![],
        };
        Dispatcher::new(threshold, vec![macro_cfg], tx, 50, 50)
    }

    #[test]
    fn test_update_threshold_changes_match_behavior() {
        let dispatcher = make_dispatcher(0.99, "eagle airstrike");
        // "eagal airstrike" has distance 1 from "eagle airstrike" (15 chars → score ≈ 0.933)
        // Should be below threshold 0.99 → NoMatch
        assert!(
            matches!(dispatcher.process("eagal airstrike"), DispatchOutcome::NoMatch),
            "expected NoMatch at threshold 0.99"
        );
        // Lower threshold to 0.5 → same input should now Fire
        dispatcher.update_threshold(0.5);
        assert!(
            matches!(
                dispatcher.process("eagal airstrike"),
                DispatchOutcome::Fired { .. }
            ),
            "expected Fired at threshold 0.5"
        );
    }

    #[test]
    fn test_update_threshold_clamp_negative() {
        let dispatcher = make_dispatcher(0.8, "eagle airstrike");
        dispatcher.update_threshold(-0.5);
        // threshold clamped to 0.0 → everything matches
        assert!(matches!(
            dispatcher.process("eagle airstrike"),
            DispatchOutcome::Fired { .. }
        ));
    }

    #[test]
    fn test_update_threshold_clamp_above_one() {
        let dispatcher = make_dispatcher(0.8, "eagle airstrike");
        dispatcher.update_threshold(2.0);
        // threshold clamped to 1.0 → only exact match fires; "eagal" is not exact
        assert!(matches!(
            dispatcher.process("eagal airstrike"),
            DispatchOutcome::NoMatch
        ));
        // exact input should still fire (score == 1.0 >= 1.0)
        assert!(matches!(
            dispatcher.process("eagle airstrike"),
            DispatchOutcome::Fired { .. }
        ));
    }

    #[test]
    fn test_update_threshold_nan_becomes_zero() {
        let dispatcher = make_dispatcher(0.8, "eagle airstrike");
        dispatcher.update_threshold(f32::NAN);
        // threshold clamped to 0.0 → any non-empty transcript with a candidate matches
        assert!(matches!(
            dispatcher.process("eagle airstrike"),
            DispatchOutcome::Fired { .. }
        ));
    }

    fn make_dispatcher_with_keys(phrase: &str, keys: Vec<crate::config::KeyAction>) -> (Dispatcher, std::sync::mpsc::Receiver<MacroCmd>) {
        let (tx, rx) = std::sync::mpsc::channel();
        let macro_cfg = MacroConfig {
            name: "eagle_airstrike".to_string(),
            phrase: Some(phrase.to_string()),
            if_flag: None,
            set_flag: None,
            sound: None,
            keys,
        };
        let dispatcher = Dispatcher::new(0.8, vec![macro_cfg], tx, 50, 50);
        (dispatcher, rx)
    }

    #[test]
    fn fire_named_found_emits_execute() {
        use crate::config::KeyAction;
        use std::sync::mpsc::TryRecvError;

        let keys = vec![
            KeyAction { key: "KEY_UP".to_string(), dwell_ms: None, gap_ms: None },
            KeyAction { key: "KEY_DOWN".to_string(), dwell_ms: None, gap_ms: None },
        ];
        let key_count = keys.len();
        let (dispatcher, rx) = make_dispatcher_with_keys("eagle airstrike", keys);

        let outcome = dispatcher.fire_named("eagle_airstrike").expect("fire_named must succeed");
        match outcome {
            DispatchOutcome::Fired { macro_id, score } => {
                assert_eq!(macro_id, "eagle_airstrike");
                assert!((score - 1.0).abs() < 1e-6, "score must be 1.0, got {score}");
            }
            DispatchOutcome::NoMatch => panic!("expected Fired, got NoMatch"),
        }

        let cmd = rx.recv().expect("must have received exactly one MacroCmd");
        match cmd {
            MacroCmd::Execute { keys, .. } => {
                assert_eq!(keys.len(), key_count, "keys vec must match configured KeyAction count");
            }
            MacroCmd::Shutdown => panic!("unexpected Shutdown cmd"),
        }
        assert!(
            matches!(rx.try_recv(), Err(TryRecvError::Empty)),
            "must have received exactly one MacroCmd"
        );
    }

    #[test]
    fn fire_named_missing_returns_err() {
        use std::sync::mpsc::TryRecvError;

        let (dispatcher, rx) = make_dispatcher_with_keys("eagle airstrike", vec![]);

        let result = dispatcher.fire_named("does_not_exist");
        match result {
            Err(msg) => assert!(
                msg.contains("macro not found"),
                "error must contain 'macro not found', got: {msg}"
            ),
            Ok(_) => panic!("expected Err, got Ok"),
        }

        assert!(
            matches!(rx.try_recv(), Err(TryRecvError::Empty)),
            "no MacroCmd must be sent when macro is not found"
        );
    }
}
