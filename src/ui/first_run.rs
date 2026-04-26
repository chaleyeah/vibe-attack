/// Setup steps in first-run wizard order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupStep {
    CreateConfig,
    InstallModel,
    SetupUinput,
    ConfigurePtt,
}

/// Pure-logic state for the first-run setup wizard.
pub struct FirstRunState {
    config_exists: bool,
    model_installed: bool,
    uinput_accessible: bool,
    ptt_configured: bool,
}

impl FirstRunState {
    /// Construct from environment probe results.
    pub fn from_checks(
        config_exists: bool,
        model_installed: bool,
        uinput_accessible: bool,
        ptt_configured: bool,
    ) -> Self {
        Self {
            config_exists,
            model_installed,
            uinput_accessible,
            ptt_configured,
        }
    }

    /// True when every setup prerequisite is satisfied.
    pub fn is_setup_complete(&self) -> bool {
        self.config_exists
            && self.model_installed
            && self.uinput_accessible
            && self.ptt_configured
    }

    /// Ordered list of steps that are not yet done (wizard order).
    pub fn steps_remaining(&self) -> Vec<SetupStep> {
        let mut steps = Vec::new();
        if !self.config_exists {
            steps.push(SetupStep::CreateConfig);
        }
        if !self.model_installed {
            steps.push(SetupStep::InstallModel);
        }
        if !self.uinput_accessible {
            steps.push(SetupStep::SetupUinput);
        }
        if !self.ptt_configured {
            steps.push(SetupStep::ConfigurePtt);
        }
        steps
    }

    /// First step the user still needs to complete, or None if done.
    pub fn first_incomplete_step(&self) -> Option<SetupStep> {
        self.steps_remaining().into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_false_returns_all_four_steps() {
        let state = FirstRunState::from_checks(false, false, false, false);
        assert!(!state.is_setup_complete());
        let steps = state.steps_remaining();
        assert_eq!(steps.len(), 4);
        assert_eq!(steps[0], SetupStep::CreateConfig);
        assert_eq!(steps[1], SetupStep::InstallModel);
        assert_eq!(steps[2], SetupStep::SetupUinput);
        assert_eq!(steps[3], SetupStep::ConfigurePtt);
    }

    #[test]
    fn all_true_returns_no_steps_and_is_complete() {
        let state = FirstRunState::from_checks(true, true, true, true);
        assert!(state.is_setup_complete());
        assert!(state.steps_remaining().is_empty());
        assert!(state.first_incomplete_step().is_none());
    }

    #[test]
    fn partial_true_returns_correct_remaining_steps() {
        // Config done, rest pending
        let state = FirstRunState::from_checks(true, false, false, false);
        let steps = state.steps_remaining();
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0], SetupStep::InstallModel);
    }

    #[test]
    fn first_incomplete_step_returns_first_false() {
        let state = FirstRunState::from_checks(true, true, false, false);
        assert_eq!(state.first_incomplete_step(), Some(SetupStep::SetupUinput));
    }

    #[test]
    fn only_ptt_missing_returns_configure_ptt_step() {
        let state = FirstRunState::from_checks(true, true, true, false);
        assert!(!state.is_setup_complete());
        assert_eq!(
            state.first_incomplete_step(),
            Some(SetupStep::ConfigurePtt)
        );
    }
}
