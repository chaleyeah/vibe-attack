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

pub struct Dispatcher {
    state: DispatcherState,
    matcher: PhraseMatcher,
    macros: Arc<RwLock<Vec<MacroConfig>>>,
    sound_player: Option<SoundPlayer>,
    macro_tx: Sender<MacroCmd>,
    default_dwell_ms: u64,
    default_gap_ms: u64,
}

// Dispatcher is safe to send between threads since all its fields are either Send or Arc-wrapped
unsafe impl Send for Dispatcher {}
unsafe impl Sync for Dispatcher {}

impl Dispatcher {
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
            matcher: PhraseMatcher::new(threshold),
            macros: Arc::new(RwLock::new(macros)),
            sound_player,
            macro_tx,
            default_dwell_ms,
            default_gap_ms,
        }
    }

    pub fn update_macros(&self, new_macros: Vec<MacroConfig>) {
        let mut macros = self.macros.write().unwrap();
        *macros = new_macros;
        tracing::info!("Registry updated with {} macros", macros.len());
    }

    pub fn macro_count(&self) -> usize {
        self.macros.read().unwrap().len()
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

    pub fn process(&self, transcript: &str) -> DispatchOutcome {
        let macros = self.macros.read().unwrap();
        let candidates = macros
            .iter()
            .filter(|m| self.check_condition(&m.if_flag))
            .filter_map(|m| m.phrase.as_ref().map(|p| (m.name.as_str(), p.as_str())));

        if let Some((best_match_name, score)) = self.matcher.find_best_match(transcript, candidates) {
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
}
