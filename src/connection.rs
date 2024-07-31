use std::{
    collections::HashMap,
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
    time::Duration,
};

use crate::{
    agent::select_best_move,
    board::{from_pos, get_pos, new_board, put, Board},
    parser::parse_request,
    popcnt64, print_board,
    proto::{Color, Error, Request},
    write_log, Args,
};

pub async fn do_move(
    board: &mut Board,
    me: &Color,
    remains: u64,
    writer: &mut BufWriter<&TcpStream>,
    history: &mut String,
) -> Result<(), Error> {
    let usable = if remains < 8000 {
        let space = 64 - (popcnt64!(board.player) + popcnt64!(board.opponent));
        if space == 0 {
            remains
        } else {
            (remains / space as u64) << 1
        }
    } else {
        2200
    };

    match select_best_move(*board, Duration::from_millis(usable)).await {
        Some(view) => {
            put(view, &mut board.player, &mut board.opponent);
            let (x, y) = from_pos(view);
            writer.write(format!("MOVE {}{}\n", (b'A' + x) as char, y + 1).as_bytes())?;
            *history += &format!("{}{}", (b'A' + x) as char, y + 1);

            write_log!(LOG, "ME {}{}", (b'A' + x) as char, y + 1);
            print_board!(LOG, board, &me);
        }
        None => {
            writer.write(b"MOVE PASS\n")?;
        }
    }
    writer.flush()?;
    Ok(())
}

static PREPROCESSED_FILE: &str = "./preprocessed.txt";

pub fn load_preprocessed() -> Result<HashMap<String, String>, Error> {
    let text = std::fs::read_to_string(PREPROCESSED_FILE)?;

    let mut table = HashMap::new();
    for line in text.lines() {
        let mut iter = line.split_whitespace();
        let key = iter.next().ok_or(Error::Parser)?;
        let value = iter.next().ok_or(Error::Parser)?;
        table.insert(key.to_string(), value.to_string());
    }

    Ok(table)
}

pub async fn play_game(args: &Args) -> Result<(), Error> {
    write_log!(DEBUG, "Loading a preprocessed table.");
    let table = load_preprocessed()?;

    let addr = format!("{}:{}", args.host, args.port);
    let stream = TcpStream::connect(addr)?;
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    write_log!(DEBUG, "Connected to the server.");

    let mut board = new_board(&Color::Black);
    let mut me = Color::Black;

    writer.write(format!("OPEN {}\n", args.name).as_bytes())?;
    writer.flush()?;

    write_log!(DEBUG, "Sent OPEN");

    let mut time_remains = 0;
    let mut history = String::new();

    loop {
        let mut buf = String::new();
        reader.read_line(&mut buf)?;

        let req = parse_request(&buf).map_err(|_| Error::ParserWithMessage(buf))?;

        match req {
            Request::Start {
                color,
                opponent,
                remains,
            } => {
                write_log!(
                    LOG,
                    "Start color={}, opponent={}, remains={}",
                    color,
                    opponent,
                    remains
                );

                me = color;
                board = new_board(&me);
                time_remains = remains;
                history = String::new();

                match &me {
                    Color::Black => {
                        // Fix the first move.
                        put(get_pos(2, 3), &mut board.player, &mut board.opponent);
                        writer.write("MOVE C4\n".as_bytes())?;
                        writer.flush()?;
                    }
                    _ => {}
                }
            }
            Request::Move { x, y } => {
                put(get_pos(x, y), &mut board.opponent, &mut board.player);
                history += &format!("{}{}", (b'A' + x) as char, y + 1);

                write_log!(LOG, "OPPONENT {}{}", (b'A' + x) as char, y + 1);
                print_board!(LOG, board, &me);

                if let Some(best_move) = table.get(&history) {
                    write_log!(DEBUG, "Preprocessed move: {}", best_move);

                    writer.write(format!("MOVE {}\n", best_move).as_bytes())?;
                    writer.flush()?;

                    write_log!(LOG, "ME {}", best_move);
                    print_board!(LOG, board, &me);

                    let mut best_move = best_move.chars();
                    let x = best_move.next().ok_or(Error::Parser)? as u8 - 'A' as u8;
                    let y = best_move.next().ok_or(Error::Parser)? as u8 - '1' as u8;
                    put(get_pos(x, y), &mut board.player, &mut board.opponent);
                } else {
                    do_move(&mut board, &me, time_remains, &mut writer, &mut history).await?;
                }
            }
            Request::Pass => {
                write_log!(LOG, "OPPONENT PASS");

                do_move(&mut board, &me, time_remains, &mut writer, &mut history).await?;
            }
            Request::GiveUp => {
                write_log!(LOG, "OPPONENT GIVEUP");
            }
            Request::End {
                result,
                score,
                opponent_score,
                reason,
            } => {
                write_log!(LOG, "The game ends.");
                write_log!(LOG, "- result: {}", result);
                write_log!(LOG, "- score me/opponent: {}/{}", score, opponent_score);
                write_log!(LOG, "- reason: {}", reason);
            }
            Request::Bye { stats } => {
                for stat in stats {
                    write_log!(LOG, "The stat of {}", stat.name);
                    write_log!(LOG, "- score: {}", stat.score);
                    write_log!(LOG, "- win/lose: {}/{}", stat.wins, stat.loses);
                }
                break;
            }
            Request::Ack { remains } => {
                write_log!(DEBUG, "Time remains: {}", remains);
                time_remains = remains;
            }
        }
    }

    Ok(())
}
