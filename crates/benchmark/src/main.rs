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

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
    pub fn from_config(config: RunConfig) -> Self {
        RunResult {
            total_games: config.games,
            config,
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

    for x in 0..options.games {
        let starting_player = get_starting_player(x);
        let stats = play_one_game(&run_config, starting_player);
        aggregate_results(&mut run_result, &stats);
        run_result.games.push(stats);
        aggregate_totals(&mut run_result);
    }
    serialize_to_file(&options.output, &run_result);
}

fn get_starting_player(x: u32) -> MinMaxPlayer {
    match x % 2 == 0 {
        true => MinMaxPlayer::Max,
        false => MinMaxPlayer::Min,
    }
}

fn serialize_to_file(file: &String, run_result: &RunResult) {
    let out = std::fs::File::create(file).unwrap();
    serde_json::to_writer_pretty(out, &run_result).unwrap();
}

fn aggregate_totals(run_result: &mut RunResult) {
    let mut total: (u128, u64, u64) = (0, 0, 0);
    run_result
        .games
        .iter()
        .for_each(|x| {
            total.0 += x.total_millis;
            total.1 += x.total_nodes;
            total.2 += x.moves as u64;
        });
    run_result.avg_millis_per_move = total.0 as f64 / total.2 as f64;
    run_result.avg_nodes_per_move = total.1 as f64 / total.2 as f64;
    run_result.total_millis = total.0 as f64;
}

fn aggregate_results(run_result: &mut RunResult, stats: &GameStats) {
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
        let search_result = {
            if let Some(_) = cfg.time_ms {
                ai::iterative_minimax(&game, &search_cfg)
            } else {
                ai::minimax(&game, &search_cfg)
            }
        };
        let mv = search_result.best_move.expect("must have a move");
        game = game.with_move(&mv).unwrap();

        moves += 1;
        total_nodes += search_result.nodes_visited;
        total_millis += search_result.millis_spent;
    }
    let winner = get_outcome_str(&game.outcome());

    GameStats {
        winner,
        moves,
        total_nodes,
        total_millis,
    }
}

fn get_outcome_str(outcome: &Option<Outcome> ) -> Option<String> {
    match outcome.unwrap() {
        Outcome::Win(player) => match player {
            MinMaxPlayer::Max => Some("Max".to_string()),
            MinMaxPlayer::Min => Some("Min".to_string()),
        },
        Outcome::Draw => Some("Draw".to_string()),
    }
}

/// Parse command line arguments into `Options`.
fn parse_options() -> Options {
    Options::parse()
}

/// Parse heuristic argument into `HeuristicVersion`.
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
    sc.time_ms = config.time_ms;
    sc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heuristics() {
        assert_eq!(parse_heuristic("v1"), HeuristicVersion::V1, "v1 should result HeuristicVersion::V1");
        assert_eq!(parse_heuristic("v2"), HeuristicVersion::V2, "v2 should result HeuristicVersion::V2");
    }

    #[test]
    fn test_build_run_config() {
        let options= Options {
            games: 50,
            depth: 10,
            time_ms: Some(50),
            output: "my_output_file.json".to_string(),
            alpha_beta: true,
            heuristic_min: "v1".to_string(),
            heuristic_max: "v2".to_string(),
        };
        let run_config = RunConfig::from(&options);

        assert_eq!(run_config.games, 50);
        assert_eq!(run_config.depth, 10);
        assert_eq!(run_config.time_ms, Some(50));
        assert_eq!(run_config.heuristic_min, HeuristicVersion::V1);
        assert_eq!(run_config.heuristic_max, HeuristicVersion::V2);
        assert_eq!(run_config.alpha_beta, true);
    }

    #[test]
    fn test_outcome() {
        let mut outcome = Some(Outcome::Win(MinMaxPlayer::Max));
        let mut outcome_str = get_outcome_str(&outcome);
        assert_eq!(outcome_str.unwrap(), "Max");

        outcome = Some(Outcome::Win(MinMaxPlayer::Min));
        outcome_str = get_outcome_str(&outcome);
        assert_eq!(outcome_str.unwrap(), "Min");

        outcome = Some(Outcome::Draw);
        outcome_str = get_outcome_str(&outcome);
        assert_eq!(outcome_str.unwrap(), "Draw")
    }

    #[test]
    fn test_play_one_game_normal() {
        let options = Options {
            games: 1,
            depth: 6,
            time_ms: None,
            output: "my_output_file.json".to_string(),
            alpha_beta: true,
            heuristic_min: "v1".to_string(),
            heuristic_max: "v2".to_string(),
        };
        let run_config = RunConfig::from(&options);
        let game_stats = play_one_game(&run_config, MinMaxPlayer::Max);
        assert!(game_stats.winner.is_some(), "We should have a result");
        assert!(game_stats.moves > 0, "We should have more than 0 moves");
        assert!(game_stats.total_nodes > 0, "We should have more than 0 nodes visited");
    }

    #[test]
    fn test_play_one_game_iterative_deepening() {
        let options = Options {
            games: 1,
            depth: 100,
            time_ms: Some(50),
            output: "my_output_file.json".to_string(),
            alpha_beta: true,
            heuristic_min: "v2".to_string(),
            heuristic_max: "v2".to_string(),
        };
        let run_config = RunConfig::from(&options);
        let game_stats = play_one_game(&run_config, MinMaxPlayer::Max);
        assert!(game_stats.winner.is_some(), "We should have a result");
        assert!(game_stats.moves > 0, "We should have more than 0 moves");
        assert!(game_stats.total_nodes > 0, "We should have more than 0 nodes visited");
    }

    #[test]
    fn test_run_result() {
        let options = Options {
            games: 1,
            depth: 100,
            time_ms: Some(50),
            output: "my_output_file.json".to_string(),
            alpha_beta: true,
            heuristic_min: "v2".to_string(),
            heuristic_max: "v2".to_string(),
        };
        let run_config = RunConfig::from(&options);
        let run_result = RunResult::from_config(RunConfig::from(&options));

        assert_eq!(run_result.total_games, options.games);
        assert_eq!(run_result.config, run_config);
    }

    #[test]
    fn test_starting_player() {
        assert_eq!(MinMaxPlayer::Max,get_starting_player(0), "Even player should be Max");
        assert_eq!(MinMaxPlayer::Min,get_starting_player(1), "Odd player should be Min");
    }
}

