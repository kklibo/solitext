use crate::cards::Card;
use crate::game_state::GameState;

mod cards;
mod game_state;
mod tui;

fn main() {
    println!("Hello, world!");
    let mut game_state = GameState::init(Card::ordered_deck());
    tui::run_ui(&mut game_state);
}
