use crate::scrambler::Scramble;
use crate::timer;

pub struct Solve {
    pub time: u64,
    pub scramble: Scramble,
    pub _dnf: bool,
    pub _plus_two: bool,
}
impl Solve {
    pub fn new(time: u64, scramble: &Scramble) -> Solve {
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

        // calculate AO5
        if self.solves.len() >= 5 {
            let last_five: &[Solve] = &self.solves[0..5];
            let mut sum = 0;
            for solve in last_five {
                sum += solve.time;
            }
            self.ao5 = Some(sum / 5);
        }
        // calculate AO12
        if self.solves.len() >= 12 {
            let last_twelve: &[Solve] = &self.solves[0..12];
            let mut sum = 0;
            for solve in last_twelve {
                sum += solve.time;
            }
            self.ao12 = Some(sum / 12);
        }
    }
}
