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
    pub ao100: Option<u64>,
}
impl Record {
    pub fn default() -> Record {
        Record {
            solves: vec![],
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
