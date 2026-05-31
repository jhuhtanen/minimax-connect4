
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    Red,
    White,
}

impl Player {
    pub fn opponent(self) -> Self {
        match self {
            Player::Red => Player::White,
            Player::White => Player::Red,
        }
    }

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