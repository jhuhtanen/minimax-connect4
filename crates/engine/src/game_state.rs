use ai::{GameState, MinMaxPlayer, Outcome, Player};
use crate::bitboard::BitBoard;
use crate::constants::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::moves::Move;
use serde::{Deserialize, Serialize};

/// Version of the heuristic evaluation function used for a player.
///
/// - `V1` is a baseline heuristic (currently 0 for all non-terminal states).
/// - `V2` is the improved heuristic that scores 4-cell windows and threat patterns.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum HeuristicVersion {
    V1, // baseline (no heuristic)
    V2, // improved heuristic
}

/// Connect Four game state used by the search engine.
///
/// This struct holds the full information needed to search and play
/// a Connect Four position:
/// - which player is to move,
/// - bitboards for each player's pieces,
/// - the current height (number of pieces) in each column,
/// - and the heuristic version used per player.
///
/// It implements the generic [`GameState`] trait, so it can be used
/// directly with the AI crate's Minimax and iterative deepening search.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ConnectFourState {
    /// Player whose turn it is to move.
    pub current_player: MinMaxPlayer,
    /// Bitboard of pieces for the maximizing player (Max).
    pub player1_board: BitBoard,
    /// Bitboard of pieces for the minimizing player (Min).
    pub player2_board: BitBoard,
    /// Number of pieces in each column (0-based). Used both to
    /// determine legal moves and to test if a cell is immediately playable.
    pub heights: [u8; 7],
    /// Selected heuristic version for each player:
    /// index 0 = Max, index 1 = Min.
    pub player_heuristic: [HeuristicVersion; 2],
}

/// Errors that can occur when applying a move to the game state.
#[derive(Debug, Clone)]
pub enum MoveError {
    ColumnOutOfBounds(u8),
    ColumnFull(u8),
}

const MOVE_ORDER: [u8; 7] = [3, 2, 4, 1, 5, 0, 6];

const THREE_IN_ROW_IMMEDIATE: i32 = 1000;
const THREE_IN_ROW_FUTURE:i32 = 100;
const TWO_IN_ROW_WEIGHT:i32 = 10;
const ONE_IN_ROW_WEIGHT:i32 = 1;



impl GameState for ConnectFourState {
    type Move = Move;
    type MoveError = MoveError;

    fn current_player(&self) -> MinMaxPlayer {
        self.current_player
    }

    /// Returns all legal moves from this position, ordered for better pruning.
    ///
    /// Legal moves are those columns that are not yet full. The order
    /// is fixed in `MOVE_ORDER`, which starts from the center column and
    /// moves outward. This helps alpha-beta pruning.
    fn legal_moves(&self) -> Vec<Self::Move> {
        MOVE_ORDER
            .iter()
            .filter(|&col| self.heights[*col as usize] < BOARD_HEIGHT)
            .map(|col| { Move { column: *col}})
            .collect()
    }

    /// Applies a move and returns the resulting state.
    ///
    /// The move is applied by adding a piece for the current player
    /// in the given column, at the row indicated by `heights[col]`.
    /// The current player is then toggled to the opponent.
    ///
    /// # Errors
    ///
    /// - [`MoveError::ColumnFull`] if the column is already full.
    fn with_move(&self, mv: &Move) -> Result<Self, MoveError> {
        let col = mv.column() as usize;

        if self.is_column_legal(mv.column()) == false {
            return Err(MoveError::ColumnFull(mv.column()));
        }

        let mut heights = self.heights;
        heights[col] += 1;

        let bit_index = BitBoard::bit_index(mv.column(), self.heights[col]);

        let (updated_player1, updated_player2) = {
            match self.current_player {
                MinMaxPlayer::Max => {
                    let mut player1 = self.player1_board;
                    player1.with_bit_set(bit_index);
                    (player1, self.player2_board)
                }
                MinMaxPlayer::Min => {
                    let mut player2 = self.player2_board;
                    player2.with_bit_set(bit_index);
                    (self.player1_board, player2)
                }
            }
        };

        Ok(ConnectFourState {
            current_player: self.current_player.opponent(),
            player1_board: updated_player1,
            player2_board: updated_player2,
            heights,
            player_heuristic: self.player_heuristic,
        })
    }

    /// Returns the outcome of the game from this position, if terminal.
    ///
    /// - If the player who moved previously has a 4-in-a-row, returns `Win(...)`.
    /// - If the board is full and there is no winner, returns `Draw`.
    /// - Otherwise, returns `None` (game still in progress).
    fn outcome(&self) -> Option<Outcome> {
        // player moved previously
        let previous_player = self.current_player.opponent();
        let previous_board = match previous_player {
            MinMaxPlayer::Max => { &self.player1_board },
            MinMaxPlayer::Min => { &self.player2_board },
        };
        if previous_board.has_won() {
            return Some(Outcome::Win(previous_player));
        }
        if self.is_draw() {
            return Some(Outcome::Draw);
        }
        None
    }

    /// Evaluates the current position using the selected heuristic for the player to move.
    ///
    /// - For V1, returns 0 (no positional scoring).
    /// - For V2, uses a window-based heuristic:
    ///   counts pure 4-cell windows and scores 1/2/3-in-a-row patterns, with
    ///   a special bonus/penalty for 3-in-a-rows where the empty cell is
    ///   immediately playable.
    fn evaluate(&self) -> i32 {
        self.heuristic_score()
    }
}

