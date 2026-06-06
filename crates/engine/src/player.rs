
/// A player in the Connect Four game.
///
/// There are two players:
/// - `Red`
/// - `White`
///
/// The game alternates turns between these two players.
///
/// # Examples
///
/// ```
/// # use engine::player::Player;
/// let p = Player::Red;
/// assert_eq!(p.opponent(), Player::White);
/// assert_eq!(p.symbol(), 'R');
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    Red,
    White,
}

impl Player {
    /// Returns the opponent of this player.
    ///
    /// # Examples
    ///
    /// ```
    /// # use engine::player::Player;
    /// assert_eq!(Player::Red.opponent(), Player::White);
    /// assert_eq!(Player::White.opponent(), Player::Red);
    /// ```
    pub fn opponent(self) -> Self {
        match self {
            Player::Red => Player::White,
            Player::White => Player::Red,
        }
    }

    /// Returns a character symbol used to display this player on the board.
    ///
    /// Typically:
    /// - `Player::Red`   → `'R'`
    /// - `Player::White` → `'W'`
    ///
    pub fn symbol(self) -> char {
        match self {
            Player::Red => 'R',
            Player::White => 'W',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq};

    #[test]
    fn test_player_opponent() {
        let red_player = Player::Red;
        let white_player = Player::White;
        assert_eq!(red_player.opponent(), Player::White, "Red's opponent should be White");
        assert_eq!(white_player.opponent(), Player::Red, "White's opponent should be Red");
    }

    #[test]
    fn test_player_symbol() {
        let red_player = Player::Red;
        let white_player = Player::White;
        assert_eq!(red_player.symbol(), 'R', "Red's symbol should be 'R'");
        assert_eq!(white_player.symbol(), 'W', "White's symbol should be 'W'");
    }
}