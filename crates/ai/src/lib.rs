use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};
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
    /// Max time in milliseconds to spend per move. Used in iterative deepening.
    pub time_ms: Option<u64>,
}

impl SearchConfig {
    pub fn new(depth: u32) -> Self {
        SearchConfig {
            depth,
            alpha: i32::MIN,
            beta: i32::MAX,
            use_alpha_beta: false,
            time_ms: None,
        }
    }

    pub fn new_alpha_beta(depth: u32) -> Self {
        SearchConfig {
            depth,
            alpha: i32::MIN,
            beta: i32::MAX,
            use_alpha_beta: true,
            time_ms: None,
        }
    }

    pub fn with_alpha_beta(mut self, use_alpha_beta: bool, alpha: i32, beta: i32) -> Self {
        self.alpha = alpha;
        self.beta = beta;
        self.use_alpha_beta = use_alpha_beta;
        self
    }

    pub fn with_time_ms(mut self, time_ms: Option<u64>) -> Self {
        self.time_ms = time_ms;
        self
    }
}

const WIN_SCORE:  i32 = 1_000_000;
const LOSS_SCORE: i32 = -1_000_000;

/// Searches a game state using iterative deepening Minimax.
///
/// The search is performed repeatedly with increasing depth limits,
/// starting from depth 1 and continuing until the maximum search depth
/// specified in `base_config` is reached.
///
/// A shared transposition table is reused between iterations, allowing
/// information discovered in shallow searches to improve move ordering
/// and reduce the amount of work required in deeper searches.
///
/// # Parameters
///
/// * `state` - Root position to evaluate.
/// * `base_config` - Search configuration containing the maximum search
///   depth and evaluation settings.
///
/// # Returns
///
/// The result of the deepest completed search, including the selected
/// move and its evaluation score.
pub fn iterative_minimax<G: GameState + Eq + Hash>(state: &G,
                            base_config: &SearchConfig, ) -> SearchResult<G::Move> {
    let mut cache: HashMap<G, G::Move> = HashMap::new();
    let start = Instant::now();
    let deadline = base_config
        .time_ms
        .map(|ms| start + Duration::from_millis(ms));

    let mut best_result: Option<SearchResult<G::Move>> = None;
    let mut last_depth_time = Duration::new(0, 0);

    for depth in 1..=base_config.depth {
        // stop if we are out of time
        if let Some(dl) = deadline {
            let now = Instant::now();
            if now >= dl {
                break;
            }
            let remaining = dl - now;
            if last_depth_time > remaining {
                break;
            }
        }
        let mut cfg = base_config.clone();
        cfg.depth = depth;
        let start_depth = Instant::now();
        let result = minimax_with_cache(state, &cfg, &mut cache);
        last_depth_time = start_depth.elapsed();

        best_result = Some(SearchResult {
            depth_reached: depth,
            ..result
        });
    }

    best_result.expect("iterative_minimax: no depth completed")
}

