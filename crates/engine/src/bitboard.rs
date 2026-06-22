use std::fmt;
use crate::constants::{BOARD_HEIGHT, BOARD_WIDTH, COL_STRIDE};

/// Bitboard representation for a Connect Four board for a single player.
///
/// Internally, the board is stored as a 64‑bit integer, where each bit
/// corresponds to one cell on the board. The layout is column‑major:
///
/// ```text
///  6 13 20 27 34 41 48
///  5 12 19 26 33 40 47
///  4 11 18 25 32 39 46
///  3 10 17 24 31 38 45
///  2  9 16 23 30 37 44
///  1  8 15 22 29 36 43
///  0  7 14 21 28 35 42
/// ```
///
/// Each `BitBoard` stores the pieces of **one** player. To represent the full
/// game state, there is one `BitBoard` for each player (e.g. red and white).
///
/// # Example
///
/// ```ignore
/// # use engine::bitboard::BitBoard;
/// # const BOARD_WIDTH: u8 = 7;
/// # const BOARD_HEIGHT: u8 = 6;
/// let mut bb = BitBoard::empty();
/// // Set a few bits and check for a win:
/// bb.with_bit_set(BitBoard::bit_index(0, 0)); // bottom-left
/// bb.with_bit_set(BitBoard::bit_index(1, 0));
/// bb.with_bit_set(BitBoard::bit_index(2, 0));
/// bb.with_bit_set(BitBoard::bit_index(3, 0));
/// assert!(bb.has_won());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitBoard {
    bits: u64,
}

impl BitBoard {
    /// Returns an empty bitboard with no bits set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use engine::bitboard::BitBoard;
    /// let bb = BitBoard::empty();
    /// assert_eq!(bb.bits(), 0);
    /// ```
    pub fn empty() -> Self {
        BitBoard {
            bits: 0u64
        }
    }

    /// Checks whether this bitboard contains any four‑in‑a‑row pattern.
    ///
    /// This uses a common bitboard trick: for each direction (vertical,
    /// horizontal, and the two diagonals), the board is shifted and ANDed
    /// with itself to detect contiguous runs. If there exists at least one
    /// sequence of four aligned bits for this player, this function returns
    /// `true`.
    ///
    /// Directions are encoded as bit shifts:
    ///
    /// - `1`  → vertical (same column, row + 1)
    /// - `7`  → horizontal (next column, same row)
    /// - `6`  → diagonal `\` (col+1, row+1)
    /// - `8`  → diagonal `/` (col+1, row-1)
    ///
    pub fn has_won(&self) -> bool {
        // "vertical", "horizontal", "diagonal top left - down right (\)", "diagonal bottom left - top right (/)"
        let directions = [1, 7, 6, 8];

        for shift in directions {
            let pairs = self.bits & (self.bits >> shift);

            if (pairs & (pairs >> (2 * shift))) != 0 {
                return true;
            }
        }
        false
    }

    /// The value of the bitboard
    pub fn bits(&self) -> u64 {
        self.bits
    }

    /// Sets the bit at the given bit index.
    ///
    /// This is a low‑level internal helper; callers usually use
    /// [`bit_index`] to compute the index from `(col, row)`.
    pub(crate) fn with_bit_set(&mut self, bit: u8) {
	    self.bits = self.bits | (1 << bit);
    }

    /// Checks if the bit at column, row is set
    #[inline]
    pub(crate) fn bit_at(&self, col: u8, row: u8) -> bool {
        let idx = Self::bit_index(col, row);
        (self.bits & (1u64 << idx)) != 0
    }

    pub fn bit_index(col: u8, row: u8) -> u8 {
        debug_assert!(col < BOARD_WIDTH);
        debug_assert!(row < BOARD_HEIGHT);
        col * COL_STRIDE + row
    }
}

impl fmt::Debug for BitBoard {
    /// Formats the bitboard as an ASCII Connect Four grid.
    ///
    /// Example output:
    ///
    /// ```text
    /// +-------+
    /// |.......|
    /// |.......|
    /// |.......|
    /// |.......|
    /// |.......|
    /// |XXX....|
    /// +-------+
    ///  0123456
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // top
        writeln!(f, "+{}+", "-".repeat(BOARD_WIDTH as usize))?;

        // rows:  from top (BOARD_HEIGHT-1) down to 0
        for row in (0..BOARD_HEIGHT).rev() {
            write!(f, "|")?;
            for col in 0..BOARD_WIDTH {
                let occupied = self.bit_at(col, row);
                let ch = if occupied { 'X' } else { '.' };
                write!(f, "{ch}")?;
            }
            writeln!(f, "|")?;
        }

        // bottom
        writeln!(f, "+{}+", "-".repeat(BOARD_WIDTH as usize))?;

        // indices
        write!(f, " ")?;
        for col in 0..BOARD_WIDTH {
            write!(f, "{}", col)?;
        }
        writeln!(f)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_board() {
        let bitboard = BitBoard::empty();

        assert_eq!(bitboard.bits(), 0u64, "Empty bitboard should not have any bits set.");
        assert_eq!(bitboard.has_won(), false, "Empty bitboard can't have a win state.");
    }

    #[test]
    fn test_change_board_state_with_allowed_value() {
        let mut bitboard = BitBoard::empty();
        let rows = BOARD_HEIGHT;
        let cols = BOARD_WIDTH;
        let mut expected_value = 0u64;
        for col in 0..cols {
            for row in 0..rows {
                let bit_to_set = col * BOARD_WIDTH + row;
                bitboard.with_bit_set(bit_to_set);
                expected_value = expected_value | (1 << bit_to_set);
                assert_eq!(bitboard.bits(), expected_value, "Board state doesn't match expected value");
            }
        }
        assert_eq!(bitboard.bits(), 279258638311359u64, "Board state should be 279258638311359u64");
    }


