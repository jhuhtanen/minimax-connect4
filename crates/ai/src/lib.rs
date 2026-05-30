use std::time::Instant;
pub use crate::player::MinMaxPlayer;
pub use crate::player::Player;
pub use crate::game_state::GameState;
pub use crate::outcome::Outcome;

mod game_state;
mod outcome;
mod player;


/// Result of a minimax (or alpha-beta) search from a given game state.
///
/// `SearchResult` captures:
/// - the chosen move (if any),
/// - the evaluated score of the root position,
/// - and some diagnostics about the search (e.g. nodes visited).
///
/// The `score` is always from the perspective of the maximizing player
pub struct SearchResult<M> {
    /// The best move found for the side to move at the root.
    ///
    /// This is `None` if the position is terminal (no legal moves).
    pub best_move: Option<M>,
    /// Evaluation score of the root position from the maximizing player's perspective.
    ///
    /// The exact scale depends on the game and evaluation function, but typically:
    /// - positive values: good for the maximizing player,
    /// - negative values: good for the minimizing player,
    /// - zero: equal / drawish position.
    pub score: i32,
    /// Total number of game states (nodes) visited during the search.
    ///
    /// This can be used for debugging, performance analysis, and to compare
    /// plain minimax vs alpha-beta pruning.
    pub nodes_visited: u64,
    /// Time spent in this search call, in milliseconds.
    pub millis_spent: u128,
    /// The search depth used for this result (for iterative deepening, this is the last completed depth).
    pub depth_reached: u32,
}

/// Configuration parameters for a minimax (or alpha-beta) search.
///
/// `SearchConfig` controls how deep the search goes and whether alpha-beta
/// pruning is enabled. It also carries the initial alpha/beta bounds when
/// pruning is used.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Maximum search depth from the root position.
    ///
    /// A depth of `0` means: do not look ahead at all, simply evaluate
    /// the current position. A depth of `1` means: evaluate all immediate
    /// children, and so on.
    pub depth : u32,
    /// Initial alpha bound (best score guaranteed so far for the maximizing player).
    ///
    /// When `use_alpha_beta` is `false`, this value is typically ignored.
    /// When `use_alpha_beta` is `true`, this is the starting alpha passed
    /// to the root call. Common defaults are `i32::MIN` at the root.
    pub alpha : i32,
    /// Initial beta bound (best score guaranteed so far for the minimizing player).
    ///
    /// When `use_alpha_beta` is `false`, this value is typically ignored.
    /// When `use_alpha_beta` is `true`, this is the starting beta passed
    /// to the root call. Common defaults are `i32::MAX` at the root.
    pub beta : i32,
    /// Whether to use alpha-beta pruning during the search.
    ///
    /// - If `false`, a plain minimax search is performed.
    /// - If `true`, the search will prune branches where `alpha >= beta`.
    pub use_alpha_beta : bool,
}

impl SearchConfig {
    pub fn new(depth: u32) -> Self {
        SearchConfig {
            depth,
            alpha: i32::MIN,
            beta: i32::MAX,
            use_alpha_beta: false
        }
    }

    pub fn new_alpha_beta(depth: u32) -> Self {
        SearchConfig {
            depth,
            alpha: i32::MIN,
            beta: i32::MAX,
            use_alpha_beta: true
        }
    }

    pub fn with_alpha_beta(mut self, use_alpha_beta: bool, alpha: i32, beta: i32) -> Self {
        self.alpha = alpha;
        self.beta = beta;
        self.use_alpha_beta = use_alpha_beta;
        self
    }
}

const WIN_SCORE:  i32 = 1_000_000;
const LOSS_SCORE: i32 = -1_000_000;

