use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Config {
    pub max_guess: usize,
    pub guess_time: usize,
}

impl Config {
    pub fn new(max_guess: usize) -> Self {
        Self {
            max_guess,
            guess_time: 0,
        }
    }
}

