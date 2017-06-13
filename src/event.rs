

pub mod to_server {
    use game::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Event<GameEvent> {
        pub player: Player,
        pub payload: Payload<GameEvent>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Payload<GameEvent> {
        Auth,
        ListGames,
        NewGame,
        JoinGame(GameId),
        AbandonGame(GameId),
        GameEvent(GameId, GameEvent)
    }
}

pub mod to_client {
    use game::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Event<GameEvent, GameDetails> {
        GameListing(Vec<GameDetails>),
        GameCreated(GameDetails),
        GameJoined(GameDetails),
        GameAbandoned(GameId),
        GameEvent(GameId, GameEvent),
    }
}