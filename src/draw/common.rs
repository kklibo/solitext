//! Common drawing code.

use super::*;

impl Draw {
    pub(crate) fn clear_screen(&mut self) {
        writeln!(self.stdout, "{}", clear::All,).unwrap();
    }

    pub(crate) fn default_bg() -> impl color::Color {
        color::Black
    }
    pub(crate) fn default_fg() -> impl color::Color {
        color::LightWhite
    }

    pub(crate) fn set_colors(
        &mut self,
        foreground: impl color::Color,
        background: impl color::Color,
    ) {
        writeln!(
            self.stdout,
            "{}{}",
            color::Fg(foreground),
            color::Bg(background),
        )
        .unwrap();
    }

    pub(crate) fn draw_box(&mut self, col1: usize, row1: usize, col2: usize, row2: usize) {
        use std::cmp::{max, min};
        for col in min(col1, col2)..=max(col1, col2) {
            for row in min(row1, row2)..=max(row1, row2) {
                self.draw_text(col, row, "█");
            }
        }
    }

    pub fn draw_text(&mut self, col: usize, row: usize, text: &str) {
        let col = u16::try_from(col).expect("column should fit in a u16");
        let row = u16::try_from(row).expect("row should fit in a u16");

        writeln!(self.stdout, "{}{}", cursor::Goto(col, row), text).unwrap();
    }

    pub fn set_up_terminal(&mut self) {
        writeln!(
            self.stdout,
            "{}{}{}{}{}",
            color::Fg(Self::default_fg()),
            color::Bg(Self::default_bg()),
            clear::All,
            cursor::Goto(1, 1),
            cursor::Hide,
        )
        .unwrap();
        self.stdout.flush().unwrap();
    }

    pub fn restore_terminal(&mut self) {
        writeln!(
            self.stdout,
            "{}{}{}{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            clear::All,
            cursor::Goto(1, 1),
            cursor::Show,
        )
        .unwrap();
        self.stdout.flush().unwrap();
    }

    fn centered_box_corners(width: usize, height: usize) -> (usize, usize, usize, usize) {
        const CENTER: (usize, usize) = (26, 5);
        (
            CENTER.0 - width / 2,
            CENTER.1 - height / 2,
            CENTER.0 + width / 2,
            CENTER.1 + height / 2,
        )
    }

    fn draw_centered_box(&mut self, width: usize, height: usize) {
        let (col1, row1, col2, row2) = Self::centered_box_corners(width, height);
        self.draw_box(col1, row1, col2, row2);
    }

    pub fn draw_text_box(&mut self, lines: &str) {
        let height = lines.split('\n').count();

        const WIDTH: usize = 38;
        self.set_colors(color::LightBlue, Self::default_bg());
        self.draw_centered_box(WIDTH, height + 2);
        self.set_colors(color::White, Self::default_bg());
        self.draw_centered_box(WIDTH - 2, height);

        self.set_colors(color::LightBlack, color::White);
        let (col, mut row, _, _) = Self::centered_box_corners(WIDTH - 2, height);

        for line in lines.split('\n') {
            self.draw_text(col, row, line);
            row += 1;
        }
    }
}
