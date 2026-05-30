use std::fmt::Debug;
use crate::outcome::Outcome;
use crate::player::{MinMaxPlayer};

pub trait GameState: Clone {
    /// The type of moves that can be played from this state.
    type Move: Clone + Debug + PartialEq;

    /// An error type returned when attempting to apply an invalid move.
    type MoveError: Clone + Debug;

    /// Returns the player whose turn it is in this state.
    fn current_player(&self) -> MinMaxPlayer;

    /// Returns the list of legal moves from this position.
    ///
    /// Must be non-empty whenever `outcome()` is `None`.
    fn legal_moves(&self) -> Vec<Self::Move>;

    /// Applies the given move to this state, producing the next state.
    ///
    /// # Errors
    ///
    /// Implementations should return `Err(Self::MoveError)` if the move is
    /// illegal in the current state.
    fn with_move(&self, mv: &Self::Move) -> Result<Self, Self::MoveError>;

    /// Returns the current outcome if the game is over, or `None` if play can continue.
    ///
    /// Contract with `legal_moves`:
    /// - If this returns `None`, then `legal_moves()` must return at least one move.
    /// - If there are no legal moves, this must return `Some(...)` (terminal state).
    fn outcome(&self) -> Option<Outcome>;

    /// Returns a heuristic evaluation of this state from the perspective of
    /// [`MinMaxPlayer::Max`].
    ///
    /// This is used by the minimax algorithm when:
    ///
    /// - The search depth limit has been reached, or
    /// - Certain fallback conditions are met (for example, as a last resort
    ///   if an implementation violates its own contracts).
    ///
    /// # Interpretation
    ///
    /// - Positive values are better for [`MinMaxPlayer::Max`].
    /// - Negative values are better for [`MinMaxPlayer::Min`].
    /// - Zero typically represents an even or neutral position.
    fn evaluate(&self) -> i32;
}