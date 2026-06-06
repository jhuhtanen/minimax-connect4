use crate::constants::BOARD_WIDTH;

/// A Connect Four move.
///
/// A move is represented simply as the column index (0-based) where a piece
/// is to be dropped. The row is determined by the current board state
/// (gravity).
///
/// # Examples
///
/// ```
/// # use engine::moves::Move;
/// # const BOARD_WIDTH: u8 = 7;
/// let m = Move::new(3).unwrap();
/// assert_eq!(m.column(), 3);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Move {
	pub column: u8
}

/// Errors that can occur when creating a [`Move`].
#[derive(Debug)]
pub enum MoveError {
	ColumnOutOfBounds(u8),
}

impl Move {
	/// Creates a new [`Move`] for the given column.
	///
	/// Returns an error if the column is not in the valid range
	/// `[0, BOARD_WIDTH)`.
	///
	/// # Errors
	///
	/// - [`MoveError::ColumnOutOfBounds`] if `column >= BOARD_WIDTH`.
	///
	/// # Examples
	///
	/// ```
	/// # use engine::moves::{Move, MoveError};
	/// # const BOARD_WIDTH: u8 = 7;
	/// let m = Move::new(2).unwrap();
	/// assert_eq!(m.column(), 2);
	///
	/// let err = Move::new(BOARD_WIDTH).unwrap_err();
	/// if let MoveError::ColumnOutOfBounds(c) = err {
	///     assert_eq!(c, BOARD_WIDTH);
	/// }
	/// ```
	pub fn new(column: u8) -> Result<Self, MoveError> {
		if column >= BOARD_WIDTH {
			return Err(MoveError::ColumnOutOfBounds(column));
		}
		Ok(Move { column })
	}

	/// Returns the column index of this move.
	///
	/// # Examples
	///
	/// ```
	/// # use engine::moves::Move;
	/// let m = Move::new(4).unwrap();
	/// assert_eq!(m.column(), 4);
	/// ```
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