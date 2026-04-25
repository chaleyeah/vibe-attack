use crate::config::MacroConfig;
use crate::input::inject::{KeyStep, MacroCmd};
use crate::pipeline::matcher::PhraseMatcher;
use crate::pipeline::sound::SoundPlayer;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

pub struct DispatcherState {
    pub flags: Arc<RwLock<HashMap<String, bool>>>,
}

impl DispatcherState {
    pub fn new() -> Self {
        Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, flag: &str) -> bool {
        *self.flags.read().unwrap().get(flag).unwrap_or(&false)
    }

    pub fn set(&self, flag: &str, value: bool) {
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

    fn check_condition(&self, if_flag: &Option<String>) -> bool {
        match if_flag {
            None => true,
            Some(if_flag) => {
                let required_val = !if_flag.starts_with('!');
                let flag_name = if if_flag.starts_with('!') {
                    &if_flag[1..]
                } else {
                    if_flag.as_str()
                };
                self.state.get(flag_name) == required_val
            }
        }
    }

    pub fn process(&self, transcript: &str) {
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
                    let new_val = !set_flag.starts_with('!');
                    let flag_name = if set_flag.starts_with('!') {
                        &set_flag[1..]
                    } else {
                        set_flag.as_str()
                    };
                    self.state.set(flag_name, new_val);
                }
            }
        }
    }
}