impl ConnectFourState {
    /// Creates a new, empty Connect Four board with the given starting player.
    ///
    /// All bitboards are empty and column heights are zero. Both players
    /// default to using the baseline heuristic (V1).
    pub fn new(starting_player: MinMaxPlayer) -> Self {
        ConnectFourState {
            current_player: starting_player,
            player1_board: BitBoard::empty(),
            player2_board: BitBoard::empty(),
            heights: [0; BOARD_WIDTH as usize],
            player_heuristic: [HeuristicVersion::V1, HeuristicVersion::V1],
        }
    }

    /// Returns `true` if the game is a draw (no legal moves remain).
    ///
    /// This does not check for wins; `outcome()` handles win detection first,
    /// and only uses `is_draw()` as a fallback.
    pub fn is_draw(&self) -> bool {
        self.legal_moves().is_empty()
    }

    /// Returns `true` if a move in the given column would be legal.
    ///
    /// A column is legal if it is within bounds and not yet full.
    pub fn is_column_legal(&self, col: u8) -> bool {
        col < BOARD_WIDTH && self.heights[col as usize] < BOARD_HEIGHT
    }

    /// Returns `true` if a piece played at `(col, row)` would be immediately playable.
    ///
    /// This is used by the heuristic to distinguish between:
    /// - "future" threats (empty cell not yet reachable), and
    /// - "immediate" threats (empty cell is exactly at the next playable height
    ///   in that column).
    fn is_playable(&self, col: u8, row: u8) -> bool {
        let h = self.heights[col as usize];
        row == h
    }

    /// Returns which player (if any) has a piece at `(col, row)`.
    ///
    /// Uses the underlying bitboards to check ownership.
    pub fn token_at(&self, col: u8, row: u8) -> Option<MinMaxPlayer> {
        if self.player1_board.bit_at(col, row) {
            Some(MinMaxPlayer::Max)
        } else if self.player2_board.bit_at(col, row) {
            Some(MinMaxPlayer::Min)
        } else {
            None
        }
    }

    pub fn set_player_heuristic(&mut self, player: MinMaxPlayer, heuristic_version: HeuristicVersion) {
        let ind = match player {
            MinMaxPlayer::Max => 0,
            MinMaxPlayer::Min => 1,
        };
        self.player_heuristic[ind] = heuristic_version;
    }

    fn heuristic_score(&self) -> i32 {
        let ind = match self.current_player {
            MinMaxPlayer::Max => 0,
            MinMaxPlayer::Min => 1,
        };
        match self.player_heuristic[ind] {
            HeuristicVersion::V1 => self.heuristic_v1(),
            HeuristicVersion::V2 => self.heuristic_v2(),
        }
    }

    /// Baseline heuristic (V1): no positional scoring.
    ///
    /// All non-terminal states evaluate to 0.
    fn heuristic_v1(&self) -> i32 {
        0
    }

    /// Improved heuristic (V2) based on 4-cell windows.
    ///
    /// For every horizontal, vertical, and diagonal 4-cell window:
    /// - Count Max pieces, Min pieces, and empty cells.
    /// - If the window is "pure" (only one side's pieces + empties),
    ///   assign a score based on:
    ///   - 1-in-a-row → small,
    ///   - 2-in-a-row → medium,
    ///   - 3-in-a-row:
    ///     - If the empty cell is immediately playable in that column,
    ///       give a large bonus/penalty (`THREE_IN_ROW_IMMEDIATE`),
    ///     - Otherwise, use a smaller "future" weight (`THREE_IN_ROW_FUTURE`).
    fn heuristic_v2(&self) -> i32 {
        let mut score = 0;

        // horizontal
        for row in 0..BOARD_HEIGHT {
            for col in 0..(BOARD_WIDTH - 3) {
                let (counts, empty_cell) = self.count_window(|offset| (col + offset, row));
                score += self.score_window(counts, empty_cell);
            }
        }

        // vertical
        for col in 0..BOARD_WIDTH {
            for row in 0..(BOARD_HEIGHT - 3) {
                let (counts, empty_cell) = self.count_window(|offset| (col, row + offset));
                score += self.score_window(counts, empty_cell);
            }
        }

        // diagonal top-left to bottom-right (\)
        for col in 0..(BOARD_WIDTH - 3) {
            for row in (3..BOARD_HEIGHT).rev() {
                let (counts, empty_cell) = self.count_window(|offset| (col + offset, row - offset));
                score += self.score_window(counts, empty_cell);
            }
        }

        // diagonal bottom-left to top-right (/)
        for col in 0..(BOARD_WIDTH - 3) {
            for row in 0..(BOARD_HEIGHT - 3) {
                let (counts, empty_cell) = self.count_window(|offset| (col + offset, row + offset));
                score += self.score_window(counts, empty_cell);
            }
        }
        score
    }

    /// Counts Max/Min/empty cells in a 4-cell window defined by `coord(offset)`.
    ///
    /// Returns both the counts and, if exactly one empty cell exists,
    /// the coordinates of that empty cell.
    #[inline]
    fn count_window<F>(&self, coord: F) -> ((u16, u16, u16), Option<(u8, u8)>)
        where F: Fn(u8) -> (u8, u8) {

        let mut counts = (0u16, 0u16, 0u16);
        let mut empty_cell: Option<(u8, u8)> = None;

        for offset in 0..4 {
            let (x, y) = coord(offset);
            match self.token_at(x, y) {
                Some(MinMaxPlayer::Max) => counts.0 += 1,
                Some(MinMaxPlayer::Min) => counts.1 += 1,
                None => {
                    counts.2 += 1;
                    empty_cell = Some((x, y));
                }
            }
        }

        (counts, empty_cell)
    }

