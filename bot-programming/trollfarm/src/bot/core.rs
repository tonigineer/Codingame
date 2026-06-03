use crate::game::{Side, Action};

pub struct Bot {
    #[allow(dead_code)]
    pub side: Side,
    pub actions: Vec<Action>,
}

impl Bot {
    #[must_use]
    pub fn new() -> Self {
        Self {
            side: Side::Me,
            actions: Vec::new(),
        }
    }
}

impl Default for Bot {
    fn default() -> Self {
        Self::new()
    }
}
