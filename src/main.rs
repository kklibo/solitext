use crate::cards::Card;
use crate::game_state::GameState;
use crate::tui::Ui;

mod cards;
mod game_state;
mod tui;

fn main() {
    println!("Hello, world!");
    let mut game_state = GameState::init(Card::ordered_deck());
    let mut ui = Ui::new();
    ui.run(&mut game_state);
}
