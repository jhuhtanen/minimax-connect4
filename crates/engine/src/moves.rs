use crate::constants::BOARD_WIDTH;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Move {
	pub column: u8
}

#[derive(Debug)]
pub enum MoveError {
	ColumnOutOfBounds(u8),
}

impl Move {
	pub fn new(column: u8) -> Result<Self, MoveError> {
		if column >= BOARD_WIDTH {
			return Err(MoveError::ColumnOutOfBounds(column));
		}
		Ok(Move { column })
	}

	pub fn column(&self) -> u8 {
		self.column
	}
}

#[test]
fn test_valid_move_creation() {
	let valid_columns = vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8];
	valid_columns
		.iter()
		.for_each(|&col| {
			_ = Move::new(col).unwrap();
		});

}

#[test]
fn test_invalid_move_creation() {
	let result = Move::new(7u8);
	assert!(result.is_err(), "Invalid move should return Err");
}