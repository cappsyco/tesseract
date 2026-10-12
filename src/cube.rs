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
            Cube::Two => "E222",
            Cube::Three => "E333",
            Cube::Four => "E444",
            Cube::Five => "E555",
            Cube::Six => "E666",
            Cube::Seven => "E777",
        }
    }
    // needed to read/write current_cube as "E333" instead of "Three"
    pub fn from_id(id: &str) -> Option<Cube> {
        match id {
            "E222" => Some(Cube::Two),
            "E333" => Some(Cube::Three),
            "E444" => Some(Cube::Four),
            "E555" => Some(Cube::Five),
            "E666" => Some(Cube::Six),
            "E777" => Some(Cube::Seven),
            _ => None,
        }
    }
}

