use rand::rng;
use rand::seq::IndexedRandom;
use std::fmt;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, EnumIter)]
enum Move {
    U,
    U2,
    Up,
    D,
    D2,
    Dp,
    F,
    F2,
    Fp,
    B,
    B2,
    Bp,
    L,
    L2,
    Lp,
    R,
    R2,
    Rp,
}

#[derive(Debug, Clone)]
pub struct Scramble {
    pub moves: Vec<String>,
}
impl Scramble {
    pub fn new() -> Self {
        let moves: Vec<String> = generate_scramble();

        Self { moves }
    }
    pub fn display(&self) -> String {
        self.moves.join("  ")
    }
}

fn generate_scramble() -> Vec<String> {
    let mut rng = rng();
    let mut scramble = Vec::with_capacity(20);
    let mut last_face = None;
    let all_moves: Vec<Move> = Move::iter().collect();
    let all_moves: &[Move] = &all_moves;

    // Get exactly 20 moves
    while scramble.len() < 20 {
        // get random move
        let next_move = *all_moves.choose(&mut rng).unwrap();

        // get the current face of the move
        // TODO: cleaner way to do this?
        let current_face = match next_move {
            Move::U | Move::U2 | Move::Up => 'U',
            Move::D | Move::D2 | Move::Dp => 'D',
            Move::F | Move::F2 | Move::Fp => 'F',
            Move::B | Move::B2 | Move::Bp => 'B',
            Move::L | Move::L2 | Move::Lp => 'L',
            Move::R | Move::R2 | Move::Rp => 'R',
        };

        // Add move to the scramble if it's
        // not on the same face as the last one
        if Some(current_face) != last_face {
            scramble.push(next_move.to_string().replace("p", "'"));
            last_face = Some(current_face);
        }
    }

    scramble
}
