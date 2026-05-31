use ai::{GameState, MinMaxPlayer, Outcome, Player};
use crate::bitboard::BitBoard;
use crate::constants::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::moves::Move;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum HeuristicVersion {
    V1, // baseline (no heuristic)
    V2, // improved heuristic
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ConnectFourState {
    pub current_player: MinMaxPlayer,
    pub player1_board: BitBoard,
    pub player2_board: BitBoard,
    pub heights: [u8; 7],
    pub player_heuristic: [HeuristicVersion; 2],
}

#[derive(Debug, Clone)]
pub enum MoveError {
    ColumnOutOfBounds(u8),
    ColumnFull(u8),
}

const COLUMN_WEIGHTS: [u64;7] = [2, 3, 5, 7, 5, 3, 2];
const MOVE_ORDER: [u8; 7] = [3, 2, 4, 1, 5, 0, 6];
const THREE_IN_ROW_WEIGHT:i32 = 100;
const THREE_IN_ROW_WEIGHT_OPPONENT:i32 = 120;
const TWO_IN_ROW_WEIGHT:i32 = 50;
const TWO_IN_ROW_WEIGHT_OPPONENT:i32 = 60;
const ONE_IN_ROW_WEIGHT:i32 = 1;
const ONE_IN_ROW_WEIGHT_OPPONENT:i32 = 1;



impl GameState for ConnectFourState {
    type Move = Move;
    type MoveError = MoveError;

    fn current_player(&self) -> MinMaxPlayer {
        self.current_player
    }

    fn legal_moves(&self) -> Vec<Self::Move> {
        MOVE_ORDER
            .iter()
            .filter(|&col| self.heights[*col as usize] < BOARD_HEIGHT)
            .map(|col| { Move { column: *col}})
            .collect()
    }

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

    fn outcome(&self) -> Option<Outcome> {
        if self.player1_board.has_won() {
            Some(Outcome::Win(MinMaxPlayer::Max))
        } else if self.player2_board.has_won() {
            Some(Outcome::Win(MinMaxPlayer::Min))
        } else if self.is_draw() {
            Some(Outcome::Draw)
        } else {
            None
        }
    }

    fn evaluate(&self) -> i32 {
        self.heuristic_score()
    }
}

impl ConnectFourState {
    pub fn new(starting_player: MinMaxPlayer) -> Self {
        ConnectFourState {
            current_player: starting_player,
            player1_board: BitBoard::empty(),
            player2_board: BitBoard::empty(),
            heights: [0; BOARD_WIDTH as usize],
            player_heuristic: [HeuristicVersion::V1, HeuristicVersion::V1],
        }
    }

    pub fn is_draw(&self) -> bool {
        self.legal_moves().iter().count() == 0 &&
            self.player1_board.has_won() == false && self.player2_board.has_won() == false
    }

    pub fn is_column_legal(&self, col: u8) -> bool {
        col < BOARD_WIDTH && self.heights[col as usize] < BOARD_HEIGHT
    }

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

    fn heuristic_v1(&self) -> i32 {
        0
    }
    
    fn heuristic_v2(&self) -> i32 {
        let mut score = 0;

        for col in 0 .. BOARD_WIDTH {
            for row in 0 .. BOARD_HEIGHT {
                if let Some(player) = self.token_at(col, row) {
                    if player == MinMaxPlayer::Max {
                        score += COLUMN_WEIGHTS[col as usize] as i32;
                    } else if player == MinMaxPlayer::Min {
                        score -= COLUMN_WEIGHTS[col as usize] as i32;
                    }
                }
            }
        }

        // horizontal
        for row in 0..BOARD_HEIGHT {
            for col in 0..(BOARD_WIDTH - 3) {
                let counts = self.count_window(|offset| (col + offset, row));
                score += self.score_window(counts);
            }
        }

        // vertical
        for col in 0..BOARD_WIDTH {
            for row in 0..(BOARD_HEIGHT - 3) {
                let counts = self.count_window(|offset| (col, row + offset));
                score += self.score_window(counts);
            }
        }

        // diagonal top-left to bottom-right (\)
        for col in 0..(BOARD_WIDTH - 3) {
            for row in (3..BOARD_HEIGHT).rev() {
                let counts = self.count_window(|offset| (col + offset, row - offset));
                score += self.score_window(counts);
            }
        }

        // diagonal bottom-left to top-right (/)
        for col in 0..(BOARD_WIDTH - 3) {
            for row in 0..(BOARD_HEIGHT - 3) {
                let counts = self.count_window(|offset| (col + offset, row + offset));
                score += self.score_window(counts);
            }
        }
        score
    }

    #[inline]
    fn count_window<F>(&self, coord: F) -> (u16, u16, u16)
        where F: Fn(u8) -> (u8, u8),
    {
        let mut counts = (0u16, 0u16, 0u16);
        for offset in 0..4 {
            let (x, y) = coord(offset);
            self.increase_counts(&mut counts, x, y);
        }
        counts
    }

    #[inline]
    fn score_window(&self, (max_count, min_count, empty): (u16, u16, u16)) -> i32 {
        // mixed window - skip
        if max_count > 0 && min_count > 0 {
            return 0;
        }

        let mut score = 0;
        if max_count > 0 && min_count == 0 {
            // only Max
            if max_count == 3 && empty == 1 {
                score += THREE_IN_ROW_WEIGHT;
            } else if max_count == 2 && empty == 2 {
                score += TWO_IN_ROW_WEIGHT;
            } else if max_count == 1 && empty == 3 {
                score += ONE_IN_ROW_WEIGHT;
            }
        } else if min_count > 0 && max_count == 0 {
            // only Min
            if min_count == 3 && empty == 1 {
                score -= THREE_IN_ROW_WEIGHT_OPPONENT;
            } else if min_count == 2 && empty == 2 {
                score -= TWO_IN_ROW_WEIGHT_OPPONENT;
            } else if min_count == 1 && empty == 3 {
                score -= ONE_IN_ROW_WEIGHT_OPPONENT;
            }
        }
        score
    }

    fn increase_counts(&self, counts: &mut (u16, u16, u16), x: u8, y: u8) {
        match self.token_at(x, y) {
            Some(player) if player == MinMaxPlayer::Max => counts.0 += 1,
            Some(player) if player == MinMaxPlayer::Min => counts.1 += 1,
            None => counts.2 += 1,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq};
    use ai::{minimax, SearchConfig};

    #[cfg(test)]
    impl ConnectFourState {
        fn collect_four_in_rows(&self) -> Vec<(u16, u16, u16)> {
            let mut windows = Vec::new();

            // horizontal
            for row in 0..BOARD_HEIGHT {
                for col in 0..(BOARD_WIDTH - 3) {
                    windows.push(self.count_window(|offset| (col + offset, row)));
                }
            }

            // vertical
            for col in 0..BOARD_WIDTH {
                for row in 0..(BOARD_HEIGHT - 3) {
                    windows.push(self.count_window(|offset| (col, row + offset)));
                }
            }

            // diagonal (\)
            for col in 0..(BOARD_WIDTH - 3) {
                for row in (3..BOARD_HEIGHT).rev() {
                    windows.push(self.count_window(|offset| (col + offset, row - offset)));
                }
            }

            // diagonal  (/)
            for col in 0..(BOARD_WIDTH - 3) {
                for row in 0..(BOARD_HEIGHT - 3) {
                    windows.push(self.count_window(|offset| (col + offset, row + offset)));
                }
            }

            windows
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
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
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
        let mut game = ConnectFourState::new(MinMaxPlayer::Min);
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
        assert!(game.heuristic_v2() < TWO_IN_ROW_WEIGHT_OPPONENT, "Heuristic score should be lower than {}", TWO_IN_ROW_WEIGHT_OPPONENT);
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
        assert!(game.heuristic_v2() < TWO_IN_ROW_WEIGHT_OPPONENT, "Heuristic score should be lower than {}", TWO_IN_ROW_WEIGHT_OPPONENT)
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
        assert!(game.heuristic_v2() < TWO_IN_ROW_WEIGHT_OPPONENT, "Heuristic score should be lower than {}", TWO_IN_ROW_WEIGHT_OPPONENT);
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
        assert_eq!(game.heuristic_v2(), 4, "Heuristic score should be 4 for Max");

    }
}