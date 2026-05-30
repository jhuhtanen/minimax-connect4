use serde::{Deserialize, Serialize};
use crate::{MinMaxPlayer};

/// The outcome of a fully resolved game state.
///
/// This is returned by [`GameState::outcome`] when the game is over.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// One of the players has won the game.
    Win (MinMaxPlayer),
    Draw
}
