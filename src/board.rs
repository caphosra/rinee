//
// Basic operations on bit boards.
//
// References:
//  https://qiita.com/sensuikan1973/items/459b3e11d91f3cb37e43 (Swift)
//

use crate::{popcnt64, proto::Color, tzcnt64};

pub type BoardView = u64;

#[derive(Clone, Copy)]
pub struct Board {
    pub player: BoardView,
    pub opponent: BoardView,
}

pub fn new_board(me: &Color) -> Board {
    match me {
        Color::Black => Board {
            player: 0x0000000810000000,
            opponent: 0x0000001008000000,
        },
        Color::White => Board {
            player: 0x0000001008000000,
            opponent: 0x0000000810000000,
        },
    }
}

#[inline]
pub fn get_pos(x: u8, y: u8) -> BoardView {
    1 << (x | (y << 3))
}

#[inline]
pub fn from_pos(view: BoardView) -> (u8, u8) {
    let v = tzcnt64!(view);
    ((v & 0b111) as u8, (v >> 3) as u8)
}

pub fn get_confirm_stone(me: BoardView) -> i32 {
    macro_rules! get_confirm_stone_internal {
        ($victim:expr, $shift:tt, $shift_num:expr, $mask:expr) => {
            let mut mask = me;
            for _ in 0..7 {
                mask = (mask $shift $shift_num) | $mask;
                $victim &= mask;
            }
        };
    }

    // Left up
    let mut left_up = me;
    get_confirm_stone_internal!(left_up, <<, 9, 0x01010101010101FF);
    get_confirm_stone_internal!(left_up, <<, 8, 0x00000000000000FF);
    get_confirm_stone_internal!(left_up, <<, 1, 0x0101010101010101);

    let mut left_up1 = left_up;
    get_confirm_stone_internal!(left_up1, <<, 7, 0x80808080808080FF);
    let mut left_up2 = left_up;
    get_confirm_stone_internal!(left_up2, >>, 7, 0xFF01010101010101);
    left_up &= left_up1 | left_up2;

    // Right up
    let mut right_up = me;
    get_confirm_stone_internal!(right_up, <<, 8, 0x00000000000000FF);
    get_confirm_stone_internal!(right_up, <<, 7, 0x80808080808080FF);
    get_confirm_stone_internal!(right_up, >>, 1, 0x8080808080808080);

    let mut right_up1 = right_up;
    get_confirm_stone_internal!(right_up1, <<, 9, 0x01010101010101FF);
    let mut right_up2 = right_up;
    get_confirm_stone_internal!(right_up2, >>, 9, 0xFF80808080808080);
    right_up &= right_up1 | right_up2;

    // Left down
    let mut left_down = me;
    get_confirm_stone_internal!(left_down, <<, 1, 0x0101010101010101);
    get_confirm_stone_internal!(left_down, >>, 7, 0xFF01010101010101);
    get_confirm_stone_internal!(left_down, >>, 8, 0xFF00000000000000);

    let mut left_down1 = left_down;
    get_confirm_stone_internal!(left_down1, <<, 9, 0x01010101010101FF);
    let mut left_down2 = left_down;
    get_confirm_stone_internal!(left_down2, >>, 9, 0xFF80808080808080);
    left_down &= left_down1 | left_down2;

    // Right down
    let mut right_down = me;
    get_confirm_stone_internal!(right_down, >>, 1, 0x8080808080808080);
    get_confirm_stone_internal!(right_down, >>, 8, 0xFF00000000000000);
    get_confirm_stone_internal!(right_down, >>, 9, 0xFF80808080808080);

    let mut right_down1 = right_down;
    get_confirm_stone_internal!(right_down1, <<, 7, 0x80808080808080FF);
    let mut right_down2 = right_down;
    get_confirm_stone_internal!(right_down2, >>, 7, 0xFF01010101010101);
    right_down &= right_down1 | right_down2;

    let mask = left_up | right_up | left_down | right_down;
    popcnt64!(me & mask)
}

///
/// Get valid moves from the board views.
///
pub fn get_valid_moves(me: BoardView, opponent: BoardView) -> BoardView {
    let horizontal = opponent & 0x7e7e7e7e7e7e7e7e;
    let vertical = opponent & 0x00FFFFFFFFFFFF00;
    let diagonal = opponent & 0x007e7e7e7e7e7e00;

    let blank = !(me | opponent);
    let mut valid = 0;

    macro_rules! get_valid_move {
        ($mask:expr, $shift:tt, $shift_num:expr) => {
            let mut tmp = $mask & (me $shift $shift_num);
            tmp |= $mask & (tmp $shift $shift_num);
            tmp |= $mask & (tmp $shift $shift_num);
            tmp |= $mask & (tmp $shift $shift_num);
            tmp |= $mask & (tmp $shift $shift_num);
            tmp |= $mask & (tmp $shift $shift_num);
            valid |= blank & (tmp $shift $shift_num);
        };
    }

    // Left
    get_valid_move!(horizontal, <<, 1);
    // Right
    get_valid_move!(horizontal, >>, 1);
    // Up
    get_valid_move!(vertical, <<, 8);
    // Down
    get_valid_move!(vertical, >>, 8);
    // Right up
    get_valid_move!(diagonal, <<, 7);
    // Left up
    get_valid_move!(diagonal, <<, 9);
    // Right down
    get_valid_move!(diagonal, >>, 9);
    // Left down
    get_valid_move!(diagonal, >>, 7);

    valid
}

