use std::cmp::PartialEq;
use std::collections::HashMap;
use std::io::{self, Write};
use std::str::FromStr;
use std::thread;
use ai::{minimax, GameState, MinMaxPlayer, Outcome, SearchConfig};
use engine::constants::{BOARD_HEIGHT, BOARD_WIDTH};
use engine::game_state::{ConnectFourState, HeuristicVersion};
use engine::moves::Move;

pub const ANSI_RESET: &str = "\u{001B}[0m";
pub const ANSI_RED: &str = "\u{001B}[31m";
pub const ANSI_YELLOW: &str = "\u{001B}[33m";
pub const FILLED_TOKEN: &str = "●";
pub const EMPTY_TOKEN: &str = "○";


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum State {
    Settings,
    Running,
    Done
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
enum PlayerColor {
    Red,
    Yellow
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum PlayerType {
    Human,
    AI
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum AiMode {
    FixedDepth,
    TimeLimited
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct AiSetting {
    pub color : PlayerColor,
    pub player: MinMaxPlayer,
    pub heuristic: HeuristicVersion,
}

#[derive(Debug)]
struct GameSettings {
    pub color_to_type: HashMap<PlayerColor, PlayerType>,
    pub minimax_to_player: HashMap<MinMaxPlayer, PlayerColor>,
    pub ai_setting: HashMap<MinMaxPlayer, AiSetting>,
    pub search_config: SearchConfig,
}

impl GameSettings {
    pub fn default() -> GameSettings {
        GameSettings {
            color_to_type: HashMap::new(),
            minimax_to_player: HashMap::new(),
            ai_setting: HashMap::new(),
            search_config: SearchConfig::new_alpha_beta(6),
        }
    }
}
struct UiState {
    error_message: Option<String>,
}

impl UiState {
    fn new() -> Self {
        Self { error_message: None }
    }
}

fn main() {
    let mut game = ConnectFourState::new(MinMaxPlayer::Max);
    let mut state = State::Settings;
    let mut game_settings = GameSettings::default();
    let mut ui = UiState::new();

    while state != State::Done {
        match state {
            State::Settings => {
                handle_settings_state(&mut state, &mut game_settings);
                apply_game_settings(&mut game, &game_settings);
            }
            State::Running => {
                handle_running_state(&mut game, &mut ui, &mut state, &game_settings);
            }
            _ => {}
        }
    }

}

fn apply_game_settings(game: &mut ConnectFourState, game_settings: &GameSettings) {
    if game_settings.color_to_type[&PlayerColor::Red] == PlayerType::AI {
        let version = game_settings
            .ai_setting
            .get(&MinMaxPlayer::Max)
            .unwrap()
            .clone();
        game.set_player_heuristic(MinMaxPlayer::Max, version.heuristic);
    }
    if game_settings.color_to_type[&PlayerColor::Yellow] == PlayerType::AI {
        let version = game_settings
            .ai_setting
            .get(&MinMaxPlayer::Min)
            .unwrap()
            .clone();
        game.set_player_heuristic(MinMaxPlayer::Min, version.heuristic);
    }
}

fn handle_settings_state(state: &mut State, game_settings: &mut GameSettings) {
    println!("Connect Four");

    let mut red_player = None;
    let mut yellow_player = None;
    let mut red_heuristic = None;
    let mut yellow_heuristic = None;
    let mut ai_mode = None;
    let mut time_ms : Option<u64> = None;
    while red_player.is_none() || yellow_player.is_none() {
        println!("Select Red player: [0 = Human], [1 = AI]");
        if let Some(red) = read_column() {
            red_player = match red {
                0 => Some(PlayerType::Human),
                1 => Some(PlayerType::AI),
                _ => None
            };
        }
        println!("Select Yellow player: [0 = Human], [1 = AI]");
        if let Some(yellow) = read_column() {
            yellow_player = match yellow {
                0 => Some(PlayerType::Human),
                1 => Some(PlayerType::AI),
                _ => None
            };
        }
    };
    if red_player == Some(PlayerType::AI) {
        while red_heuristic.is_none() {
            println!("Select Red player heuristic: [0 = v1], [1 = v2]");
            if let Some(red) = read_column() {
                red_heuristic = match red {
                    0 => Some(HeuristicVersion::V1),
                    1 => Some(HeuristicVersion::V2),
                    _ => None
                };
            }
        }
    }
    if yellow_player == Some(PlayerType::AI) {
        while yellow_heuristic.is_none() {
            println!("Select Yellow player heuristic: [0 = v1], [1 = v2]");
            if let Some(red) = read_column() {
                yellow_heuristic = match red {
                    0 => Some(HeuristicVersion::V1),
                    1 => Some(HeuristicVersion::V2),
                    _ => None
                };
            }
        }
    }
    if yellow_player == Some(PlayerType::AI) || red_player == Some(PlayerType::AI) {
        while ai_mode.is_none() {
            println!("AI mode: [0 = Fixed Depth], [1 = Time Limited]");
            if let Some(col) = read_column() {
                ai_mode = match col {
                    0 => Some(AiMode::FixedDepth),
                    1 => Some(AiMode::TimeLimited),
                    _ => None
                };
            }
        }
    }
    if ai_mode == Some(AiMode::TimeLimited) {
        while time_ms.is_none() {
            println!("Time limit: (ms)");
            if let Some(col) = read_value::<u64>() {
                time_ms = Some(col);
                game_settings.search_config.depth = 9;
            };
        }
    }
    game_settings.color_to_type.insert(PlayerColor::Red, red_player.unwrap_or(PlayerType::Human));
    game_settings.color_to_type.insert(PlayerColor::Yellow, yellow_player.unwrap_or(PlayerType::Human));
    if red_player == Some(PlayerType::AI) {
        game_settings.ai_setting.insert(MinMaxPlayer::Max,
                                        AiSetting { color: PlayerColor::Red, player: MinMaxPlayer::Max, heuristic: red_heuristic.unwrap() });
    }
    game_settings.minimax_to_player.insert(MinMaxPlayer::Max, PlayerColor::Red);
    if yellow_player == Some(PlayerType::AI) {
        game_settings.ai_setting.insert(MinMaxPlayer::Min,
                                        AiSetting { color: PlayerColor::Yellow, player: MinMaxPlayer::Min, heuristic: yellow_heuristic.unwrap() });
    }
    game_settings.minimax_to_player.insert(MinMaxPlayer::Min, PlayerColor::Yellow);
    game_settings.search_config.time_ms = time_ms;

    *state = State::Running;
}

fn handle_running_state(game: &mut ConnectFourState, ui: &mut UiState, state: &mut State, game_settings: &GameSettings) {
    redraw_screen(&game, &ui);

    if let Some(outcome) = game.outcome() {
        match outcome {
            Outcome::Win(winner) => {
                let player_color = game_settings.minimax_to_player[&winner];
                let player_type = game_settings.color_to_type[&player_color];
                let type_presentation= match player_type {
                    PlayerType::Human => {
                        format!("{:?}", player_type)
                    },
                    PlayerType::AI => {
                        format!("{:?}, heuristic: {:?}", player_type, game_settings.ai_setting.get(&winner).unwrap().heuristic)
                    }
                };

                println!("Player {:?} ({:?}) wins!", represent_player(&winner, &game_settings), type_presentation);
            },
            Outcome::Draw => {
                println!("It's a draw!");
            }
        };
        wait_for_enter();
        handle_state_change(state);
        return;
    }

    println!("Player {}'s ({:?}) turn.", represent_player(&game.current_player(), &game_settings),
             game_settings.minimax_to_player[&game.current_player()]);
    println!(
        "Available columns: {:?}",
        game
            .legal_moves()
            .iter()
            .map(|m| m.column())
            .collect::<Vec<_>>()
    );
    // clear previous error
    ui.error_message = None;
    // crate move based on the current player
    let player_color = game_settings.minimax_to_player[&game.current_player()];
    let mv = match game_settings.color_to_type[&player_color] {
        PlayerType::Human => {
            let col = match prompt_column_inline() {
                Some(c) => c,
                None => {
                    ui.error_message = Some("Exiting game.".to_string());
                    redraw_screen(&game, &ui);
                    handle_state_change(state);
                    return;
                }
            };

            let mv = match Move::new(col) {
                Ok(m) => m,
                Err(_) => {
                    ui.error_message = Some(format!("Column {} is out of bounds. Try again.", col));
                    return;
                }
            };
            mv
        },
        PlayerType::AI => {
            println!("color {:?}, {:?}", player_color, game_settings.ai_setting.get(&game.current_player()).unwrap());
            println!("game: {:?}, {:?}", game.player_heuristic[0], game.current_player);
            let result = if game_settings.search_config.time_ms.is_some() {
                ai::iterative_minimax(game, &game_settings.search_config)
            } else {
                ai::minimax(game, &game_settings.search_config)
            };
            //let result = minimax(game, &game_settings.search_config);
            result.best_move.unwrap()
        }
    };
    let result = game.with_move(&mv);

    *game = match result {
        Ok(new_state) => new_state,
        Err(e) => {
            ui.error_message = Some(format!("Cannot play in column {}: {e:?}", mv.column()));
            return;
        }
    };
    if both_players_ai(&game_settings) {
        thread::sleep(core::time::Duration::from_millis(100));
    }
}

fn both_players_ai(game_settings: &GameSettings) -> bool {
    game_settings.color_to_type[&PlayerColor::Red] == PlayerType::AI &&
        game_settings.color_to_type[&PlayerColor::Yellow] == PlayerType::AI
}

fn handle_state_change(current_state: &mut State) {
    *current_state = match *current_state {
        State::Settings => State::Running,
        State::Running => State::Done,
        State::Done => State::Settings,
    };
}

fn redraw_screen(game: &ConnectFourState, ui_state: &UiState) {
    // Clear screen and move cursor to top-left
    print!("\x1b[2J\x1b[H");
    print_board(game);
    print_status(ui_state);
    io::stdout().flush().ok();
}

fn represent_player(player: &MinMaxPlayer, game_settings: &GameSettings) -> String {
    match game_settings.minimax_to_player[player] {
        PlayerColor::Yellow => String::from("Yellow"),
        PlayerColor::Red => String::from("Red"),
    }
}

fn prompt_column_inline() -> Option<u8> {
    print!("\nEnter column (0-{}), or 'q' to quit: ", BOARD_WIDTH - 1);
    read_column()
}

fn read_column() -> Option<u8> {
    read_value::<u8>()
}
fn read_value<T>() -> Option<T> where
    T: FromStr, {
    io::stdout().flush().ok();

    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return None;
    }
    let s = line.trim();
    if s.eq_ignore_ascii_case("q") {
        return None;
    }
    s.parse::<T>().ok()
}

fn wait_for_enter() {
    println!("\nPress Enter to exit...");
    _ = io::stdin().read_line(&mut String::new());
}

fn print_board(game: &ConnectFourState) {
    println!("Connect Four");
    println!("+{}+", "-".repeat(BOARD_WIDTH as usize));

    for row in (0..BOARD_HEIGHT).rev() {
        print!("|");
        for col in 0..BOARD_WIDTH {
            let ch = match game.token_at(col, row) {
                Some(MinMaxPlayer::Max) => format!("{}{}{}", ANSI_RED, FILLED_TOKEN, ANSI_RESET),
                Some(MinMaxPlayer::Min) => format!("{}{}{}", ANSI_YELLOW, FILLED_TOKEN, ANSI_RESET),
                None => format!("{}{}", ANSI_RESET, EMPTY_TOKEN)
            };
            print!("{ch}");
        }
        println!("|");
    }

    println!("+{}+", "-".repeat(BOARD_WIDTH as usize));
    print!(" ");
    for col in 0..BOARD_WIDTH {
        print!("{col}");
    }
    println!();
}

fn print_status(ui_state: &UiState) {
    println!();
    if let Some(ref error_message) = ui_state.error_message {
        print!("Error: {}", error_message);
    }
    else {
        println!()
    }
}
