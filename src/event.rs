

pub mod to_server {
    use game::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Event<GameEvent, GameDescription> {
        pub player: Player,
        pub payload: Payload<GameEvent, GameDescription>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Payload<GameEvent, GameDescription> {
        Auth, // some secret or something ...
        ListGames,
        NewGame(GameDescription), // requires some ... description?
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