pub fn minimax<G: GameState>(state: &G, config: &SearchConfig) -> SearchResult<G::Move> {
    let start = Instant::now();

    fn inner<G: GameState>(state: &G,
                            config: &SearchConfig) -> (Option<G::Move>, i32, u64) {
        // game has ended (terminal)
        if let Some(outcome) = state.outcome() {
            let score = match outcome {
                Outcome::Win(min_max_player) => match min_max_player {
                    MinMaxPlayer::Max => WIN_SCORE,
                    MinMaxPlayer::Min => LOSS_SCORE
                }
                Outcome::Draw => 0,
            };
            return (None, score, 1);
        }
        // it hasn't ended, but we reached the max search depth
        if config.depth == 0 {
            return (None, state.evaluate(), 1);
        }

        let moves = state.legal_moves();
        if moves.is_empty() {
            // Per the GameState contract, this should never happen:
            // non-terminal state with no legal moves.
            unreachable!("GameState contract violated: non-terminal state with no legal moves. \
            This should not happen.");
        }

        let mut best_move = None;
        let mut best_score;
        let mut nodes = 1; // start with current node

        match state.current_player() {
            MinMaxPlayer::Max => {
                best_score = i32::MIN;
                let mut current_alpha = config.alpha;
                for mv in moves {
                    let child = state.with_move(&mv).unwrap();
                    let new_config = SearchConfig::new(config.depth - 1)
                        .with_alpha_beta(config.use_alpha_beta, current_alpha, config.beta);
                    let (_, score, child_nodes) = inner(&child, &new_config);
                    nodes += child_nodes;
                    if score > best_score {
                        best_score = score;
                        best_move = Some(mv);
                    }
                    current_alpha = current_alpha.max(score);
                    if config.use_alpha_beta && current_alpha >= config.beta {
                        break;
                    }
                }
            }
            MinMaxPlayer::Min => {
                best_score = i32::MAX;
                let mut current_beta = config.beta;
                for mv in moves {
                    let child = state.with_move(&mv).unwrap();
                    let new_config = SearchConfig::new(config.depth - 1)
                        .with_alpha_beta(config.use_alpha_beta, config.alpha, current_beta);
                    let (_, score, child_nodes) = inner(&child, &new_config);
                    nodes += child_nodes;
                    if score < best_score {
                        best_score = score;
                        best_move = Some(mv);
                    }
                    current_beta = current_beta.min(score);
                    if config.use_alpha_beta && current_beta < config.alpha {
                        break;
                    }
                }
            }
        }
        (best_move, best_score, nodes)
    }

    let (best_move, score, nodes_visited) = inner(state, config);
    SearchResult { best_move, score, nodes_visited,
        millis_spent: start.elapsed().as_millis(), depth_reached: config.depth }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Eq, PartialEq)]
    enum MockMove {
        Left, Right
    }

    #[derive(Debug, Clone)]
    struct MockMoveError {}
    #[derive(Debug, Clone)]
    struct MockState {
        current_player: MinMaxPlayer,
        remaining_depth: u32,
        base_score: i32,
    }

    impl MockState {
        pub fn from(current_player: MinMaxPlayer, remaining_depth: u32, base_score: i32) -> Self {
            MockState {
                current_player,
                remaining_depth,
                base_score
            }
        }
    }

    impl GameState for MockState
    {
        type Move = MockMove;
        type MoveError = MockMoveError;

        fn current_player(&self) -> MinMaxPlayer {
            self.current_player
        }

        fn legal_moves(&self) -> Vec<Self::Move> {
            if self.remaining_depth == 0 {
                vec![]
            } else {
                vec![MockMove::Left, MockMove::Right]
            }
        }

        fn with_move(&self, mv: &Self::Move) -> Result<Self, Self::MoveError> {
            let mut state = self.clone();
            state.remaining_depth -= 1;
            state.current_player = state.current_player.opponent();
            match mv {
                MockMove::Left  => state.base_score += 1, // good for Max
                MockMove::Right => state.base_score -= 1, // good for Min
            }
            Ok(state)
        }

        fn outcome(&self) -> Option<Outcome> {
            // still exploring to do
            if self.remaining_depth > 0 {
                None
            } else { // done with exploring
                if self.base_score > 0 {
                    Some(Outcome::Win(MinMaxPlayer::Max))
                } else if self.base_score < 0 {
                    Some(Outcome::Win(MinMaxPlayer::Min))
                } else {
                    Some(Outcome::Draw)
                }
            }
        }

        fn evaluate(&self) -> i32 {
            if let Some(outcome) = self.outcome() {
                match outcome {
                    Outcome::Win(min_max_player)  => match min_max_player {
                        MinMaxPlayer::Max => 1,
                        MinMaxPlayer::Min => -1,
                    }
                    Outcome::Draw => 0,
                }
            } else {
                0 // non-terminal neutral
            }
        }
    }

    #[test]
    fn default_search_config() {
        let context = SearchConfig::new(3);

        assert_eq!(context.depth, 3, "depth should be 3");
        assert_eq!(context.alpha, i32::MIN, "alpha should be {}", i32::MIN);
        assert_eq!(context.beta, i32::MAX, "beta should be {}", i32::MAX);
        assert_eq!(context.use_alpha_beta, false, "use_alpha_beta should be false");
    }

    #[test]
    fn search_config_with_alpha_beta_enabled() {
        let mut context = SearchConfig::new(1);
        context = context.with_alpha_beta(true, 15, -25);

        assert_eq!(context.depth, 1, "depth should be 1");
        assert_eq!(context.alpha, 15, "alpha should be {}", 15);
        assert_eq!(context.beta, -25, "beta should be {}",-25);
        assert_eq!(context.use_alpha_beta, true, "use_alpha_beta should be true");
    }


    #[test]
    fn depth_zero_max_wins() {
        let state = MockState::from(MinMaxPlayer::Max, 0, 2);
        let mv = minimax(&state, &SearchConfig::new(0));
        assert!(state.outcome().is_some());
        assert_eq!(mv.score, WIN_SCORE, "Max winning, score should have been {}", WIN_SCORE);
        assert_eq!(state.outcome().unwrap(), Outcome::Win(MinMaxPlayer::Max), "Max should have won at 0 depth")
    }

    #[test]
    fn depth_zero_min_wins() {
        let state = MockState::from(MinMaxPlayer::Max, 0, -2);
        let mv = minimax(&state, &SearchConfig::new(0));
        assert!(state.outcome().is_some());
        assert_eq!(mv.score, LOSS_SCORE, "Min winning, score should have been {}", LOSS_SCORE);
        assert_eq!(state.outcome().unwrap(), Outcome::Win(MinMaxPlayer::Min), "Min should have won at 0 depth")
    }

    #[test]
    fn depth_zero_draw() {
        let state = MockState::from(MinMaxPlayer::Max, 0, 0);
        let mv = minimax(&state, &SearchConfig::new(0));
        assert!(state.outcome().is_some());
        assert_eq!(mv.score, 0, "Draw score should have been 0");
        assert_eq!(state.outcome().unwrap(), Outcome::Draw, "Game should have ended in draw")
    }

    #[test]
    fn depth_zero_non_terminal() {
        let state = MockState::from(MinMaxPlayer::Max, 1, 0);
        let mv = minimax(&state, &SearchConfig::new(0));
        assert_eq!(mv.score, 0, "Non terminal score should have been 0");
        assert!(state.outcome().is_none());
    }

    #[test]
    fn depth_one_max_should_play_left() {
        let mut state = MockState::from(MinMaxPlayer::Max, 1, 0);
        let mv = minimax(&state, &SearchConfig::new(1));
        assert!(mv.best_move.is_some());
        assert_eq!(mv.best_move.unwrap(), MockMove::Left, "Max should have chosen left");
        assert_eq!(mv.score, WIN_SCORE, "Score for playing left should have been {}", WIN_SCORE);
        assert!(state.outcome().is_none());
    }

    #[test]
    fn depth_one_max_should_play_left_with_pruning() {
        let state = MockState::from(MinMaxPlayer::Max, 1, 0);
        let mv = minimax(&state, &SearchConfig::new_alpha_beta(1));
        assert_eq!(mv.best_move.unwrap(), MockMove::Left, "Max should have chosen left");
        assert_eq!(mv.score, WIN_SCORE, "Score for playing left should have been {}", WIN_SCORE);
    }

    #[test]
    fn depth_one_min_should_play_right() {
        let state = MockState::from(MinMaxPlayer::Min, 1, 0);
        let mv = minimax(&state, &SearchConfig::new(1));
        assert!(mv.best_move.is_some());
        assert_eq!(mv.best_move.unwrap(), MockMove::Right, "Min should have chosen right");
        assert_eq!(mv.score, LOSS_SCORE, "Score for playing right should have been {}", LOSS_SCORE);
        assert!(state.outcome().is_none());
    }

    #[test]
    fn depth_one_min_should_play_right_with_pruning() {
        let state = MockState::from(MinMaxPlayer::Min, 1, 0);
        let mv = minimax(&state, &SearchConfig::new_alpha_beta(1));
        assert_eq!(mv.best_move.unwrap(), MockMove::Right, "Min should have chosen right");
        assert_eq!(mv.score, LOSS_SCORE, "Score for playing right should have been {}", LOSS_SCORE);
    }

    #[test]
    fn depth_two_max_should_play_left_with_higher_min_score() {
        let state = MockState::from(MinMaxPlayer::Max, 2, 0);
        let mv = minimax(&state, &SearchConfig::new(2));
        assert!(mv.best_move.is_some());
        assert_eq!(mv.best_move.unwrap(), MockMove::Left, "Max should have chosen left");
        assert_eq!(mv.score, 0, "Score for playing left should have been 0");
        assert!(state.outcome().is_none());
    }

    #[test]
    fn depth_two_max_should_play_left_with_higher_min_score_with_pruning() {
        let state = MockState::from(MinMaxPlayer::Max, 2, 0);
        let mv = minimax(&state, &SearchConfig::new_alpha_beta(2));
        assert_eq!(mv.best_move.unwrap(), MockMove::Left, "Max should have chosen left");
        assert_eq!(mv.score, 0, "Score for playing left should have been 0");
    }

    #[test]
    fn alpha_beta_and_plain_return_same_score_for_mock_state() {
        let state = MockState::from(MinMaxPlayer::Max, 4, 0);

        for depth in 0..=4 {
            let no_pruning = minimax(&state, &SearchConfig::new(depth));
            let pruning    = minimax(&state, &SearchConfig::new_alpha_beta(depth));

            assert_eq!(no_pruning.score, pruning.score, "scores should be same at  depth = {}", depth);
            assert!(pruning.nodes_visited <= no_pruning.nodes_visited, "No pruning should have visited more nodes");
        }
    }

    #[test]
    fn test_alpha_beta_pruning_gives_same_score() {
        let mut no_pruning_state = MockState::from(MinMaxPlayer::Max, 4, 0);
        let mut pruning_state = MockState::from(MinMaxPlayer::Max, 4, 0);
        let no_pruning_config = SearchConfig::new(9);
        let pruning_config = SearchConfig::new_alpha_beta(9);
        while no_pruning_state.outcome().is_none() && pruning_state.outcome().is_none() {
            let no_pruning_result = minimax(&no_pruning_state, &no_pruning_config);
            let pruning_result = minimax(&pruning_state, &pruning_config);
            let no_pruning_move = no_pruning_result.best_move.expect("must have a move");
            let pruning_move = pruning_result.best_move.expect("must have a move");
            assert_eq!(pruning_move, no_pruning_move, "Moves should be the same");
            no_pruning_state = no_pruning_state.with_move(&no_pruning_move).unwrap();
            pruning_state = pruning_state.with_move(&pruning_move).unwrap();
        }
        assert_eq!(no_pruning_state.outcome(), pruning_state.outcome(), "Outcomes should be the same");
    }
}