    #[test]
    fn test_win_with_vertical_four_connected() {
        let mut bitboard = BitBoard::empty();
        // tokens in column 0 connected with rows 0,1,2,3 forming a win state
        // token coordinates in format [colum, row]
        let tokens: [[u8; 2]; 4] = [[0, 0], [0, 1], [0, 2], [0, 3]];
        set_token_positions(&mut bitboard, tokens);
        assert!(bitboard.has_won(), "Board should have won state");
        assert_eq!(bitboard.bits(),  15, "Board state should be 15");
    }

    fn set_token_positions(bitboard: &mut BitBoard, tokens: [[u8; 2]; 4]) {
        tokens.iter().for_each(|&coord| {
            let bit_to_set = BitBoard::bit_index(coord[0], coord[1]);
            bitboard.with_bit_set(bit_to_set);
        });
    }

    #[test]
    fn test_no_win_with_vertical_four_disconnected() {
        let mut bitboard = BitBoard::empty();
        // tokens in column 0 disconnected with rows 2 and 3 being empty, not forming a win state
        let tokens: [[u8; 2]; 4] = [[0, 0], [0, 1], [0, 4], [0, 5]];
        set_token_positions(&mut bitboard, tokens);
        assert_eq!(bitboard.has_won(), false, "Board should have not have a won state");
        assert_eq!(bitboard.bits(),  51, "Board state should be 51");
    }

    #[test]
    fn test_win_with_horizontal_four_connected() {
        let mut bitboard = BitBoard::empty();
        // tokens in row 3 connected with columns 1,2,3,4 forming a win state
        let tokens: [[u8; 2]; 4] = [[1, 3], [2, 3], [3, 3], [4, 3]];
        set_token_positions(&mut bitboard, tokens);
        let expected_value = (1 << 10) + (1 << 17) + (1 << 24) + (1 << 31);
        assert!(bitboard.has_won(), "Board should have won state");
        assert_eq!(bitboard.bits(), expected_value, "Board state should be {}", expected_value);
    }

    #[test]
    fn test_no_win_with_horizontal_four_disconnected() {
        let mut bitboard = BitBoard::empty();
        // tokens in row 3 disconnected with row 3 being empty, not forming a win state
        let tokens: [[u8; 2]; 4] = [[1, 3], [2, 3], [4, 3], [5, 3]];
        set_token_positions(&mut bitboard, tokens);
        let expected_value = (1 << 10) + (1 << 17) + (1 << 31) + (1 << 38);
        assert_eq!(bitboard.has_won(), false, "Board should have not have a won state");
        assert_eq!(bitboard.bits(),  expected_value, "Board state should be {}", expected_value);
    }

    #[test]
    fn test_win_with_diagonal_four_connected() {
        let mut bitboard = BitBoard::empty();
        // tokens starting from row 1, col 1 form a diagonal connect with pairs [2,2], [3,3] and [4,4]
        // forming a win state
        let tokens: [[u8; 2]; 4] = [[1, 1], [2, 2], [3, 3], [4, 4]];
        set_token_positions(&mut bitboard, tokens);
        let expected_value = (1 << 8) + (1 << 16) + (1 << 24) + (1 << 32);
        assert!(bitboard.has_won(), "Board should have won state");
        assert_eq!(bitboard.bits(),  expected_value, "Board state should be {}", expected_value);
    }

    #[test]
    fn test_no_win_with_diagonal_four_disconnected() {
        let mut bitboard = BitBoard::empty();
        // tokens starting from row 1, col 1 have disconnect at [3,3] going to [4,4] and [5,5] but
        // not forming a win state
        let tokens: [[u8; 2]; 4] = [[1, 1], [2, 2], [4, 4], [5, 5]];
        set_token_positions(&mut bitboard, tokens);
        let expected_value:u64 = (1 << 8) + (1 << 16) + (1 << 32) + (1 << 40);
        assert_eq!(bitboard.has_won(), false, "Board should have not have a won state");
        assert_eq!(bitboard.bits(),  expected_value, "Board state should be {}", expected_value);
    }

    #[test]
    fn test_stride_with_two_next_columns_no_win() {
        let mut bitboard = BitBoard::empty();
        // tokens on column 2 rows 4 and 5 do not get calculated to
        // tokens on column 3 rows 0 and 1
        let tokens: [[u8; 2]; 4] = [[2, 4], [2, 5], [3, 0], [3, 1]];
        set_token_positions(&mut bitboard, tokens);
        let expected_value:u64 = (1 << 18) + (1 << 19) + (1 << 21) + (1 << 22);
        assert_eq!(bitboard.has_won(), false, "Board should have not have a won state");
        assert_eq!(bitboard.bits(),  expected_value, "Board state should be {}", expected_value);
    }

    #[test]
    fn test_debug_printing() {
        let mut bitboard = BitBoard::empty();
        let tokens: [[u8; 2]; 4] = [[1, 1], [2, 2], [3, 3], [4, 4]];
        set_token_positions(&mut bitboard, tokens);
        let debug_print = format!("{:?}", bitboard);
        let expected = "+-------+\n\
                                    |.......|\n\
                                    |....X..|\n\
                                    |...X...|\n\
                                    |..X....|\n\
                                    |.X.....|\n\
                                    |.......|\n\
                                    +-------+\n\
                                    \x200123456\n".to_string();
        assert_eq!(debug_print, expected, "Debug print did not match expected");
    }
}