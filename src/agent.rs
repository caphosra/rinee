use crate::{
    board::{get_valid_moves, Board, BoardView},
    popcnt64,
    tzcnt64,
};

pub fn select_best_move(board: &Board) -> Option<BoardView> {
    let valid = get_valid_moves(board.player, board.opponent);
    let count = popcnt64!(valid);

    if count == 0 {
        None
    } else if count == 1 {
        Some(valid)
    } else {
        Some(1 << tzcnt64!(valid))
    }
}
