pub enum GameResult {
    Win,
    Lose,
    Tie,
}

pub struct GameStat {
    pub name: String,
    pub score: i32,
    pub wins: u32,
    pub loses: u32,
}

pub enum Request {
    Start {
        color: u8,
        opponent: String,
        remains: u32,
    },
    Move {
        x: u8,
        y: u8,
    },
    Pass,
    GiveUp,
    Ack {
        remains: u32,
    },
    End {
        result: GameResult,
        score: u8,
        opponent_score: u8,
        reason: String,
    },
    Bye {
        stats: Vec<GameStat>,
    },
}
