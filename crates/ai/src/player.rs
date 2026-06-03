use serde::{Deserialize, Serialize};

/// A player type used by the minimax algorithm.
///
/// - [`MinMaxPlayer::Max`] is the player for whom we are maximizing the score.
/// - [`MinMaxPlayer::Min`] is the opponent (minimizing player).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MinMaxPlayer { Max, Min }

/// Represents a player in a two-player zero-sum game.
///
/// The minimax algorithm is implemented generically and operates on any
/// game whose player type implements this trait. Implementations provide
/// a way to switch turns between players and get a symbol
/// for displaying game states.
///
/// Typical implementations are game-specific enums such as
/// `ConnectFourPlayer` or `TicTacToePlayer`.
pub trait Player : Clone {
    /// Returns the opponent of this player.
    fn opponent(self) -> Self;

    /// Returns the character symbol used by this player
    fn symbol(self) -> char;
}

impl Player for MinMaxPlayer {
    fn opponent(self) -> Self {
        match self {
            MinMaxPlayer::Max => MinMaxPlayer::Min,
            MinMaxPlayer::Min => MinMaxPlayer::Max,
        }
    }

    fn symbol(self) -> char {
        match self {
            MinMaxPlayer::Max => 'X',
            MinMaxPlayer::Min => 'O',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn player_opponent() {
        let min_player = MinMaxPlayer::Min;
        assert_eq!(min_player.opponent(), MinMaxPlayer::Max, "Min player opponent should be Max");

        let max_player = MinMaxPlayer::Max;
        assert_eq!(max_player.opponent(), MinMaxPlayer::Min, "Max player opponent should be Min");
    }

    #[test]
    fn player_symbol() {
        let min_player = MinMaxPlayer::Min;
        assert_eq!(min_player.symbol(), 'O', "Min player symbol should be 'O'");

        let max_player = MinMaxPlayer::Max;
        assert_eq!(max_player.symbol(), 'X', "Max player symbol should be 'X'");
    }
}