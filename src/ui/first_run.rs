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
