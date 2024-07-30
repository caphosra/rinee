use std::{
    cmp::{max, min}, collections::HashMap, sync::{
        atomic::{AtomicBool, Ordering},
        Arc, LazyLock, Mutex,
    }, thread::sleep, time::Duration
};

use crate::{
    board::{get_valid_moves, put, Board, BoardView},
    popcnt64, tzcnt64, write_log,
};

#[inline]
pub fn evaluate(board: Board) -> i32 {
    popcnt64!(board.player)
}

pub fn alpha_beta(
    interrupt: &Arc<AtomicBool>,
    board: Board,
    player: bool,
    depth: u8,
    alpha: i32,
    beta: i32,
) -> Result<i32, ()> {
    if interrupt.load(Ordering::Relaxed) {
        return Err(());
    }

    let mut valid = if player {
        get_valid_moves(board.player, board.opponent)
    } else {
        get_valid_moves(board.opponent, board.player)
    };

    if depth == 0 || popcnt64!(valid) == 0 {
        Ok(evaluate(board))
    } else {
        let mut alpha = alpha;
        let mut beta = beta;

        while valid != 0 {
            let view = 1 << tzcnt64!(valid);
            valid ^= view;

            let mut board = board;
            if player {
                put(view, &mut board.player, &mut board.opponent);
            } else {
                put(view, &mut board.opponent, &mut board.player);
            }

            let score = alpha_beta(interrupt, board, !player, depth - 1, alpha, beta)?;
            if player {
                alpha = max(score, alpha);
                if alpha >= beta {
                    break;
                }
            } else {
                beta = min(score, beta);
                if alpha >= beta {
                    break;
                }
            }
        }

        if player {
            Ok(alpha)
        } else {
            Ok(beta)
        }
    }
}

pub async fn search_move(interrupt: Arc<AtomicBool>, view: BoardView, board: Board) {
    let mut depth = 1;

    let mut board = board;
    put(view, &mut board.player, &mut board.opponent);

    while let Ok(score) = alpha_beta(&interrupt, board, false, depth, -1000, 1000) {
        CHOICES.lock().unwrap().insert(view, score);

        write_log!(DEBUG, "View{}: depth = {}, score = {}", view, depth, score);

        depth += 1;
        if depth >= 60 {
            break;
        }
    }

    write_log!(DEBUG, "View{}: Interrupted", view);
}

static CHOICES: LazyLock<Mutex<HashMap<BoardView, i32>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn select_best_move(board: Board, duration: Duration) -> Option<BoardView> {
    let mut valid = get_valid_moves(board.player, board.opponent);
    let count = popcnt64!(valid);

    if count == 0 {
        None
    } else if count == 1 {
        Some(valid)
    } else {
        {
            // Clear previous results.
            CHOICES.lock().unwrap().clear();
        }

        let interrupt = Arc::new(AtomicBool::new(false));

        let mut tasks = Vec::new();
        while valid != 0 {
            let view = 1 << tzcnt64!(valid);
            valid ^= view;

            tasks.push(tokio::spawn(search_move(interrupt.clone(), view, board)));
        }

        // Waiting for the search.
        sleep(duration);

        write_log!(DEBUG, "Flipping the interrupt flag.");
        interrupt.store(true, Ordering::Relaxed);

        for task in tasks {
            let _ = task.await;
        }
        write_log!(DEBUG, "The search was interrupted.");

        match CHOICES
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .max_by_key(|(_, v)| *v)
        {
            Some((view, _)) => Some(view),
            None => Some(1 << tzcnt64!(valid)),
        }
    }
}
