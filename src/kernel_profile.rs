//! Kernel profiles identify optional layers without weakening older checkpoint contracts.

/// Each implemented layer combination has one definition identity and checkpoint version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelProfile {
    JsonAuthoring,
    CliComposition,
    ConstrainedAuthoring,
    ConstrainedComposition,
}

impl KernelProfile {
    /// Select the independent composition and constraints layers.
    pub fn from_layer_flags(compose: bool, constraints: bool) -> Self {
        match (compose, constraints) {
            (false, false) => Self::JsonAuthoring,
            (true, false) => Self::CliComposition,
            (false, true) => Self::ConstrainedAuthoring,
            (true, true) => Self::ConstrainedComposition,
        }
    }

    /// Resolve only identities whose semantics are implemented by this kernel.
    pub fn from_definition_id(identity: &str) -> Option<Self> {
        [
            Self::JsonAuthoring,
            Self::CliComposition,
            Self::ConstrainedAuthoring,
            Self::ConstrainedComposition,
        ]
        .into_iter()
        .find(|profile| profile.definition_id() == identity)
    }

    /// Return the stable declaration identity for this layer combination.
    pub fn definition_id(self) -> &'static str {
        match self {
            Self::JsonAuthoring => "bootstrap-json-authoring-v1",
            Self::CliComposition => "bootstrap-cli-composition-v1",
            Self::ConstrainedAuthoring => "bootstrap-json-constraints-v1",
            Self::ConstrainedComposition => "bootstrap-cli-constraints-v1",
        }
    }

    /// Return the checkpoint format admitted by this profile.
    pub fn checkpoint_version(self) -> u32 {
        match self {
            Self::JsonAuthoring => 1,
            Self::CliComposition => 2,
            Self::ConstrainedAuthoring => 3,
            Self::ConstrainedComposition => 4,
        }
    }

    /// Identify profiles that carry an active authored CLI interface.
    pub fn has_composition(self) -> bool {
        matches!(self, Self::CliComposition | Self::ConstrainedComposition)
    }

    /// Identify profiles that permit an explicit schema attachment.
    pub fn has_constraints(self) -> bool {
        matches!(
            self,
            Self::ConstrainedAuthoring | Self::ConstrainedComposition
        )
    }
}
