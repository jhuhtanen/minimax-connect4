use std::cmp::PartialEq;
use std::collections::HashMap;
use std::io::{self, Write};
use ai::{minimax, GameState, MinMaxPlayer, Outcome, SearchConfig};
use engine::constants::{BOARD_HEIGHT, BOARD_WIDTH};
use engine::game_state::{ConnectFourState};
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum PlayerType {
    Human,
    AI
}

#[derive(Debug)]
struct GameSettings {
    pub player_types: HashMap<MinMaxPlayer, PlayerType> ,
    pub search_config: SearchConfig,
}

impl GameSettings {
    pub fn default() -> GameSettings {
        GameSettings {
            player_types: HashMap::new(),
            search_config: SearchConfig::new_alpha_beta(5)
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
                handle_settings_state(&mut game, &mut ui, &mut state, &mut game_settings);
            }
            State::Running => {
                handle_running_state(&mut game, &mut ui, &mut state, &game_settings);
            }
            _ => {}
        }
    }

}

fn handle_settings_state(game: &mut ConnectFourState, ui: &mut UiState, state: &mut State, game_settings: &mut GameSettings) {
    println!("Connect Four");

    let mut red_player = None;
    let mut yellow_player = None;
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
    game_settings.player_types.insert(MinMaxPlayer::Max, red_player.unwrap_or(PlayerType::Human));
    game_settings.player_types.insert(MinMaxPlayer::Min, yellow_player.unwrap_or(PlayerType::Human));
    *state = State::Running;
}

fn handle_running_state(game: &mut ConnectFourState, ui: &mut UiState, state: &mut State, game_settings: &GameSettings) {
    redraw_screen(&game, &ui);

    if let Some(outcome) = game.outcome() {
        match outcome {
            Outcome::Win(winner) => {
                println!("Player {:?} ({:?}) wins!", represent_player(&winner), game_settings.player_types[&winner]);
            },
            Outcome::Draw => {
                println!("It's a draw!");
            }
        };
        wait_for_enter();
        handle_state_change(state);
        return;
    }

    println!("Player {}'s ({:?}) turn.", represent_player(&game.current_player()),
             game_settings.player_types[&game.current_player()]);
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
    let mv = match game_settings.player_types[&game.current_player()] {
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
            let result = minimax(game, &game_settings.search_config);
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

fn represent_player(player: &MinMaxPlayer) -> String {
    match player {
        &MinMaxPlayer::Max => String::from("Red"),
        &MinMaxPlayer::Min => String::from("Yellow"),
    }
}

fn prompt_column_inline() -> Option<u8> {
    print!("\nEnter column (0-{}), or 'q' to quit: ", BOARD_WIDTH - 1);
    read_column()
}

fn read_column() -> Option<u8> {
    io::stdout().flush().ok();

    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return None;
    }
    let s = line.trim();
    if s.eq_ignore_ascii_case("q") {
        return None;
    }
    s.parse::<u8>().ok()
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
