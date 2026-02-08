use crate::fl;
use crate::timer;
use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub enum Cube {
    Two,
    #[default]
    Three,
    Four,
    Five,
    Six,
    Seven,
    // TODO: add all the other WCA events
}
impl Cube {
    pub fn as_string(&self) -> String {
        match self {
            Cube::Two => fl!("cube_two"),
            Cube::Three => fl!("cube_three"),
            Cube::Four => fl!("cube_four"),
            Cube::Five => fl!("cube_five"),
            Cube::Six => fl!("cube_six"),
            Cube::Seven => fl!("cube_seven"),
        }
    }
    pub fn config_key(&self) -> &str {
        match self {
            Cube::Two => "record_two",
            Cube::Three => "record_three",
            Cube::Four => "record_four",
            Cube::Five => "record_five",
            Cube::Six => "record_six",
            Cube::Seven => "record_seven",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Solve {
    pub time: u64,
    pub scramble: Vec<String>,
    pub _dnf: bool,
    pub _plus_two: bool,
}
impl Solve {
    pub fn new(time: u64, scramble: &Vec<String>) -> Solve {
        Self {
            time,
            scramble: scramble.clone(),
            _dnf: false,
            _plus_two: false,
        }
    }
    pub fn time(&self) -> String {
        timer::format_from_ms(self.time)
    }
}

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub cube: Cube,
    pub solves: Vec<Solve>,
    pub best_solve: Option<Solve>,
    pub ao5: Option<u64>,
    pub ao12: Option<u64>,
    pub ao100: Option<u64>,
}
impl Record {
    pub fn default() -> Record {
        Record {
            cube: Cube::Three,
            solves: vec![],
            best_solve: None,
            ao5: None,
            ao12: None,
            ao100: None,
        }
    }
    pub fn add_solve(&mut self, solve: Solve) {
        self.solves.splice(0..0, vec![solve]);

        // Recalculate averages
        self.ao5 = calc_average(&self.solves, 5);
        self.ao12 = calc_average(&self.solves, 12);
        self.ao100 = calc_average(&self.solves, 100);
    }
}

fn calc_average(solves: &Vec<Solve>, ao: usize) -> Option<u64> {
    if solves.len() >= ao {
        let last_n: &[Solve] = &solves[0..ao];
        let mut sum = 0;
        let mut high = 0;
        let mut low = 0;
        for solve in last_n {
            if solve.time > high {
                high = solve.time;
            }
            if solve.time < low || low == 0 {
                low = solve.time;
            }
            sum += solve.time;
        }
        sum = sum - high - low;
        Some(sum / (ao as u64 - 2))
    } else {
        None
    }
}
