struct Solve {
    time: u64,
    sramble: Scramble,
    dnf: bool,
    plus_two: bool,
}

struct Record {
    solves: Vec<Solve>,
    ao5: u64,
    ao12: u64,
}
