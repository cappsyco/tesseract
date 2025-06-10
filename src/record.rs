use crate::timer;
use std::time::SystemTime;
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
            Cube::Two => "2x2x2".to_string(),
            Cube::Three => "3x3x3".to_string(),
            Cube::Four => "4x4x4".to_string(),
            Cube::Five => "5x5x5".to_string(),
            Cube::Six => "6x6x6".to_string(),
            Cube::Seven => "7x7x7".to_string(),
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
    pub time: u32,
    pub timestamp: Option<u64>,
    pub scramble: Vec<String>,
    pub _dnf: bool,
    pub _plus_two: bool,
}
impl Solve {
    pub fn new(time: u32, scramble: &Vec<String>) -> Solve {
        Self {
            time,
            timestamp: Some(SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()),
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
    pub ao5: Average,
    pub ao12: Average,
    pub ao100: Average,
}
impl Record {
    pub fn default() -> Record {
        Record {
            cube: Cube::Three,
            solves: vec![],
            best_solve: None,
            ao5: Average::Incomplete,
            ao12: Average::Incomplete,
            ao100: Average::Incomplete,
        }
    }
    pub fn recalc_averages(&mut self) {
        // Recalculate averages (called after adding or removing a solve)
        self.ao5   = calc_average(&self.solves, 5);
        self.ao12  = calc_average(&self.solves, 12);
        self.ao100 = calc_average(&self.solves, 100);
    }
    pub fn add_solve(&mut self, solve: Solve) {
        self.solves.splice(0..0, vec![solve]);
        self.recalc_averages();
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Average {
    Value(u32),
    DNF,
    #[default]
    Incomplete,
}

impl Average {
    pub fn to_display(&self) -> String {
        match self {
            Average::Value(t) => timer::format_from_ms(*t),
            Average::DNF => "DNF".to_string(),
            Average::Incomplete => "---".to_string(),
        }
    }
}

fn calc_average(solves: &[Solve], ao: usize) -> Average {
    // Early return: do we have enough solves?
    if solves.len() < ao {
        return Average::Incomplete;
    }
    // Take the most recent 'ao' solves
    let last_n = &solves[0..ao];
    // Map solves to times, converting DNFs to u32::MAX
    let mut times: Vec<u32> = last_n
        .iter()
        .map(|s| if s._dnf { u32::MAX } else { s.time })
        .collect();
    // Sort: real times first, DNFs last
    times.sort();
    // WCA rule: if the second-to-last time is DNF, the whole average is DNF
    // (for Ao5, this means 2 or more DNFs)
    if times[times.len() - 2] == u32::MAX {
        return Average::DNF;
    }
    // Trimmed sum: discard the best (first) and worst (last) times
    let sum: u32 = times[1..times.len() - 1].iter().sum();
    // Return the trimmed mean wrapped in the Average enum
    Average::Value(sum / (ao as u32 - 2))
}
