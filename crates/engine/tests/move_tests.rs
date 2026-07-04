
#[cfg(test)]
mod tests {
    use ai::{GameState, MinMaxPlayer, Outcome, SearchConfig};
    use engine::game_state::{ConnectFourState, HeuristicVersion};
    use engine::moves::Move;

    #[test]
    fn test_ai_blocks_immediate_win() {
        //+-------+
        //|.......|
        //|.......|
        //|.......|
        //|.......|
        //|......O|
        //|XXX...O|
        //+-------+
        // 0123456

        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
        game.set_player_heuristic(MinMaxPlayer::Max, HeuristicVersion::V3);
        game.set_player_heuristic(MinMaxPlayer::Min, HeuristicVersion::V3);

        let search_config = SearchConfig::new_alpha_beta(6)
            .with_time_ms(Some(50));

        game = game.with_move(&Move::new(0).unwrap()).unwrap(); // Max
        game = game.with_move(&Move::new(6).unwrap()).unwrap(); // Min
        game = game.with_move(&Move::new(1).unwrap()).unwrap(); // Max
        game = game.with_move(&Move::new(6).unwrap()).unwrap(); // Min
        game = game.with_move(&Move::new(2).unwrap()).unwrap(); // Max

        let result = ai::minimax(&game, &search_config);
        let mv = result.best_move.expect("AI must have a move");

        assert_eq!(mv, Move::new(3).unwrap(), "AI should block Max's immediate win at column 3");
    }

    #[test]
    fn test_ai_takes_immediate_win() {
        //+-------+
        //|.......|
        //|.......|
        //|.......|
        //|......O|
        //|......O|
        //|XXX..XO|
        //+-------+
        // 0123456

        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
        game.set_player_heuristic(MinMaxPlayer::Max, HeuristicVersion::V3);
        game.set_player_heuristic(MinMaxPlayer::Min, HeuristicVersion::V3);

        let search_config = SearchConfig::new_alpha_beta(6)
            .with_time_ms(Some(50));

        game = game.with_move(&Move::new(0).unwrap()).unwrap(); // Max
        game = game.with_move(&Move::new(6).unwrap()).unwrap(); // Min
        game = game.with_move(&Move::new(1).unwrap()).unwrap(); // Max
        game = game.with_move(&Move::new(6).unwrap()).unwrap(); // Min
        game = game.with_move(&Move::new(2).unwrap()).unwrap(); // Max
        game = game.with_move(&Move::new(6).unwrap()).unwrap(); // Min
        game = game.with_move(&Move::new(5).unwrap()).unwrap(); // Max

        let result = ai::minimax(&game, &search_config);
        let mv = result.best_move.expect("AI must have a move");

        assert_eq!(mv, Move::new(6).unwrap(), "AI should secure immediate win at column 6");
    }

    #[test]
    fn test_ai_avoids_playing_moves_leading_to_loss() {
        // this test shows that AI should not play a move that lead to
        // victory of other player if there's another safe move

        //+-------+
        //|.......|
        //|.......|
        //|.......|
        //|.......|
        //|OOO...X|
        //|XXO...X|
        //+-------+
        // 0123456

        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
        game.set_player_heuristic(MinMaxPlayer::Max, HeuristicVersion::V3);
        game.set_player_heuristic(MinMaxPlayer::Min, HeuristicVersion::V3);

        let search_config = SearchConfig::new_alpha_beta(6)
            .with_time_ms(Some(50));

        game = game.with_move(&Move::new(0).unwrap()).unwrap(); // Max
        game = game.with_move(&Move::new(0).unwrap()).unwrap(); // Min
        game = game.with_move(&Move::new(1).unwrap()).unwrap(); // Max
        game = game.with_move(&Move::new(1).unwrap()).unwrap(); // Min
        game = game.with_move(&Move::new(6).unwrap()).unwrap(); // Max
        game = game.with_move(&Move::new(2).unwrap()).unwrap(); // Min
        game = game.with_move(&Move::new(6).unwrap()).unwrap(); // Max
        game = game.with_move(&Move::new(2).unwrap()).unwrap(); // Min

        let result = ai::minimax(&game, &search_config);
        let mv = result.best_move.expect("AI must have a move");

        assert_ne!(mv, Move::new(3).unwrap(), "AI should not give win by playing colum 3");
    }

    #[test]
    fn test_full_ai_to_ai_game() {
        let mut game = ConnectFourState::new(MinMaxPlayer::Max);
        game.set_player_heuristic(MinMaxPlayer::Max, HeuristicVersion::V3);
        game.set_player_heuristic(MinMaxPlayer::Min, HeuristicVersion::V3);

        let mut cfg = SearchConfig::new_alpha_beta(6);
        cfg.time_ms = Some(50);

        while game.outcome().is_none() {
            let res = ai::minimax(&game, &cfg);
            let mv  = res.best_move.unwrap();
            game = game.with_move(&mv).unwrap();
        }

        let outcome = game.outcome().unwrap();
        match outcome {
            Outcome::Win(_) | Outcome::Draw => {
                // OK
            }
        }
    }
}