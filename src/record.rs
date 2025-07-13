use crate::scrambler::Scramble;

pub struct Solve {
    pub time: u64,
    pub scramble: Scramble,
    pub dnf: bool,
    pub plus_two: bool,
}
impl Solve {
    pub fn new(time: u64, scramble: &Scramble) -> Solve {
        Self {
            time,
            scramble: scramble.clone(),
            dnf: false,
            plus_two: false,
        }
    }
}

pub struct Record {
    pub solves: Vec<Solve>,
    pub ao5: Option<u64>,
    pub ao12: Option<u64>,
}
impl Record {
    pub fn default() -> Record {
        Record {
            solves: vec![],
            ao5: None,
            ao12: None,
        }
    }
    pub fn add_solve(&mut self, solve: Solve) {
        self.solves.splice(0..0, vec![solve]);
    }
}
