

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameId(pub u64);

#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct Human {
    pub id: u64,
    pub name : String,
}

#[derive(Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub enum Player {
    Human(Human),
    AI, // AI ID? probably ... get's sloted in at game time
}

impl Player {
    pub fn is_human(&self, human:&Human) -> bool {
        match self {
            &Player::Human(ref player) => player == human,
            &Player::AI => false, 
        }
    }
}