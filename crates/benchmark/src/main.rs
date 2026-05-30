use clap::Parser;
use serde::{Deserialize, Serialize};
use ai::{GameState, MinMaxPlayer, Outcome};
use engine::game_state::{ConnectFourState, HeuristicVersion};

#[derive(Parser, Debug)]
#[command(name = "benchmark")]
#[command(about = "Connect Four AI benchmark runner")]
struct Options {
    /// Number of games to run
    #[arg(long, default_value_t = 50)]
    games: u32,

    /// Max search depth
    #[arg(long, default_value_t = 6)]
    depth: u32,

    /// Time limit per move in milliseconds (optional)
    #[arg(long)]
    time_ms: Option<u64>,

    /// Output JSON file
    #[arg(long, default_value = "results.json")]
    output: String,

    /// alpha-beta pruning enabled
    #[arg(long, default_value_t = true)]
    alpha_beta: bool,

    /// Heuristic version for Min player
    #[arg(long, default_value = "v1")]
    heuristic_min: String,

    /// Heuristic version for Max player
    #[arg(long, default_value = "v1")]
    heuristic_max: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct RunConfig {
    games: u32,
    depth: u32,
    time_ms: Option<u64>,
    alpha_beta: bool,
    heuristic_min: HeuristicVersion,
    heuristic_max: HeuristicVersion,
}

impl RunConfig {
    fn from(options: &Options) -> Self {
        RunConfig {
            games: options.games,
            depth: options.depth,
            time_ms: options.time_ms,
            alpha_beta: options.alpha_beta,
            heuristic_min: parse_heuristic(&options.heuristic_min),
            heuristic_max: parse_heuristic(&options.heuristic_max),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GameStats {
    winner: Option<String>,
    moves: u32,
    total_nodes: u64,
    total_millis: u128,
}

#[derive(Serialize, Deserialize)]
struct RunResult {
    config: RunConfig,
    total_games: u32,
    max_wins: u32,
    min_wins: u32,
    draws: u32,
    avg_nodes_per_move: f64,
    avg_millis_per_move: f64,
    total_millis: f64,
    games: Vec<GameStats>,
}

impl RunResult {
    pub fn from_config( config: RunConfig) -> Self {
        RunResult {
            config,
            total_games: 0,
            max_wins: 0,
            min_wins: 0,
            draws: 0,
            avg_nodes_per_move: 0.0,
            avg_millis_per_move: 0.0,
            total_millis: 0.0,
            games: Vec::new(),
        }
    }
}

fn main() {
    let options = parse_options();
    let run_config = RunConfig::from(&options);
    let mut run_result = RunResult::from_config(run_config.clone());
    run_result.total_games = options.games;

    for x in 0..options.games {
        let starting_player = match x % 2 {
            0 => MinMaxPlayer::Max,
            1 => MinMaxPlayer::Min,
            _ => unreachable!("Shouldn't be possible to get starting player other than 0 or 1")
        };

        let stats = play_one_game(&run_config, starting_player);
        match stats.winner.clone() {
            Some(player) => {
                match player.as_str() {
                    "Max" => run_result.max_wins += 1,
                    "Min" => run_result.min_wins += 1,
                    "Draw" => run_result.draws += 1,
                    _ => unreachable!("Unrecognised player, should be Min or Max.")
                };
            },
            _ => unreachable!("Unknown winner.")
        }
        run_result.games.push(stats);

        let mut total: (u128, u64, u64) = (0,0,0);
        run_result.games.iter().for_each(|x| {
            total.0 += x.total_millis;
            total.1 += x.total_nodes;
            total.2 += x.moves as u64;
        });
        run_result.avg_millis_per_move = total.0 as f64 / total.2 as f64;
        run_result.avg_nodes_per_move = total.1 as f64 / total.2 as f64;
        run_result.total_millis = total.0 as f64;
    }
    let out = std::fs::File::create(options.output).unwrap();
    serde_json::to_writer_pretty(out, &run_result).unwrap();
}

fn play_one_game(cfg: &RunConfig, starting_player: MinMaxPlayer) -> GameStats {
    let mut game = ConnectFourState::new(starting_player.clone());
    game.set_player_heuristic(MinMaxPlayer::Min, cfg.heuristic_min);
    game.set_player_heuristic(MinMaxPlayer::Max, cfg.heuristic_max);
    let mut total_nodes = 0u64;
    let mut total_millis = 0u128;
    let mut moves = 0u32;

    while game.outcome().is_none() {

        let search_cfg = build_search_config(cfg);

        let search_result = ai::minimax(&game, &search_cfg);
        let mv = search_result.best_move.expect("must have a move");
        game = game.with_move(&mv).unwrap();

        moves += 1;
        total_nodes += search_result.nodes_visited;
        total_millis += search_result.millis_spent;
    }

    let winner = match game.outcome().unwrap() {
        Outcome::Win(player) => match player {
            MinMaxPlayer::Max => Some("Max".to_string()),
            MinMaxPlayer::Min => Some("Min".to_string()),
        },
        Outcome::Draw => Some("Draw".to_string()),
    };

    GameStats {
        winner,
        moves,
        total_nodes,
        total_millis,
    }
}

/// Parse command line arguments into `Options`.
fn parse_options() -> Options {
    Options::parse()
}

fn parse_heuristic(s: &str) -> HeuristicVersion {
    match s {
        "v1" => HeuristicVersion::V1,
        "v2" => HeuristicVersion::V2,
        _    => HeuristicVersion::V1,
    }
}

fn build_search_config(config: &RunConfig) -> ai::SearchConfig {
    let mut sc = ai::SearchConfig::new(config.depth);
    if config.alpha_beta {
        sc = sc.with_alpha_beta(true, i32::MIN, i32::MAX);
    }
    sc
}
