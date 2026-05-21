use ai::{GameState, MinMaxPlayer, Outcome, Player};
use crate::bitboard::BitBoard;
use crate::constants::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::moves::Move;
//use crate::player::Player;

#[derive(Clone)]
pub struct ConnectFourState {
    pub current_player: MinMaxPlayer,
    pub player1_board: BitBoard,
    pub player2_board: BitBoard,
    pub heights: [u8; 7],
}

#[derive(Debug, Clone)]
pub enum MoveError {
    ColumnOutOfBounds(u8),
    ColumnFull(u8),
}

impl GameState for ConnectFourState {
    type Move = Move;
    type MoveError = MoveError;

    fn current_player(&self) -> MinMaxPlayer {
        self.current_player
    }

    fn legal_moves(&self) -> Vec<Self::Move> {
        (0..BOARD_WIDTH)
            .filter(|&col| self.heights[col as usize] < BOARD_HEIGHT)
            .map(|col| { Move { column: col}})
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
            heights
        })
    }

    fn outcome(&self) -> Option<Outcome> {
        if self.player1_board.has_won() {
            Some(Outcome::Win(MinMaxPlayer::Max))
        } else if self.player2_board.has_won() {
            Some(Outcome::Win(MinMaxPlayer::Min))
        } else {
            None
        }
    }

    fn evaluate(&self) -> i32 {
        0
    }
}

impl ConnectFourState {
    pub fn new(starting_player: MinMaxPlayer) -> Self {
        ConnectFourState {
            current_player: starting_player,
            player1_board: BitBoard::empty(),
            player2_board: BitBoard::empty(),
            heights: [0; BOARD_WIDTH as usize]
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
}


#[test]
fn test_initial_state() {
    let game_state = ConnectFourState::new(MinMaxPlayer::Max);
    let expected_legal_moves = vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8]
        .iter()
        .map(|&x|  Move {column: x })
        .collect::<Vec<_>>();
    assert_eq!(game_state.legal_moves(), expected_legal_moves, "Expected legal columns do not match");
}

#[test]
fn test_fill_first_column() {
    let game_state = fill_column(ConnectFourState::new(MinMaxPlayer::Max), 0u8);
    let expected_legal_moves = vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8]
        .iter()
        .map(|&x|  Move {column: x })
        .collect::<Vec<_>>();
    assert_eq!(game_state.is_column_legal(0u8), false, "First column should be full");
    assert_eq!(game_state.current_player, MinMaxPlayer::Max, "Current player should be Max");
    assert_eq!(game_state.legal_moves(), expected_legal_moves, "Legal columns should be 1..6");

    let red_player_board = game_state.player1_board;
    let white_player_board = game_state.player2_board;
    // [column, row]
    let red_tokens = [[0,0],[0,2],[0,4]];
    let white_tokens = [[0,1],[0,3],[0,5]];
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
    let red_tokens = [[0,0],[1,0],[4,0],[5,0],[6,0],
                                [2,1],[3,1],
                                [0,2],[1,2],[4,2],[5,2],[6,2],
                                [2,3],[3,3],
                                [0,4],[1,4],[4,4],[5,4],[6,4],
                                [2,5],[3,5]];

    let white_tokens =  [[2,0],[3,0],
                                    [0,1],[1,1],[4,1],[5,1],[6,1],
                                    [2,2],[3,2],
                                    [0,3],[1,3],[4,3],[5,3],[6,3],
                                    [2,4],[3,4],
                                    [0,5],[1,5],[4,5],[5,5],[6,5]];
    // let's fill the board with specific token positioning
    for i in 0..(BOARD_HEIGHT * BOARD_WIDTH) as usize{
        let coord = {
            if i % 2 == 0 {
                red_tokens[i/2]
            } else {
                white_tokens[i/2]
            }
        };
        let mv = Move::new(coord[0]).unwrap();
        game_state = game_state.with_move(&mv).unwrap();
    }

    assert_eq!(game_state.outcome(), None, "Game shouldn't have a winner");
    assert!(game_state.is_draw(), "Game should be a draw");
    assert_eq!(game_state.legal_moves().iter().count(), 0, "Board should be full");
}