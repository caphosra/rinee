pub enum Error {
    IO(std::io::Error),
    Parser,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err)
    }
}

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

pub enum Color {
    None,
    Black,
    White,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Color::None => write!(f, "None"),
            Color::Black => write!(f, "Black"),
            Color::White => write!(f, "White"),
        }
    }
}

pub enum Request {
    Start {
        color: Color,
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
