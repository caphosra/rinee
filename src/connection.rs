use async_std::{
    io::{prelude::BufReadExt, BufReader, BufWriter, WriteExt},
    net::TcpStream,
};

use crate::{
    agent::select_best_move,
    board::{from_pos, get_pos, new_board, put, Board},
    parser::parse_request,
    print_board,
    proto::{Color, Error, Request},
    write_log, Args,
};

pub async fn do_move(
    board: &mut Board,
    me: &Color,
    writer: &mut BufWriter<&TcpStream>,
) -> Result<(), Error> {
    match select_best_move(&board) {
        Some(view) => {
            put(view, &mut board.player, &mut board.opponent);
            let (x, y) = from_pos(view);
            writer
                .write(format!("MOVE {}{}\n", (b'A' + x) as char, y + 1).as_bytes())
                .await?;

            write_log!(LOG, "ME {}{}", (b'A' + x) as char, y + 1);
            print_board!(board, &me);
        }
        None => {
            writer.write(b"MOVE PASS\n").await?;
        }
    }
    writer.flush().await?;
    Ok(())
}

pub async fn play_game(args: &Args) -> Result<(), Error> {
    let addr = format!("{}:{}", args.host, args.port);
    let stream = TcpStream::connect(addr).await?;
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    write_log!(DEBUG, "Connected to the server.");

    let mut board = new_board(&Color::Black);
    let mut me = Color::Black;

    writer
        .write(format!("OPEN {}\n", args.name).as_bytes())
        .await?;
    writer.flush().await?;

    write_log!(DEBUG, "Sent OPEN");

    loop {
        let mut buf = String::new();
        reader.read_line(&mut buf).await?;

        let req = parse_request(&buf)?;

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

                match &me {
                    Color::Black => {
                        do_move(&mut board, &me, &mut writer).await?;
                    }
                    _ => {}
                }
            }
            Request::Move { x, y } => {
                put(get_pos(x, y), &mut board.opponent, &mut board.player);
                write_log!(LOG, "OPPONENT {}{}", (b'A' + x) as char, y + 1);
                print_board!(board, &me);

                do_move(&mut board, &me, &mut writer).await?;
            }
            Request::Pass => {
                write_log!(LOG, "OPPONENT PASS");

                do_move(&mut board, &me, &mut writer).await?;
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
            }
        }
    }

    Ok(())
}
