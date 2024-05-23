#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Constraint {
    FixedMaxSize(f32),

    PercentMaxSize(f32),

    NoConstraint,
}

impl Default for Constraint {
    fn default() -> Self {
        Self::NoConstraint
    }
}

impl Constraint {
    pub(super) fn apply(&self, value: f32) -> f32 {
        match *self {
            Constraint::FixedMaxSize(max) => value.min(max),
            Constraint::PercentMaxSize(percentage) => value * percentage,
            Constraint::NoConstraint => value,
        }
    }
}