    /// Assigns a heuristic score to a single 4-cell window.
    ///
    /// The input is:
    /// - `(max_count, min_count, empty)` counts, and
    /// - `empty_cell` = the coordinates of the single empty cell if `empty == 1`.
    ///
    /// Mixed windows (both Max and Min present) return 0. Pure windows are
    /// scored according to 1/2/3-in-a-row patterns and whether the 3-in-a-row
    /// pattern is immediately playable.
    #[inline]
    fn score_window(&self, (max_count, min_count, empty): (u16, u16, u16), empty_cell: Option<(u8, u8)>, ) -> i32 {
        // mixed window - skip
        if max_count > 0 && min_count > 0 {
            return 0;
        }

        let mut score = 0;
        if max_count > 0 && min_count == 0 {
            // only Max
            if max_count == 3 && empty == 1 {
                // if the empty is actually playable, higher threat
                if let Some((col, row)) = empty_cell {
                    if self.is_playable(col, row) {
                        score += THREE_IN_ROW_IMMEDIATE;
                    } else {
                        score += THREE_IN_ROW_FUTURE;
                    }
                }
            } else if max_count == 2 && empty == 2 {
                score += TWO_IN_ROW_WEIGHT;
            } else if max_count == 1 && empty == 3 {
                score += ONE_IN_ROW_WEIGHT;
            }
        } else if min_count > 0 && max_count == 0 {
            // only Min
            if min_count == 3 && empty == 1 {
                // if the empty is actually playable, higher threat
                if let Some((col, row)) = empty_cell {
                    if self.is_playable(col, row) {
                        score -= THREE_IN_ROW_IMMEDIATE;
                    } else {
                        score -= THREE_IN_ROW_FUTURE;
                    }
                }
            } else if min_count == 2 && empty == 2 {
                score -= TWO_IN_ROW_WEIGHT;
            } else if min_count == 1 && empty == 3 {
                score -= ONE_IN_ROW_WEIGHT;
            }
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq};
    use std::fmt;
    use ai::{iterative_minimax, minimax, SearchConfig};
    use crate::pvs_data::PVS_TEST_CASES;

    #[cfg(test)]
    impl ConnectFourState {
        fn collect_four_in_rows(&self) -> Vec<(u16, u16, u16)> {
            let mut windows = Vec::new();

            // horizontal
            for row in 0..BOARD_HEIGHT {
                for col in 0..(BOARD_WIDTH - 3) {
                    let (counts, _empty_cell) = self.count_window(|offset| (col + offset, row));
                    windows.push(counts);
                }
            }

            // vertical
            for col in 0..BOARD_WIDTH {
                for row in 0..(BOARD_HEIGHT - 3) {
                    let (counts, _empty_cell) = self.count_window(|offset| (col, row + offset));
                    windows.push(counts);
                }
            }

            // diagonal (\)
            for col in 0..(BOARD_WIDTH - 3) {
                for row in (3..BOARD_HEIGHT).rev() {
                    let (counts, _empty_cell) = self.count_window(|offset| (col + offset, row - offset));
                    windows.push(counts);
                }
            }

            // diagonal  (/)
            for col in 0..(BOARD_WIDTH - 3) {
                for row in 0..(BOARD_HEIGHT - 3) {
                    let (counts, _empty_cell) = self.count_window(|offset| (col + offset, row + offset));
                    windows.push(counts);
                }
            }

            windows
        }
    }

    #[cfg(test)]
    impl fmt::Debug for ConnectFourState {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "+{}+", "-".repeat(BOARD_WIDTH as usize))?;

            for row in (0..BOARD_HEIGHT).rev() {
                write!(f, "|")?;
                for col in 0..BOARD_WIDTH {
                    let ch = match self.token_at(col, row) {
                        Some(player) => player.symbol(),
                        None => '.'
                    };
                    write!(f, "{ch}")?;
                }
                writeln!(f,"|")?;
            }

            writeln!(f, "+{}+", "-".repeat(BOARD_WIDTH as usize))?;
            write!(f, " ")?;
            for col in 0..BOARD_WIDTH {
                write!(f, "{col}")?;
            }
            writeln!(f)?;
            writeln!(f, " TURN: {:?}", self.current_player)?;
            writeln!(f, " HEURISTIC: {}",self.heuristic_score())?;
            Ok(())
        }
    }

    #[cfg(test)]
    impl ConnectFourState {
        /// Parse an ASCII board in the same format as `Display` into a sequence of moves.
        ///
        /// The input should look like:
        ///
        /// ```text
        /// +-------+
        /// |.......|
        /// |.......|
        /// |.......|
        /// |.......|
        /// |.......|
        /// |XXXX...|
        /// +-------+
        ///  0123456
        /// ```
        ///
        /// - Top row first, bottom row last.
        /// - `'.'` = empty, `'X'` / `'O'` = occupied.
        /// - This helper does **not** validate whether the position is reachable
        ///   by legal play; it just generates column moves that produce the same
        ///   column heights.
        pub fn moves_from_ascii(lines: &[&str], starting_player: MinMaxPlayer) -> Vec<Move> {
            // Expect BOARD_HEIGHT board rows + 2 borders + 1 index line
            assert_eq!(lines.len(), (BOARD_HEIGHT as usize) + 3,
                       "expected {} lines (1 top, {} rows, 1 bottom, 1 indices), got {}",
                        (BOARD_HEIGHT as usize) + 3,
                        BOARD_HEIGHT,
                        lines.len());

            // Extract inner board rows
            let mut inner_rows: Vec<String> = Vec::new();
            for i in 1..=BOARD_HEIGHT as usize {
                let row = lines[i];
                assert!(row.starts_with('|') && row.ends_with('|'),
                    "board row must start and end with '|'");
                let inner = &row[1..row.len() - 1];
                assert_eq!(inner.len(), BOARD_WIDTH as usize,
                    "inner row length must equal BOARD_WIDTH");
                inner_rows.push(inner.to_string());
            }

            let height = inner_rows.len();
            let width = BOARD_WIDTH as usize;

            // owner_at[col][row] with row 0 = bottom.
            let mut owner_at: Vec<Vec<Option<MinMaxPlayer>>> =
                vec![vec![None; height]; width];

            for ascii_row_idx in 0..height {
                // ascii_row_idx: 0 = top, height-1 = bottom
                let ascii_row = &inner_rows[ascii_row_idx];
                let row = height - 1 - ascii_row_idx; // row 0 = bottom
                for col in 0..width {
                    let ch = ascii_row.as_bytes()[col] as char;
                    owner_at[col][row] = match ch {
                        'X' => Some(MinMaxPlayer::Max),
                        'O' => Some(MinMaxPlayer::Min),
                        '.' => None,
                        other => panic!("unexpected character '{}' in board", other),
                    };
                }
            }

            // Column heights start at 0.
            let mut heights: Vec<usize> = vec![0; width];

            // Count total number of pieces.
            let total_pieces: usize = owner_at
                .iter()
                .map(|col_vec| col_vec
                    .iter()
                    .filter(|o| o.is_some())
                    .count())
                .sum();

            let mut moves: Vec<Move> = Vec::with_capacity(total_pieces);

            // Backtrack
            fn backtrack(owner_at: &Vec<Vec<Option<MinMaxPlayer>>>, size: &(usize, usize),
                heights: &mut [usize], current_player: MinMaxPlayer, placed: usize, total_pieces: usize,
                moves: &mut Vec<Move>, ) -> bool {

                if placed == total_pieces {
                    return true;
                }

                // Try all columns as candidates for this move.
                for col in 0..size.1 {
                    let row = heights[col];
                    // column full
                    if row >= size.0 {
                        continue;
                    }
                    if owner_at[col][row] == Some(current_player) {
                        // Try playing here.
                        moves.push(Move { column: col as u8 });
                        heights[col] += 1;

                        if backtrack(owner_at, size,
                            heights, current_player.opponent(),placed + 1, total_pieces,
                            moves, ) {
                            // found a full sequence
                            return true;
                        }

                        // Backtrack.
                        heights[col] -= 1;
                        moves.pop();
                    }
                }

                // No choice led to a solution.
                false
            }

            if !backtrack(&owner_at, &(height, width), &mut heights, starting_player,
                0, total_pieces, &mut moves, ) {
                panic!("unreachable position for this starting_player");
            }

            moves
        }
    }

    #[test]
    fn test_initial_state() {
        let game_state = ConnectFourState::new(MinMaxPlayer::Max);
        let expected_legal_moves = MOVE_ORDER
            .iter()
            .map(|&x| Move { column: x })
            .collect::<Vec<_>>();
        assert_eq!(game_state.legal_moves(), expected_legal_moves, "Expected legal columns do not match");
    }

    #[test]
    fn test_fill_first_column() {
        let game_state = fill_column(ConnectFourState::new(MinMaxPlayer::Max), 0u8);
        let expected_legal_moves = MOVE_ORDER
            .iter()
            .filter(|&x| *x != 0)
            .map(|&x| Move { column: x })
            .collect::<Vec<_>>();
        assert_eq!(game_state.is_column_legal(0u8), false, "First column should be full");
        assert_eq!(game_state.current_player, MinMaxPlayer::Max, "Current player should be Max");
        assert_eq!(game_state.legal_moves(), expected_legal_moves, "Legal columns should be 1..6");

        let red_player_board = game_state.player1_board;
        let white_player_board = game_state.player2_board;
        // [column, row]
        let red_tokens = [[0, 0], [0, 2], [0, 4]];
        let white_tokens = [[0, 1], [0, 3], [0, 5]];
        red_tokens
            .iter()
            .for_each(|coord| {
                assert!(red_player_board.bit_at(coord[0], coord[1]), "Red player token position mismatch")
            });
        white_tokens
            .iter()
            .for_each(|coord| {
                assert!(white_player_board.bit_at(coord[0], coord[1]), "White player token position mismatch")
            });
    }

    fn fill_column(game_state: ConnectFourState, column: u8) -> ConnectFourState {
        let mut game_state = game_state;
        let fill_move = Move::new(column).unwrap();
        for _ in 0..BOARD_HEIGHT {
            game_state = game_state.with_move(&fill_move).unwrap();
        }
        game_state
    }

    #[test]
    fn test_player_turn_change_after_move() {
        let game_state = ConnectFourState::new(MinMaxPlayer::Max);
        let mv = Move::new(0u8).unwrap();
        let new_state = game_state.with_move(&mv).unwrap();
        assert_eq!(new_state.current_player(), MinMaxPlayer::Min, "Expected player after turn change is Min");
    }

    #[test]
    fn test_player_trying_to_fill_full_column() {
        let game_state = fill_column(ConnectFourState::new(MinMaxPlayer::Max), 0u8);
        let mv = Move::new(0u8).unwrap();
        let result = game_state.with_move(&mv);
        assert!(result.is_err(), "Player shouldn't be able to play full column");
    }

    #[test]
    fn test_player_trying_to_fill_column_outside_bounds() {
        _ = ConnectFourState::new(MinMaxPlayer::Max);
        let result = Move::new(7u8);
        assert!(result.is_err(), "Player shouldn't be able to create invalid move");
    }

    #[test]
    fn test_no_winner_can_be_found() {
        // this test forms a specific pattern filling the board without
        // connecting 4 tokens for either player (X = Red, O = White)
        // XXOOXXX
        // OOXXOOO
        // XXOOXXX
        // OOXXOOO
        // XXOOXXX
        // OOXXOOO

        let mut game_state = ConnectFourState::new(MinMaxPlayer::Max);
        // [column, row]
        let red_tokens = [[0, 0], [1, 0], [4, 0], [5, 0], [6, 0],
            [2, 1], [3, 1],
            [0, 2], [1, 2], [4, 2], [5, 2], [6, 2],
            [2, 3], [3, 3],
            [0, 4], [1, 4], [4, 4], [5, 4], [6, 4],
            [2, 5], [3, 5]];

        let white_tokens = [[2, 0], [3, 0],
            [0, 1], [1, 1], [4, 1], [5, 1], [6, 1],
            [2, 2], [3, 2],
            [0, 3], [1, 3], [4, 3], [5, 3], [6, 3],
            [2, 4], [3, 4],
            [0, 5], [1, 5], [4, 5], [5, 5], [6, 5]];
        // let's fill the board with specific token positioning
        for i in 0..(BOARD_HEIGHT * BOARD_WIDTH) as usize {
            let coord = {
                if i % 2 == 0 {
                    red_tokens[i / 2]
                } else {
                    white_tokens[i / 2]
                }
            };
            let mv = Move::new(coord[0]).unwrap();
            game_state = game_state.with_move(&mv).unwrap();
        }

        assert_eq!(game_state.outcome(), Some(Outcome::Draw), "Game shouldn't have a winner");
        assert!(game_state.is_draw(), "Game should be a draw");
        assert_eq!(game_state.legal_moves().iter().count(), 0, "Board should be full");
    }

    #[test]
    fn test_outcome_max_wins() {
        // we need to start with current player being Min, but we do not do any moves
        // hence the previous player was Max
        let mut game = ConnectFourState::new(MinMaxPlayer::Min);
        game.set_player_heuristic(MinMaxPlayer::Max, HeuristicVersion::V2);

        // column, row
        let max_tokens = [[0, 0], [1, 0], [2, 0], [3, 0]];
        max_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0], coord[1]);
            game.player1_board.with_bit_set(bit_index);
        });

        assert_eq!(Some(Outcome::Win(MinMaxPlayer::Max)), game.outcome(), "Max should have won");
    }

    #[test]
    fn test_outcome_min_wins() {
        // we need to start with current player being Max, but we do not do any moves
        // hence the previous player was Min
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
        game.set_player_heuristic(MinMaxPlayer::Min, HeuristicVersion::V2);

        // column, row
        let min_tokens = [[0, 0], [0, 1], [0, 2], [0, 3]];
        min_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0], coord[1]);
            game.player2_board.with_bit_set(bit_index);
        });

        assert_eq!(Some(Outcome::Win(MinMaxPlayer::Min)), game.outcome(), "Min should have won");
    }

    #[test]
    fn test_outcome_heuristic_evaluate() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
        game.set_player_heuristic(MinMaxPlayer::Min, HeuristicVersion::V2);

        assert_eq!(0, game.evaluate(), "Evaluate should have been 0 for empty board");
    }

    #[test]
    fn test_no_heuristics() {
        let game = ConnectFourState::new(MinMaxPlayer::Max);
        assert_eq!(game.heuristic_score(), 0, "Heuristic v1 score should be 0");
    }

    #[test]
    fn test_heuristics_v2_returns_with_empty_board_is_zero() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
        game.set_player_heuristic(MinMaxPlayer::Max, HeuristicVersion::V2);

        assert_eq!(game.heuristic_score(), 0, "Heuristic v2 score on empty board should be 0");
    }

    #[test]
    fn test_heuristics_v2_returns_more_than_zero_for_max() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
        game.set_player_heuristic(MinMaxPlayer::Max, HeuristicVersion::V2);
        // let's put just one piece somewhere
        let bit_index = BitBoard::bit_index(3, 3);
        game.player1_board.with_bit_set(bit_index);

        assert!(game.heuristic_score() > 0, "Heuristic v2 score should be above 0");
    }

    #[test]
    fn test_heuristics_v2_returns_less_than_zero_for_min() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Min);
        game.set_player_heuristic(MinMaxPlayer::Min, HeuristicVersion::V2);
        // let's put just one piece somewhere
        let bit_index = BitBoard::bit_index(3, 3);
        game.player2_board.with_bit_set(bit_index);

        assert!(game.heuristic_score() < 0, "Heuristic v2 score should be below 0");
    }

    #[test]
    fn test_all_four_in_row_counts_match() {
        let game = ConnectFourState::new(MinMaxPlayer::Max);
        let results = game.collect_four_in_rows();
        assert_eq!(results.len(), 69, "There should be total 69 four in row possibilities");
        results.iter().for_each(|&x| {
            assert_eq!(x, (0, 0, 4), "For empty board all four in rows should be (0,0,4)")
        })
    }

    #[test]
    fn test_all_four_in_row_for_max_horizontal() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
        // Max should have at least these, there are multiple (1,0,3) due to verticals and diagonals as well
        let expected_four_in_row = vec![(4, 0, 0), (3, 0, 1), (2, 0, 2), (1, 0, 3)];
        //+-------+
        //|.......|
        //|.......|
        //|.......|
        //|.......|
        //|.......|
        //|XXXX...|
        //+-------+
        // 0123456

        // column, row
        let max_tokens = [[0, 0], [1, 0], [2, 0], [3, 0]];
        max_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0], coord[1]);
            game.player1_board.with_bit_set(bit_index);
        });
        let fours = game.collect_four_in_rows();
        expected_four_in_row
            .iter()
            .for_each(|x| {
                assert!(fours.contains(&x), "Four in rows should contain {:?}", &x);
            });
        assert!(game.heuristic_v2() > TWO_IN_ROW_WEIGHT, "Heuristic score should be higher than {}", TWO_IN_ROW_WEIGHT);
    }

    #[test]
    fn test_all_four_in_row_for_min_horizontal() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Min);
        // Min should have at least these, there are multiple (0,1,3) due to verticals and diagonals as well
        let expected_four_in_row = vec![(0, 4, 0), (0, 3, 1), (0, 2, 2), (0, 1, 3)];
        //+-------+
        //|...XXXX|
        //|.......|
        //|.......|
        //|.......|
        //|.......|
        //|.......|
        //+-------+
        // 0123456

        // column, row
        let min_tokens = [[3, 5], [4, 5], [5, 5], [6, 5]];
        min_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0], coord[1]);
            game.player2_board.with_bit_set(bit_index);
        });
        let fours = game.collect_four_in_rows();
        expected_four_in_row
            .iter()
            .for_each(|x| {
                assert!(fours.contains(&x), "Four in rows should contain {:?}", &x);
            });
        assert!(game.heuristic_v2() < TWO_IN_ROW_WEIGHT, "Heuristic score should be lower than {}", TWO_IN_ROW_WEIGHT);
    }

    #[test]
    fn test_all_three_in_row_for_max_vertical() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
        let three_in_row_max = (3, 0, 1); // 3 in row for max, 0 for min, 1 empty space
        //+-------+
        //|X..X...|
        //|X..X...|
        //|X..X...|
        //|.......|
        //|.......|
        //|.......|
        //+-------+
        // 0123456
        // column, row
        let max_tokens = [[0, 5], [0, 4], [0, 3], [3, 5], [3, 4], [3, 3]];
        max_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0], coord[1]);
            game.player1_board.with_bit_set(bit_index);
        });
        let fours = game.collect_four_in_rows();
        let three_in_row_actual = fours
            .iter()
            .filter(|&x| { *x == three_in_row_max })
            .collect::<Vec<_>>()
            .iter()
            .count();
        assert_eq!(three_in_row_actual, 2, "Max should have two three in rows");
        assert!(game.heuristic_v2() > TWO_IN_ROW_WEIGHT, "Heuristic score should be higher than {}", TWO_IN_ROW_WEIGHT)
    }

    #[test]
    fn test_all_three_in_row_for_min_vertical() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Min);
        let three_in_row_min = (0, 3, 1); // 3 in row for min, 0 for max, 1 empty space
        let two_in_row_min = (0, 2, 2); // 2 in row for min, 0 for max, 2 empty space
        //+-------+
        //|.......|
        //|.......|
        //|....X..|
        //|..X.X..|
        //|..X.X..|
        //|..X....|
        //+-------+
        // 0123456
        // column, row
        let min_tokens = [[2, 0], [2, 1], [2, 2], [4, 1], [4, 2], [4, 3]];
        min_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0], coord[1]);
            game.player2_board.with_bit_set(bit_index);
        });
        let fours = game.collect_four_in_rows();
        let three_in_row_actual = fours
            .iter()
            .filter(|&x| { *x == three_in_row_min })
            .collect::<Vec<_>>()
            .iter()
            .count();
        let two_in_row_actual = fours
            .iter()
            .filter(|&x| { *x == two_in_row_min })
            .collect::<Vec<_>>()
            .iter()
            .count();
        assert_eq!(three_in_row_actual, 3, "Min should have 3 three in rows");
        assert_eq!(two_in_row_actual, 9, "Min should have 9 two in rows");
        assert!(game.heuristic_v2() < TWO_IN_ROW_WEIGHT, "Heuristic score should be lower than {}", TWO_IN_ROW_WEIGHT)
    }

    #[test]
    fn test_all_three_in_row_for_max_diagonal() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
        let three_in_row_max = (3, 0, 1); // 3 in row for max, 0 for min, 1 empty space
        // +-------+
        // |.......|
        // |.X...X.|
        // |..X.X..|
        // |...X...|
        // |.......|
        // |.......|
        // +-------+
        //  0123456
        // column, row
        let max_tokens = [[1, 4], [2, 3], [3, 2], [4, 3], [5, 4]];
        max_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0], coord[1]);
            game.player1_board.with_bit_set(bit_index);
        });
        let fours = game.collect_four_in_rows();
        let three_in_row_actual = fours
            .iter()
            .filter(|&x| { *x == three_in_row_max })
            .collect::<Vec<_>>()
            .iter()
            .count();
        assert_eq!(three_in_row_actual, 4, "Max should have four three in rows");
        assert!(game.heuristic_v2() > TWO_IN_ROW_WEIGHT, "Heuristic score should be higher than {}", TWO_IN_ROW_WEIGHT);
    }

    #[test]
    fn test_all_three_in_row_for_min_diagonal() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Min);
        let three_in_row_min = (0, 3, 1); // 3 in row for min, 0 for max, 1 empty space
        let twos_in_row_min = (0, 2, 2); // 2 in row for min, 0 for max, 2 empty space
        // +-------+
        // |.......|
        // |XX.....|
        // |.XX....|
        // |..XX...|
        // |.......|
        // |.......|
        // +-------+
        //  0123456
        // column, row
        let min_tokens = [[0,4],[1,3],[2,2],[1,4],[2,3],[3,2]];
        min_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0],coord[1]);
            game.player2_board.with_bit_set(bit_index);
        });
        let fours = game.collect_four_in_rows();
        let three_in_row_actual = fours
            .iter()
            .filter(|&x| { *x == three_in_row_min })
            .collect::<Vec<_>>()
            .iter()
            .count();
        let twos_in_row_actual = fours
            .iter()
            .filter(|&x| { *x == twos_in_row_min })
            .collect::<Vec<_>>()
            .iter()
            .count();
        assert_eq!(three_in_row_actual, 3, "Min should have 3 three in rows");
        assert_eq!(twos_in_row_actual, 13, "Min should have 13 twos in rows");
        assert!(game.heuristic_v2() < TWO_IN_ROW_WEIGHT, "Heuristic score should be lower than {}", TWO_IN_ROW_WEIGHT);
    }

    #[test]
    fn test_mixed_four_in_row_window() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
        let mixed_rows_max = (2, 1, 1); // 2 for max, 1 for min, 1 empty space
        let one_in_rows_max = (1, 0, 3); // 2 for max, 1 for min, 1 empty space
        let mixed_rows_min = (1, 2, 1); // 1  for max, 1 for min, 1 empty space
        // +-------+
        // |.......|
        // |.X...O.|
        // |..O..X.|
        // |...X.O.|
        // |.......|
        // |.......|
        // +-------+
        //  0123456
        // column, row
        let max_tokens = [[1, 4], [3, 2], [5, 3]];
        let min_tokens = [[2, 3], [5, 4], [5, 2]];
        max_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0], coord[1]);
            game.player1_board.with_bit_set(bit_index);
        });
        min_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0], coord[1]);
            game.player2_board.with_bit_set(bit_index);
        });
        let fours = game.collect_four_in_rows();
        let mixed_max_actual = fours
            .iter()
            .filter(|&x| { *x == mixed_rows_max })
            .collect::<Vec<_>>()
            .iter()
            .count();
        let mixed_min_actual = fours
            .iter()
            .filter(|&x| { *x == mixed_rows_min })
            .collect::<Vec<_>>()
            .iter()
            .count();
        let one_in_rows_max_actual = fours
            .iter()
            .filter(|&x| { *x == one_in_rows_max })
            .collect::<Vec<_>>()
            .iter()
            .count();
        assert_eq!(mixed_max_actual, 2, "Max should have 2 mixed rows");
        assert_eq!(mixed_min_actual, 2, "Min should have 2 mixed rows");
        assert_eq!(one_in_rows_max_actual, 14, "Max should have 14 one in rows");
        assert_eq!(game.heuristic_v2(), 2, "Heuristic score should be 2 for Max");
    }

    #[test]
    fn test_debug_printing() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
        // +-------+
        // |.......|
        // |.X...X.|
        // |..X.X..|
        // |...X...|
        // |.OOOOO.|
        // |.......|
        // +-------+
        //  0123456
        // column, row
        let max_tokens = [[1, 4], [2, 3], [3, 2], [4, 3], [5, 4]];
        max_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0], coord[1]);
            game.player1_board.with_bit_set(bit_index);
        });

        let min_tokens = [[1, 1], [2, 1], [3, 1], [4, 1], [5, 1]];
        min_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0], coord[1]);
            game.player2_board.with_bit_set(bit_index);
        });
        let debug_print = format!("{:?}", game);
        let expected = "+-------+\n\
                                    |.......|\n\
                                    |.X...X.|\n\
                                    |..X.X..|\n\
                                    |...X...|\n\
                                    |.OOOOO.|\n\
                                    |.......|\n\
                                    +-------+\n\
                                    \x200123456\n\
                                    \x20TURN: Max\n\
                                    \x20HEURISTIC: 0\n\
                                    ".to_string();
        std::assert_eq!(debug_print, expected, "Debug print did not match expected");
    }

    // invariant tests
    #[test]
    fn test_piece_count_difference_invariant() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);

        // put some tokens on the board
        for col in [0, 1, 2, 3, 4] {
            let mv = Move::new(col).unwrap();
            if game.is_column_legal(col) {
                game = game.with_move(&mv).unwrap();
            }
        }

        // count tokens on the board
        let mut max_count = 0;
        let mut min_count = 0;
        for col in 0..BOARD_WIDTH {
            for row in 0..BOARD_HEIGHT {
                match game.token_at(col, row) {
                    Some(MinMaxPlayer::Max) => max_count += 1,
                    Some(MinMaxPlayer::Min) => min_count += 1,
                    None => {}
                }
            }
        }

        // difference has to be less or equal than 1
        assert!((max_count  - min_count ) <= 1);
    }

    #[test]
    fn test_terminal_and_legal_moves_consistency() {
        // terminal state first (Min starts)
        let mut game = ConnectFourState::new(MinMaxPlayer::Min);
        // column, row
        let max_tokens = [[3, 0], [4, 0], [5, 0], [6, 0]];
        max_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0], coord[1]);
            game.player1_board.with_bit_set(bit_index);
        });

        assert!(game.outcome().is_some());

        // Non-terminal state (Max starts)
        let game2 = ConnectFourState::new(MinMaxPlayer::Max);
        // column, row
        let max_tokens = [[2, 0], [3, 0], [5, 0], [6, 0]];
        max_tokens.iter().for_each(|coord| {
            let bit_index = BitBoard::bit_index(coord[0], coord[1]);
            game.player1_board.with_bit_set(bit_index);
        });

        assert!(game2.outcome().is_none());
        assert_ne!(game2.legal_moves().is_empty(), true, "We chould have remaining legal moves");
    }

    #[test]
    fn test_no_overlapping_pieces_in_bitboards() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);

        // columns to play
        let columns = [3, 2, 3, 3, 2, 4, 4, 3, 3, 2, 2];
        columns.iter().for_each(|&col| {
            game = game.with_move(&Move::new(col).unwrap()).unwrap();
        });

        // check there's no overlap
        for col in 0..BOARD_WIDTH {
            for row in 0..BOARD_HEIGHT {
                let max_here = game.player1_board.bit_at(col, row);
                let min_here = game.player2_board.bit_at(col, row);
                assert!(!(max_here && min_here), "Board has both Max and Min at ({col},{row})");
            }
        }
    }

    #[test]
    fn test_heights_match_bitboards() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);

        // columns to play
        let columns = [3, 2, 3, 3, 2, 4, 4, 3, 3, 2, 2];
        columns.iter().for_each(|&col| {
            game = game.with_move(&Move::new(col).unwrap()).unwrap();
        });

        for col in 0..BOARD_WIDTH {
            let mut count = 0;
            for row in 0..BOARD_HEIGHT {
                if game.token_at(col, row).is_some() {
                    count += 1;
                }
            }
            assert_eq!(game.heights[col as usize] as usize, count,
                       "heights[{}] doesn't match number of pieces in column", col);
        }
    }

    #[test]
    fn test_current_player_alternates() {
        let start = MinMaxPlayer::Max;
        let mut game = ConnectFourState::new(start);

        // columns to play
        let columns = [0, 1, 2, 3, 4, 5, 6];
        for (i, col) in columns.iter().enumerate() {
            let mv = Move::new(*col).unwrap();
            game = game.with_move(&mv).unwrap();

            let expected = match (i + 1) % 2 == 0 {
                true => start,
                false => start.opponent(),
            };
            assert_eq!(game.current_player, expected, "Current player isn't expected player {:?}", expected);
        }
    }

    // invariant tests end

    #[test]
    fn test_moves_from_ascii_yields_same_state_when_moves_applied() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);

        let state: &str = "+-------+\n\
                        |.......|\n\
                        |...X...|\n\
                        |...O...|\n\
                        |..OX...|\n\
                        |.OOO..O|\n\
                        |.XXX..X|\n\
                        +-------+\n\
                        \x200123456";

        // create moves leading to desired state
        let moves = ConnectFourState::moves_from_ascii(&state.lines().collect::<Vec<_>>(), MinMaxPlayer::Max);
        // apply moves
        moves
            .iter()
            .for_each(|m| {
                game = game.with_move(m).unwrap();
            });
        // verify state
        let actual = format!("{:?}", game);
        actual
            .lines()
            .take(BOARD_HEIGHT as usize + 2)
            .for_each(|line| {
                assert!(state.contains(line), "The actual state doesn't contain line {}", line);
            });
    }

    #[test]
    fn test_moves_from_ascii_yields_same_state_when_moves_applied_all() {
        for (_, ascii, starting_player, depth) in PVS_TEST_CASES {
            let moves = ConnectFourState::moves_from_ascii(*ascii, *starting_player);

            let mut game = ConnectFourState::new(*starting_player);

            // apply moves
            moves
                .iter()
                .for_each(|m| {
                    game = game.with_move(m).unwrap();
                });
            // verify state
            let actual = format!("{:?}", game);
            let state: String = ascii.join("");
            actual
                .lines()
                .take(BOARD_HEIGHT as usize + 2)
                .for_each(|line| {
                    assert!(state.contains(line), "The actual state doesn't contain line {}", line);
                });
        }
    }

    #[test]
    fn iterative_pvs_matches_plain_iterative_on_midgame_positions() {
        for (name, ascii, starting_player, depth) in PVS_TEST_CASES {
            let moves = ConnectFourState::moves_from_ascii(*ascii, *starting_player);

            let mut game = ConnectFourState::new(*starting_player);
            game.set_player_heuristic(MinMaxPlayer::Max, HeuristicVersion::V2);
            game.set_player_heuristic(MinMaxPlayer::Min, HeuristicVersion::V2);

            for mv in &moves {
                game = game.with_move(mv).unwrap();
            }

            let mut cfg = SearchConfig::new_alpha_beta(*depth);
            cfg.time_ms = None;
            cfg.pvs_start_depth = 4;

            let iterative_plain = iterative_minimax(&game, &cfg);
            let pvs   = minimax(&game, &cfg);

            assert_eq!(iterative_plain.score, pvs.score, "PVS must match plain iterative minimax score");
            assert_eq!(iterative_plain.best_move, pvs.best_move, "PVS must choose same best move");
            if pvs.nodes_visited > iterative_plain.nodes_visited {
                println!(
                        "Case: {} PVS visited more nodes {} > {} than pure iterative",
                        name, pvs.nodes_visited, iterative_plain.nodes_visited);
            }
        }

    }
}