

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameId(u64);

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