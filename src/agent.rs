use std::{cmp::{max, min}, collections::{HashMap, VecDeque}, future::IntoFuture, time::Duration};

use async_std::{future::timeout, sync::Mutex};
use futures::future::join_all;

use crate::{
    board::{get_valid_moves, put, Board, BoardView}, popcnt64, print_board, proto::Color, tzcnt64, write_log
};

#[inline]
pub fn evaluate(board: Board) -> i32 {
    popcnt64!(board.player)
}

pub fn alpha_beta(board: Board, player: bool, depth: u8, alpha: i32, beta: i32) -> i32 {
    let mut valid= if player {
        get_valid_moves(board.player, board.opponent)
    }
    else {
        get_valid_moves(board.opponent, board.player)
    };

    if depth == 0 || popcnt64!(valid) == 0 {
        evaluate(board)
    }
    else {
        let mut alpha = alpha;
        let mut beta = beta;

        while valid != 0 {
            let view = 1 << tzcnt64!(valid);
            valid ^= view;

            let mut board = board;
            if player {
                put(view, &mut board.player, &mut board.opponent);
            }
            else {
                put(view, &mut board.opponent, &mut board.player);
            }

            let score = alpha_beta(board, !player, depth - 1, alpha, beta);
            if player {
                alpha = max(score, alpha);
                if alpha >= beta {
                    break;
                }
            }
            else {
                beta = min(score, beta);
                if alpha >= beta {
                    break;
                }
            }
        }

        if player {
            alpha
        }
        else {
            beta
        }
    }
}

pub async fn search_move(view: BoardView, board: Board, result: &Mutex<&mut HashMap<BoardView, i32>>) {
    let mut depth = 1;

    let mut board = board;
    put(view, &mut board.player, &mut board.opponent);

    loop {
        let score = alpha_beta(board, false, depth, -1000, 1000);
        result.lock().await.insert(view, score);

        // write_log!(DEBUG, "View: {}, Depth: {}, Score: {}", view, depth, score);

        depth += 1;
        if depth >= 60 {
            break;
        }
    }
}

pub async fn select_best_move(board: Board, duration: Duration) -> Option<BoardView> {
    let mut valid = get_valid_moves(board.player, board.opponent);
    let count = popcnt64!(valid);

    if count == 0 {
        None
    } else if count == 1 {
        Some(valid)
    } else {
        // let mut possible_choices = HashMap::new();
        // let choices_mutex = Mutex::new(&mut possible_choices);

        // let mut tasks = Vec::new();
        // while valid != 0 {
        //     let view = 1 << tzcnt64!(valid);
        //     valid ^= view;

        //     tasks.push(search_move(view, board, &choices_mutex));
        // }
        // let _ = timeout(duration, join_all(tasks)).await;

        // match possible_choices.into_iter().max_by_key(|(_, v)| *v) {
        //     Some((view, _)) => Some(view),
        //     None => Some(1 << tzcnt64!(valid))
        // }

        let mut best_score = -1000;
        let mut best = 1 << tzcnt64!(valid);

        while valid != 0 {
            let view = 1 << tzcnt64!(valid);
            valid ^= view;

            let mut board = board;
            put(view, &mut board.player, &mut board.opponent);
            let score = alpha_beta(board, false, 5, -1000, 1000);
            if score > best_score {
                best_score = score;
                best = view;
            }

            write_log!(DEBUG, "Score: {}, Best: {}", score, best_score);
            print_board!(DEBUG, &board, &Color::Black);
        }
        Some(best)
    }
}
