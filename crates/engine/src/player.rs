
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