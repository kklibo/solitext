use crate::cards::Card;
use crate::draw::Draw;
use crate::game_logic;
use crate::game_state::GameState;
use crate::selection::Selection;
use std::io::stdin;
use termion::event::Key;
use termion::input::TermRead;

pub struct Ui {
    ui_state: UiState,
    draw: Draw,
}

enum UiState {
    Intro,
    NewGame,
    Game,
    Victory,
    Quit,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            ui_state: UiState::Intro,
            draw: Draw::new(),
        }
    }
    pub fn reset_for_new_game(&mut self) {
        self.draw.cursor = Selection::Deck;
        self.draw.selected = None;
        self.draw.debug_message.clear();
        self.draw.context_help_message.clear();
    }

    fn move_cards(from: Selection, to: Selection, game_state: &mut GameState) -> Result<(), ()> {
        if from.same_collection(to) {
            return Err(());
        }

        let cards = from
            .selected_collection(game_state)
            .take(from.card_count())?;

        to.selected_collection(game_state).receive(cards)?;
        Ok(())
    }

    fn cards_action(&mut self, game_state: &mut GameState) {
        if let (Some(from), to) = (self.draw.selected, self.draw.cursor) {
            self.draw.selected = None;

            if game_logic::valid_move(from, to, game_state).is_ok() {
                match Self::move_cards(from, to, game_state) {
                    Ok(_) => self.draw.debug_message = "move OK".to_string(),
                    Err(_) => self.draw.debug_message = "move attempt failed".to_string(),
                }
            } else {
                self.draw.debug_message = "invalid move".to_string();
            }
        } else if self.draw.cursor.card_count() > 0 {
            self.draw.selected = Some(self.draw.cursor);
        }
    }

    fn enter_key_action(&mut self, game_state: &mut GameState) {
        if matches!(self.draw.cursor, Selection::Deck) {
            game_state.deck_hit();
        } else if let Selection::Column { index, .. } = self.draw.cursor {
            let from = Selection::Column {
                index,
                card_count: 1,
            };
            //todo: refactor this
            for i in 0..4 {
                let to = Selection::Pile { index: i };
                if game_logic::valid_move(from, to, game_state).is_ok() {
                    let _ = Self::move_cards(from, to, game_state);
                }
            }
        }
    }

    fn debug_unchecked_cards_action(&mut self, game_state: &mut GameState) {
        if let Some(selected) = self.draw.selected {
            self.draw.selected = None;
            let _ = Self::move_cards(selected, self.draw.cursor, game_state);
        } else {
            self.draw.selected = Some(self.draw.cursor)
        }
    }

    fn debug_check_valid(&mut self, game_state: &mut GameState) {
        if let (Some(from), to) = (self.draw.selected, self.draw.cursor) {
            self.draw.debug_message = format!("{:?}", game_logic::valid_move(from, to, game_state));
        } else {
            self.draw.debug_message = "".to_string();
        }
    }

    fn apply_column_selection_rules(&mut self, game_state: &mut GameState) {
        self.draw
            .cursor
            .apply_column_selection_rules(game_state, self.draw.debug_mode);
        if let Some(mut selected) = self.draw.selected {
            selected.apply_column_selection_rules(game_state, self.draw.debug_mode);
        }
    }

    fn set_context_help_message(&mut self) {
        self.draw.context_help_message = match self.draw.cursor {
            Selection::Deck => "Enter: Hit",
            Selection::Column { .. } => "Enter: Try to Move to Stack",
            _ => "",
        }
        .to_string()
    }

    /// Actions run on each user turn
    /// Returns: true IFF UiState has changed
    fn turn_actions(&mut self, game_state: &mut GameState) -> bool {
        // Ensure a face-up card at the end of each column
        game_logic::face_up_on_columns(game_state);
        // Hit if the deck has cards and the drawn deck is empty
        game_state.auto_hit();
        // Fix column selections, if needed
        self.apply_column_selection_rules(game_state);
        // Update context help line
        self.set_context_help_message();

        // (Any other automatic state changes can go here too)

        if game_logic::victory(game_state) {
            self.draw.debug_message = "Victory".to_string();
            self.ui_state = UiState::Victory;
            return true;
        }

        self.draw.display_game_state(game_state);
        false
    }

    fn run_game(&mut self, game_state: &mut GameState) {
        if self.turn_actions(game_state) {
            return;
        }

        let stdin = stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Left => self.draw.cursor.move_left(),
                Key::Right => self.draw.cursor.move_right(),
                Key::Up => self.draw.cursor.select_up(),
                Key::Down => self.draw.cursor.select_down(),
                Key::Home => self.draw.cursor = Selection::Deck,
                Key::End => self.draw.cursor = Selection::Pile { index: 0 },
                Key::Char(' ') => self.cards_action(game_state),
                Key::Char('\n') => self.enter_key_action(game_state),
                Key::Char('c') if self.draw.debug_mode => {
                    self.debug_unchecked_cards_action(game_state)
                }
                Key::Char('x') => self.draw.selected = None,
                Key::Char('z') if self.draw.debug_mode => self.debug_check_valid(game_state),
                Key::Char('d') => self.draw.debug_mode = !self.draw.debug_mode,
                Key::Char('h') => self.run_help(game_state),
                Key::Esc | Key::Ctrl('c') => {
                    self.ui_state = UiState::Quit;
                    break;
                }
                _ => {}
            }
            if self.turn_actions(game_state) {
                return;
            }
        }
    }

    fn run_intro(&mut self) {
        self.draw.display_intro();
        self.ui_state = UiState::NewGame;
    }

    fn run_victory(&mut self, game_state: &mut GameState) {
        self.draw.display_victory(game_state);

        let stdin = stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('y') => {
                    self.ui_state = UiState::NewGame;
                    break;
                }
                Key::Char('n') | Key::Esc | Key::Ctrl('c') => {
                    self.ui_state = UiState::Quit;
                    break;
                }
                _ => {}
            }
        }
    }

    pub fn run_new_game(&mut self, game_state: &mut GameState) {
        *game_state = GameState::init(Card::shuffled_deck());
        self.reset_for_new_game();
        self.ui_state = UiState::Game;
    }

    pub fn run_help(&mut self, game_state: &mut GameState) {
        self.draw.display_help(game_state);
        stdin().keys().next();
    }

    pub fn run(&mut self, game_state: &mut GameState) {
        self.draw.set_up_terminal();

        loop {
            match self.ui_state {
                UiState::Intro => self.run_intro(),
                UiState::NewGame => self.run_new_game(game_state),
                UiState::Game => self.run_game(game_state),
                UiState::Victory => self.run_victory(game_state),
                UiState::Quit => break,
            }
        }

        self.draw.restore_terminal();
        self.draw
            .draw_text(1, 1, "please send bug reports via IRC or ham radio");
        self.draw.draw_text(1, 1, "");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Card;

    #[test]
    fn test_same_collection() {
        assert!(!Selection::Deck.same_collection(Selection::Column {
            index: 1,
            card_count: 1
        }));
        assert!(!Selection::Column {
            index: 1,
            card_count: 1
        }
        .same_collection(Selection::Pile { index: 1 }));
        assert!(!Selection::Pile { index: 1 }.same_collection(Selection::Deck));

        assert!(Selection::Deck.same_collection(Selection::Deck));

        assert!(Selection::Column {
            index: 1,
            card_count: 1
        }
        .same_collection(Selection::Column {
            index: 1,
            card_count: 1
        }));
        assert!(Selection::Column {
            index: 1,
            card_count: 2
        }
        .same_collection(Selection::Column {
            index: 1,
            card_count: 1
        }));
        assert!(!Selection::Column {
            index: 2,
            card_count: 1
        }
        .same_collection(Selection::Column {
            index: 1,
            card_count: 1
        }));

        assert!(Selection::Pile { index: 1 }.same_collection(Selection::Pile { index: 1 }));
        assert!(!Selection::Pile { index: 2 }.same_collection(Selection::Pile { index: 1 }));
    }

    #[test]
    fn test_selected_collection() {
        let mut a = GameState::init(Card::ordered_deck());
        let _b = Selection::Deck.selected_collection(&mut a);
    }
}
