use std::{
    cmp::{max, min},
    collections::HashMap,
    i32,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, LazyLock, Mutex,
    },
    thread::sleep,
    time::Duration,
};

use crate::{
    board::{get_confirm_stone, get_valid_moves, put, Board, BoardView},
    popcnt64, tzcnt64, write_log,
};

#[inline]
pub fn evaluate(board: Board) -> i32 {
    let pl = popcnt64!(board.player);
    let op = popcnt64!(board.opponent);
    if pl + op > 60 {
        (pl - op) * 64
    } else {
        (get_confirm_stone(board.player) - get_confirm_stone(board.opponent)) * 64
            + (popcnt64!(board.player & 0x8100000000000081)
                - popcnt64!(board.opponent & 0x8100000000000081))
                * 16
            - (popcnt64!(board.player & 0x4281000000008142)
                - popcnt64!(board.opponent & 0x4281000000008142))
                * 4
            - (popcnt64!(board.player & 0x0040000000000200)
                - popcnt64!(board.opponent & 0x0040000000000200))
                * 8
            + (popcnt64!(board.player & 0x2400810000810024)
                - popcnt64!(board.opponent & 0x2400810000810024))
                * 2
            + (popcnt64!(board.player & 0x1800248181240018)
                - popcnt64!(board.opponent & 0x1800248181240018))
            + (popcnt64!(board.player & 0x0000182424180000)
                - popcnt64!(board.opponent & 0x0000182424180000))
            - (popcnt64!(board.player & 0x003c424242423c00)
                - popcnt64!(board.opponent & 0x003c424242423c00))
    }
}

const INF: i32 = i32::MAX - 100;

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

    // When there is no valid move.
    if popcnt64!(valid) == 0 {
        let valid = if player {
            get_valid_moves(board.opponent, board.player)
        } else {
            get_valid_moves(board.player, board.opponent)
        };
        // Is the game over?
        if popcnt64!(valid) == 0 {
            let player_num = popcnt64!(board.player);
            let opponent_num = popcnt64!(board.opponent);
            if player_num > opponent_num {
                return Ok(INF);
            } else if player_num < opponent_num {
                return Ok(-INF);
            } else {
                return Ok(0);
            }
        } else {
            return alpha_beta(interrupt, board, !player, depth, alpha, beta);
        }
    }

    if depth == 0 {
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
    let mut depth = 5;

    let mut board = board;
    put(view, &mut board.player, &mut board.opponent);

    while let Ok(score) = alpha_beta(&interrupt, board, false, depth, i32::MIN, i32::MAX) {
        CHOICES.lock().unwrap().insert(view, score);

        write_log!(DEBUG, "View{}: depth = {}, score = {}", view, depth, score);

        if score == INF || score == -INF {
            break;
        }

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
    let valid = get_valid_moves(board.player, board.opponent);
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
        let mut counter = valid;
        while counter != 0 {
            let view = 1 << tzcnt64!(counter);
            counter ^= view;

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