pub fn put(pos: BoardView, player: &mut BoardView, opponent: &mut BoardView) {
    let mut result: BoardView = 0;

    macro_rules! put_internal {
        ($shift:tt, $shift_num:expr, $mask:expr) => {
            let mut tmp: BoardView = 0;
            let mut mask =  (pos $shift $shift_num) & ($mask);
            while (mask != 0) && ((mask & *opponent) != 0) {
                tmp |= mask;
                mask = (mask $shift $shift_num) & ($mask);
            }
            if (mask & *player) != 0 {
                result |= tmp
            }
        };
    }

    // Left
    put_internal!(<<, 1, 0xfefefefefefefefe);
    // Right
    put_internal!(>>, 1, 0x7f7f7f7f7f7f7f7f);
    // Up
    put_internal!(<<, 8, 0xffffffffffffff00);
    // Down
    put_internal!(>>, 8, 0x00ffffffffffffff);
    // Right up
    put_internal!(<<, 7, 0x7f7f7f7f7f7f7f00);
    // Left up
    put_internal!(<<, 9, 0xfefefefefefefe00);
    // Right down
    put_internal!(>>, 9, 0x007f7f7f7f7f7f7f);
    // Left down
    put_internal!(>>, 7, 0x00fefefefefefefe);

    *player ^= pos | result;
    *opponent ^= result;
}

#[cfg(debug_assertions)]
pub trait DebugBoard {
    fn to_string_as_board(&self, me: &Color) -> String;
}

#[cfg(debug_assertions)]
impl DebugBoard for Board {
    fn to_string_as_board(&self, me: &Color) -> String {
        let valid = get_valid_moves(self.player, self.opponent);
        let mut board = String::new();
        for y in 0..8 {
            for x in 0..8 {
                let pos = get_pos(x, y);
                if valid & pos != 0 {
                    assert_eq!(self.player & pos, 0);
                    assert_eq!(self.opponent & pos, 0);
                    board.push('+');
                } else if self.player & pos != 0 {
                    match me {
                        Color::Black => board.push('B'),
                        Color::White => board.push('W'),
                    }
                } else if self.opponent & pos != 0 {
                    match me {
                        Color::Black => board.push('W'),
                        Color::White => board.push('B'),
                    }
                } else {
                    board.push('-');
                }
            }
            if y != 7 {
                board.push('\n');
            }
        }
        board
    }
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! print_board {
    ($level:tt, $board:expr, $color:expr) => {{
        use crate::board::DebugBoard;
        crate::write_log!($level, "{}", $board.to_string_as_board($color));
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! print_board {
    ($level:tt, $board:expr, $color:expr) => {};
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_pos() {
        assert_eq!(from_pos(get_pos(5, 2)), (5, 2));
    }

    #[test]
    fn test_new_board() {
        let board = new_board(&Color::Black);
        assert_eq!(
            board.to_string_as_board(&Color::Black),
            concat!(
                "--------\n",
                "--------\n",
                "---+----\n",
                "--+WB---\n",
                "---BW+--\n",
                "----+---\n",
                "--------\n",
                "--------"
            )
        );
    }

    #[test]
    fn test_get_valid_moves() {
        let board = new_board(&Color::Black);
        let moves = get_valid_moves(board.player, board.opponent);
        assert_eq!(
            moves,
            get_pos(3, 2) | get_pos(2, 3) | get_pos(5, 4) | get_pos(4, 5)
        );
    }

    #[test]
    fn test_get_confirm_stone() {
        assert_eq!(get_confirm_stone(0x07010F), 6);
        assert_eq!(get_confirm_stone(0x070707), 6);
    }

    #[test]
    fn test_put() {
        let mut board = new_board(&Color::Black);

        put(get_pos(4, 5), &mut board.player, &mut board.opponent);
        assert_eq!(
            board.to_string_as_board(&Color::Black),
            concat!(
                "--------\n",
                "--------\n",
                "--++----\n",
                "--+WB---\n",
                "---BB---\n",
                "----B---\n",
                "--------\n",
                "--------"
            )
        );

        put(get_pos(3, 5), &mut board.opponent, &mut board.player);
        assert_eq!(
            board.to_string_as_board(&Color::Black),
            concat!(
                "--------\n",
                "--------\n",
                "--+-----\n",
                "--+WB---\n",
                "--+WB---\n",
                "--+WB---\n",
                "--+-----\n",
                "--------"
            )
        );

        put(get_pos(2, 3), &mut board.player, &mut board.opponent);
        assert_eq!(
            board.to_string_as_board(&Color::Black),
            concat!(
                "--------\n",
                "--------\n",
                "--------\n",
                "--BBB---\n",
                "---BB---\n",
                "--+WB---\n",
                "--++----\n",
                "--------"
            )
        );
    }
}
