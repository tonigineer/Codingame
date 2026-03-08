use std::io;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

macro_rules! diff_fields {
    ($self:expr, $other:expr, $($field:ident),+) => {
        $(
            if $self.$field != $other.$field {
                eprintln!("{}: {} -> {}", stringify!($field), $self.$field, $other.$field);
            }
        )+
    };
}

#[derive(Clone)]
pub struct GameState {
    units: i32,
    opp_units: i32,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    /// Creates a new `GameState` from stdin input.
    ///
    /// # Panics
    /// Panics if reading from stdin fails.
    #[must_use]
    pub fn new() -> Self {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs: Vec<&str> = input_line.split_whitespace().collect();

        Self {
            units: parse_input!(inputs[0], i32),
            opp_units: parse_input!(inputs[1], i32),
        }
    }
    /// Updates the `GameState` from stdin input.
    ///
    /// # Panics
    /// Panics if reading from stdin fails.
    pub fn update(&mut self) {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs: Vec<&str> = input_line.split_whitespace().collect();

        self.units = parse_input!(inputs[0], i32);
        self.opp_units = parse_input!(inputs[1], i32);
    }

    pub fn diff(&self, other: &GameState) {
        diff_fields!(self, other, units, opp_units);
    }
}
