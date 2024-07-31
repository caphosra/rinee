use crate::proto::{Color, Error, GameResult, GameStat, Request};

///
/// Parses a request from a string.
///
pub fn parse_request(req: &str) -> Result<Request, Error> {
    let mut req = req.split_whitespace().peekable();
    match req.next().ok_or(Error::Parser)? {
        "START" => {
            let color = match req.next().ok_or(Error::Parser)? {
                "BLACK" => Ok(Color::Black),
                "WHITE" => Ok(Color::White),
                _ => Err(Error::Parser),
            }?;
            let opponent = req.next().ok_or(Error::Parser)?.to_string();
            let remains = req
                .next()
                .ok_or(Error::Parser)?
                .parse()
                .map_err(|_| Error::Parser)?;
            Ok(Request::Start {
                color,
                opponent,
                remains,
            })
        }
        "MOVE" => match req.next().ok_or(Error::Parser)? {
            "PASS" => Ok(Request::Pass),
            "GIVEUP" => Ok(Request::GiveUp),
            mov => {
                let mut mov = mov.chars();
                let x = mov.next().ok_or(Error::Parser)? as u8 - 'A' as u8;
                let y = mov.next().ok_or(Error::Parser)? as u8 - '1' as u8;
                Ok(Request::Move { x, y })
            }
        },
        "ACK" => {
            let remains: i64 = req
                .next()
                .ok_or(Error::Parser)?
                .parse()
                .map_err(|_| Error::Parser)?;
            if remains < 0 {
                Ok(Request::Ack { remains: 0 })
            } else {
                Ok(Request::Ack {
                    remains: remains as u64,
                })
            }
        }
        "END" => {
            let result = match req.next().ok_or(Error::Parser)? {
                "WIN" => GameResult::Win,
                "LOSE" => GameResult::Lose,
                "TIE" => GameResult::Tie,
                _ => return Err(Error::Parser),
            };
            let score = req
                .next()
                .ok_or(Error::Parser)?
                .parse()
                .map_err(|_| Error::Parser)?;
            let opponent_score = req
                .next()
                .ok_or(Error::Parser)?
                .parse()
                .map_err(|_| Error::Parser)?;
            let reason = req.next().ok_or(Error::Parser)?.to_string();
            Ok(Request::End {
                result,
                score,
                opponent_score,
                reason,
            })
        }
        "BYE" => {
            let mut stats = Vec::new();
            while let Some(name) = req.next() {
                let score = req
                    .next()
                    .ok_or(Error::Parser)?
                    .parse()
                    .map_err(|_| Error::Parser)?;
                let wins = req
                    .next()
                    .ok_or(Error::Parser)?
                    .parse()
                    .map_err(|_| Error::Parser)?;
                let loses = req
                    .next()
                    .ok_or(Error::Parser)?
                    .parse()
                    .map_err(|_| Error::Parser)?;
                stats.push(GameStat {
                    name: name.to_string(),
                    score,
                    wins,
                    loses,
                });
            }
            Ok(Request::Bye { stats })
        }
        _ => Err(Error::Parser),
    }
}
