use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Config {
    pub max_guess: usize,
    pub guess_time: usize,
    pub start_year: usize,
    pub end_year: usize,
}

impl Config {
    pub fn new(max_guess: usize, start_year: usize, end_year: usize) -> Self {
        Self {
            max_guess,
            guess_time: 0,
            start_year,
            end_year,
        }
    }
}

