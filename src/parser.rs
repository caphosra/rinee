use crate::proto::{GameResult, GameStat, Request};

pub fn parse_request(req: &str) -> Result<Request, ()> {
    let mut req = req.split_whitespace().peekable();
    match req.next().ok_or(())? {
        "START" => {
            let color = req.next().ok_or(())?.parse().map_err(|_| ())?;
            let opponent = req.next().ok_or(())?.to_string();
            let remains = req.next().ok_or(())?.parse().map_err(|_| ())?;
            Ok(Request::Start {
                color,
                opponent,
                remains,
            })
        }
        "MOVE" => match req.next().ok_or(())? {
            "PASS" => Ok(Request::Pass),
            "GIVEUP" => Ok(Request::GiveUp),
            mov => {
                let mut mov = mov.chars();
                let x = mov.next().ok_or(())? as u8 - 'A' as u8;
                let y = mov.next().ok_or(())? as u8 - '1' as u8;
                Ok(Request::Move { x, y })
            }
        },
        "ACK" => {
            let remains = req.next().ok_or(())?.parse().map_err(|_| ())?;
            Ok(Request::Ack { remains })
        }
        "END" => {
            let result = match req.next().ok_or(())? {
                "WIN" => GameResult::Win,
                "LOSE" => GameResult::Lose,
                "TIE" => GameResult::Tie,
                _ => return Err(()),
            };
            let score = req.next().ok_or(())?.parse().map_err(|_| ())?;
            let opponent_score = req.next().ok_or(())?.parse().map_err(|_| ())?;
            let reason = req.next().ok_or(())?.to_string();
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
                let score = req.next().ok_or(())?.parse().map_err(|_| ())?;
                let wins = req.next().ok_or(())?.parse().map_err(|_| ())?;
                let loses = req.next().ok_or(())?.parse().map_err(|_| ())?;
                stats.push(GameStat {
                    name: name.to_string(),
                    score,
                    wins,
                    loses,
                });
            }
            Ok(Request::Bye { stats })
        }
        _ => Err(()),
    }
}