/// Searches the game tree using Minimax with alpha-beta pruning and a
/// transposition table.
///
/// The function evaluates the given game state and returns the best move
/// together with its score. Previously evaluated positions are stored in
/// `cache` and reused when encountered again, reducing the amount of
/// repeated search work.
///
/// The cache maps game states to the best move found for that position and
/// is also used to improve move ordering in subsequent searches.
///
/// # Parameters
///
/// * `state` - Root position to evaluate.
/// * `config` - Search configuration, including depth limits and evaluation settings.
/// * `cache` - Transposition table used for move caching and move ordering.
///
/// # Returns
///
/// A [`SearchResult`] containing the selected move and its evaluation.
fn minimax_with_cache<G: GameState + Eq + Hash>(state: &G, config: &SearchConfig,
                                    cache: &mut HashMap<G, G::Move>, ) -> SearchResult<G::Move>{

    let start = Instant::now();

    fn inner<G: GameState>(state: &G,
                           config: &SearchConfig,
                           cache: &mut HashMap<G, G::Move>,) -> (Option<G::Move>, i32, u64) where G: Eq + Hash {
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

        let mut best_move = None;
        let mut moves = state.legal_moves();
        if moves.is_empty() {
            // Per the GameState contract, this should never happen:
            // non-terminal state with no legal moves.
            unreachable!("GameState contract violated: non-terminal state with no legal moves. \
            This should not happen.");
        }
        // check cache first
        if let Some(move_hint) = cache.get(state) {
            if let Some(pos) = moves
                .iter()
                .position(|m| m == move_hint) {
                moves.swap(0, pos);
            }
        }

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
                    let (_, score, child_nodes) = inner(&child, &new_config, cache);
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
                    let (_, score, child_nodes) = inner(&child, &new_config, cache);
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

    let (best_move, score, nodes_visited) = inner(state, config, cache);
    // this should always have a valid move
    if let Some(ref mv) = best_move {
        cache.insert(state.clone(), mv.clone());
    }
    SearchResult { best_move, score, nodes_visited,
        millis_spent: start.elapsed().as_millis(), depth_reached: config.depth }
}

/// Searches a game tree using the Minimax algorithm with alpha-beta pruning.
///
/// The search explores legal moves from the given game state and assumes
/// that both players play optimally. Terminal positions are evaluated
/// directly, while non-terminal positions are scored using the game's
/// evaluation function when the search depth limit is reached.
///
/// Alpha-beta pruning is used to avoid exploring branches that cannot
/// affect the final decision, reducing the number of evaluated positions
/// compared to a naive Minimax search.
///
/// # Parameters
///
/// * `state` - Root position to evaluate.
/// * `config` - Search configuration containing depth limits and evaluation settings.
///
/// # Returns
///
/// A [`SearchResult`] containing the best move found and its evaluation score.
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

pub fn minimax_pvs<G: GameState>(state: &G, config: &SearchConfig) -> SearchResult<G::Move> {
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
                for (i, mv) in moves.iter().enumerate() {
                    let child = state.with_move(mv).unwrap();
                    let (_, score, child_nodes) = {
                        // for the first do full search
                        if i == 0 {
                            let new_config = create_child_config(&config, current_alpha, config.beta);
                            inner(&child, &new_config)
                        } else { // all the rest try null window
                            let new_config = create_child_config(&config, current_alpha, current_alpha + 1);
                            let (mv, score, child_nodes) = inner(&child, &new_config);
                            // if we didn't find anything interesting, do full search
                            if current_alpha < score && score < config.beta {
                                let new_config = create_child_config(&config, current_alpha, config.beta);
                                inner(&child, &new_config)
                            } else {
                                (mv, score, child_nodes)
                            }
                        }
                    };

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
                for (i, mv) in moves.iter().enumerate() {
                    let child = state.with_move(&mv).unwrap();
                    let (_, score, child_nodes) = {
                        // for the first do full search
                        if i == 0 {
                            let new_config = create_child_config(&config, config.alpha, current_beta);
                            inner(&child, &new_config)
                        } else { // all the rest try null window
                            let new_config = create_child_config(&config, current_beta - 1, current_beta);
                            let (mv, score, child_nodes) = inner(&child, &new_config);
                            // if we didn't find anything interesting, do full search
                            if config.alpha < score && score < current_beta {
                                let new_config = create_child_config(&config, config.alpha, current_beta);
                                inner(&child, &new_config)
                            } else {
                                (mv, score, child_nodes)
                            }
                        }
                    };

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
        (best_move.cloned(), best_score, nodes)
    }

    let (best_move, score, nodes_visited) = inner(state, config);
    SearchResult { best_move, score, nodes_visited,
        millis_spent: start.elapsed().as_millis(), depth_reached: config.depth }
}

fn create_child_config(parent: &SearchConfig, alpha: i32, beta: i32) -> SearchConfig {
    SearchConfig::new(parent.depth - 1)
        .with_alpha_beta(parent.use_alpha_beta, alpha, beta)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Eq, PartialEq)]
    enum MockMove {
        Left,
        Right
    }

    #[derive(Debug, Clone)]
    struct MockMoveError {}
    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
                MockMove::Left => state.base_score += 1, // good for Max
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
                    Outcome::Win(min_max_player) => match min_max_player {
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
        assert_eq!(context.beta, -25, "beta should be {}", -25);
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
        let state = MockState::from(MinMaxPlayer::Max, 1, 0);
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
            let pruning = minimax(&state, &SearchConfig::new_alpha_beta(depth));

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

    #[test]
    fn test_iterative_deepening_with_time_per_move_not_exceeded() {
        let state = MockState::from(MinMaxPlayer::Max, 1, 0);
        let config = SearchConfig::new_alpha_beta(9).with_time_ms(Some(10));
        let start = Instant::now();
        let result = iterative_minimax(&state, &config);
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 10, "One move should take less than 10 millis");
    }

    #[test]
    fn test_iterative_deepening_vs_traditional_suggests_same_move() {
        let iterative_state = MockState::from(MinMaxPlayer::Max, 4, 0);
        let iterative_config = SearchConfig::new_alpha_beta(6).with_time_ms(Some(50));

        let state = MockState::from(MinMaxPlayer::Max, 4, 0);
        let config = SearchConfig::new_alpha_beta(6);

        let iterative_result = iterative_minimax(&iterative_state, &iterative_config);
        let result = minimax(&state, &config);

        assert!(iterative_result.best_move.is_some(), "Should return best move");
        assert!(result.best_move.is_some(), "Should return best move");
        assert_eq!(result.best_move, iterative_result.best_move, "Results should be the same");
    }

    #[test]
    fn test_mock_state_evaluate() {
        let state = MockState::from(MinMaxPlayer::Max, 4, 0);
        assert_eq!(state.evaluate(), 0, "Should be non terminal state");

        let state = MockState::from(MinMaxPlayer::Max, 0, 5);
        assert_eq!(state.evaluate(), 1, "Terminal state should give score 1 for Max");

        let state = MockState::from(MinMaxPlayer::Min, 0, -5);
        assert_eq!(state.evaluate(), -1, "Terminal state should give score -1 for Min");

        let state = MockState::from(MinMaxPlayer::Min, 0, 0);
        assert_eq!(state.evaluate(), 0, "Terminal state should give score 0 for Draw");

    }

    #[test]
    fn test_no_legal_moves_at_zero_depth() {
        let state = MockState::from(MinMaxPlayer::Max, 0, 0);
        assert!(state.legal_moves().is_empty(), "There should be no legal moves");
    }
}
