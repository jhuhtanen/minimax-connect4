
/// A player type used by the minimax algorithm.
///
/// - [`MinMaxPlayer::Max`] is the player for whom we are maximizing the score.
/// - [`MinMaxPlayer::Min`] is the opponent (minimizing player).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MinMaxPlayer { Max, Min }

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