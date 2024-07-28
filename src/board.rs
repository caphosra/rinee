//
// Basic operations on bit boards.
//
// References:
//  https://qiita.com/sensuikan1973/items/459b3e11d91f3cb37e43 (Swift)
//

pub type BoardView = u64;

pub struct Board {
    pub black: BoardView,
    pub white: BoardView,
}

pub fn new_board() -> Board {
    Board {
        black: 0x0000000810000000,
        white: 0x0000001008000000,
    }
}

///
/// Get valid moves from the board views.
///
pub fn get_valid_moves(me: BoardView, opponent: BoardView) -> BoardView {
    let horizontal = opponent & 0x7e7e7e7e7e7e7e7e;
    let vertical = opponent & 0x00FFFFFFFFFFFF00;
    let diagonal = opponent & 0x007e7e7e7e7e7e00;

    let blank = !(me | opponent);

    // Left
    let mut tmp = horizontal & (me << 1);
    tmp |= horizontal & (tmp << 1);
    tmp |= horizontal & (tmp << 1);
    tmp |= horizontal & (tmp << 1);
    tmp |= horizontal & (tmp << 1);
    tmp |= horizontal & (tmp << 1);
    let mut valid = blank & (tmp << 1);

    // Right
    tmp = horizontal & (me >> 1);
    tmp |= horizontal & (tmp >> 1);
    tmp |= horizontal & (tmp >> 1);
    tmp |= horizontal & (tmp >> 1);
    tmp |= horizontal & (tmp >> 1);
    tmp |= horizontal & (tmp >> 1);
    valid |= blank & (tmp >> 1);

    // Up
    tmp = vertical & (me << 8);
    tmp |= vertical & (tmp << 8);
    tmp |= vertical & (tmp << 8);
    tmp |= vertical & (tmp << 8);
    tmp |= vertical & (tmp << 8);
    tmp |= vertical & (tmp << 8);
    valid |= blank & (tmp << 8);

    // Down
    tmp = vertical & (me >> 8);
    tmp |= vertical & (tmp >> 8);
    tmp |= vertical & (tmp >> 8);
    tmp |= vertical & (tmp >> 8);
    tmp |= vertical & (tmp >> 8);
    tmp |= vertical & (tmp >> 8);
    valid |= blank & (tmp >> 8);

    // Right up
    tmp = diagonal & (me << 7);
    tmp |= diagonal & (tmp << 7);
    tmp |= diagonal & (tmp << 7);
    tmp |= diagonal & (tmp << 7);
    tmp |= diagonal & (tmp << 7);
    tmp |= diagonal & (tmp << 7);
    valid |= blank & (tmp << 7);

    // Left up
    tmp = diagonal & (me << 9);
    tmp |= diagonal & (tmp << 9);
    tmp |= diagonal & (tmp << 9);
    tmp |= diagonal & (tmp << 9);
    tmp |= diagonal & (tmp << 9);
    tmp |= diagonal & (tmp << 9);
    valid |= blank & (tmp << 9);

    // Right down
    tmp = diagonal & (me >> 9);
    tmp |= diagonal & (tmp >> 9);
    tmp |= diagonal & (tmp >> 9);
    tmp |= diagonal & (tmp >> 9);
    tmp |= diagonal & (tmp >> 9);
    tmp |= diagonal & (tmp >> 9);
    valid |= blank & (tmp >> 9);

    // Left down
    tmp = diagonal & (me >> 7);
    tmp |= diagonal & (tmp >> 7);
    tmp |= diagonal & (tmp >> 7);
    tmp |= diagonal & (tmp >> 7);
    tmp |= diagonal & (tmp >> 7);
    tmp |= diagonal & (tmp >> 7);
    valid |= blank & (tmp >> 7);

    return valid
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
    fn to_string_as_board(&self) -> String;
}

#[cfg(debug_assertions)]
impl DebugBoard for Board {
    fn to_string_as_board(&self) -> String {
        let mut board = String::new();
        for y in 0..8 {
            for x in 0..8 {
                let idx = y * 8 + x;
                if self.black & (1 << idx) != 0 {
                    board.push('B');
                } else if self.white & (1 << idx) != 0 {
                    board.push('W');
                } else {
                    board.push(' ');
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
    ($board:expr) => {{
        use crate::board::DebugBoard;
        crate::write_log!(LOG, "{}", $board.to_string_as_board());
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! print_board {
    ($board:expr) => {};
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_board() {
        let board = new_board();
        assert_eq!(board.to_string_as_board(),
            concat!(
                "        \n",
                "        \n",
                "        \n",
                "   WB   \n",
                "   BW   \n",
                "        \n",
                "        \n",
                "        "
            )
        );
    }

    #[test]
    fn test_get_valid_moves() {
        let board = new_board();
        let moves = get_valid_moves(board.black, board.white);
        assert_eq!(moves, 1 << (3 + 2 * 8) | 1 << (2 + 3 * 8) | 1 << (5 + 4 * 8) | 1 << (4 + 5 * 8));
    }

    #[test]
    fn test_put() {
        let mut board = new_board();

        put(1 << (4 + 5 * 8), &mut board.black, &mut board.white);
        assert_eq!(board.to_string_as_board(),
            concat!(
                "        \n",
                "        \n",
                "        \n",
                "   WB   \n",
                "   BB   \n",
                "    B   \n",
                "        \n",
                "        "
            )
        );

        put(1 << (3 + 5 * 8), &mut board.white, &mut board.black);
        assert_eq!(board.to_string_as_board(),
            concat!(
                "        \n",
                "        \n",
                "        \n",
                "   WB   \n",
                "   WB   \n",
                "   WB   \n",
                "        \n",
                "        "
            )
        );

        put(1 << (2 + 3 * 8), &mut board.black, &mut board.white);
        assert_eq!(board.to_string_as_board(),
            concat!(
                "        \n",
                "        \n",
                "        \n",
                "  BBB   \n",
                "   BB   \n",
                "   WB   \n",
                "        \n",
                "        "
            )
        );
    }
}
