use crate::game::{Cell, Game, GameConfig, Status};
use crate::scores::{Scores, format_time};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind,
    },
    execute, queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{Stdout, Write, stdout};
use std::time::Duration;

pub struct App {
    game: Game,
    scores: Scores,
    show_help: bool,
    show_scores: bool,
    recorded: bool,
    viewport: Viewport,
}

#[derive(Debug, Clone, Copy, Default)]
struct Viewport {
    x: usize,
    y: usize,
    columns: usize,
    rows: usize,
    cell_width: u16,
    cell_height: u16,
    board_x: u16,
    board_y: u16,
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter(stdout: &mut Stdout) -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = stdout();
        let _ = execute!(
            stdout,
            Show,
            DisableMouseCapture,
            LeaveAlternateScreen,
            ResetColor
        );
        let _ = terminal::disable_raw_mode();
    }
}

impl App {
    pub fn new(config: GameConfig) -> Self {
        Self {
            game: Game::new(config),
            scores: Scores::load(),
            show_help: false,
            show_scores: false,
            recorded: false,
            viewport: Viewport::default(),
        }
    }

    pub fn run(mut self) -> std::io::Result<()> {
        let mut stdout = stdout();
        let _guard = TerminalGuard::enter(&mut stdout)?;
        loop {
            self.draw(&mut stdout)?;
            if event::poll(Duration::from_millis(100))? && !self.handle(event::read()?) {
                break;
            }
        }
        Ok(())
    }

    fn handle(&mut self, event: Event) -> bool {
        if self.show_help {
            if matches!(event, Event::Key(key) if matches!(key.code, KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc))
            {
                self.show_help = false;
            }
            return true;
        }
        if self.show_scores {
            if matches!(event, Event::Key(key) if matches!(key.code, KeyCode::Char('s') | KeyCode::Char('q') | KeyCode::Esc))
            {
                self.show_scores = false;
            }
            return true;
        }
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return false,
                KeyCode::Char('?') => self.show_help = !self.show_help,
                KeyCode::Char('s') => self.show_scores = !self.show_scores,
                KeyCode::Char('r') => self.restart(),
                KeyCode::Up | KeyCode::Char('k') => self.game.move_cursor(0, -1),
                KeyCode::Down | KeyCode::Char('j') => self.game.move_cursor(0, 1),
                KeyCode::Left | KeyCode::Char('h') => self.game.move_cursor(-1, 0),
                KeyCode::Right | KeyCode::Char('l') => self.game.move_cursor(1, 0),
                KeyCode::Enter | KeyCode::Char(' ') => self.game.reveal_cursor(),
                KeyCode::Char('f') => self.game.flag_cursor(),
                _ => {}
            },
            Event::Mouse(mouse) if !self.show_help && !self.show_scores => {
                if let Some((x, y)) = self.mouse_cell(mouse.column, mouse.row) {
                    self.game.cursor = (x, y);
                    match mouse.kind {
                        MouseEventKind::Down(MouseButton::Left) => self.game.reveal(x, y),
                        MouseEventKind::Down(MouseButton::Right) => self.game.toggle_flag(x, y),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        true
    }

    fn restart(&mut self) {
        self.game.restart();
        self.recorded = false;
        self.show_help = false;
        self.show_scores = false;
    }

    fn draw(&mut self, stdout: &mut Stdout) -> std::io::Result<()> {
        if self.game.status == Status::Won && !self.recorded {
            self.scores.record(
                &self.game.config.name,
                self.game.config.width,
                self.game.config.height,
                self.game.config.mines,
                self.game.elapsed(),
            )?;
            self.recorded = true;
        }
        let (width, height) = terminal::size()?;
        self.viewport = layout(width, height, &self.game);
        queue!(stdout, Clear(ClearType::All))?;
        self.draw_header(stdout, width)?;
        self.draw_board(stdout)?;
        self.draw_footer(stdout, width, height)?;
        if self.show_help {
            self.draw_help(stdout, width, height)?;
        } else if self.show_scores {
            self.draw_scores(stdout, width, height)?;
        } else if matches!(self.game.status, Status::Won | Status::Lost) {
            self.draw_result(stdout, width, height)?;
        }
        stdout.flush()
    }

    fn draw_header(&self, stdout: &mut Stdout, width: u16) -> std::io::Result<()> {
        let elapsed = format_time(self.game.elapsed().as_millis());
        let status = format!(
            " MINES  {}  {}x{}  bombs {}  flags {:+}  time {} ",
            self.game.config.name,
            self.game.config.width,
            self.game.config.height,
            self.game.config.mines,
            self.game.remaining_mines(),
            elapsed
        );
        queue!(
            stdout,
            MoveTo(0, 0),
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White),
            Print(center(&status, width as usize)),
            ResetColor
        )
    }

    fn draw_board(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        let view = self.viewport;
        for row in 0..view.rows {
            for column in 0..view.columns {
                let x = view.x + column;
                let y = view.y + row;
                let cell = self.game.cell(x, y);
                for inner_y in 0..view.cell_height {
                    let screen_x = view.board_x + column as u16 * view.cell_width;
                    let screen_y = view.board_y + row as u16 * view.cell_height + inner_y;
                    self.draw_cell(stdout, screen_x, screen_y, view.cell_width, cell, (x, y))?;
                }
            }
        }
        Ok(())
    }

    fn draw_cell(
        &self,
        stdout: &mut Stdout,
        x: u16,
        y: u16,
        width: u16,
        cell: Cell,
        coordinates: (usize, usize),
    ) -> std::io::Result<()> {
        let selected = coordinates == self.game.cursor;
        let (symbol, foreground, background) = if cell.revealed && cell.mine {
            ("✹", Color::White, Color::DarkRed)
        } else if cell.flagged {
            ("⚑", Color::Yellow, Color::DarkGrey)
        } else if !cell.revealed {
            ("·", Color::Grey, Color::DarkGrey)
        } else if cell.adjacent == 0 {
            (" ", Color::White, Color::Black)
        } else {
            (
                number(cell.adjacent),
                number_color(cell.adjacent),
                Color::Black,
            )
        };
        let content = center(symbol, width as usize);
        queue!(
            stdout,
            MoveTo(x, y),
            SetForegroundColor(foreground),
            SetBackgroundColor(if selected {
                Color::DarkCyan
            } else {
                background
            }),
            SetAttribute(if selected {
                Attribute::Bold
            } else {
                Attribute::Reset
            }),
            Print(content),
            ResetColor,
            SetAttribute(Attribute::Reset)
        )
    }

    fn draw_footer(&self, stdout: &mut Stdout, width: u16, height: u16) -> std::io::Result<()> {
        let text =
            " arrows/hjkl move  Space reveal/chord  F flag  S scores  ? help  R restart  Q quit ";
        queue!(
            stdout,
            MoveTo(0, height.saturating_sub(1)),
            SetForegroundColor(Color::DarkGrey),
            Print(center(text, width as usize)),
            ResetColor
        )
    }

    fn draw_result(&self, stdout: &mut Stdout, width: u16, height: u16) -> std::io::Result<()> {
        let (title, color) = if self.game.status == Status::Won {
            (" MINEFIELD CLEARED ", Color::Green)
        } else {
            (" DETONATED ", Color::Red)
        };
        let lines = [
            title.to_string(),
            format!("Time {}", format_time(self.game.elapsed().as_millis())),
            "R replay   S scores   Q quit".to_string(),
        ];
        overlay(stdout, width, height, &lines, color)
    }

    fn draw_help(&self, stdout: &mut Stdout, width: u16, height: u16) -> std::io::Result<()> {
        let lines = [
            " MINES HELP ".to_string(),
            "Reveal every safe cell without touching a mine.".to_string(),
            "The first cell and its eight neighbors are always safe.".to_string(),
            "Space/Enter or left click: reveal; revealed number: chord.".to_string(),
            "F or right click: flag; arrows/hjkl: move.".to_string(),
            "Esc closes this help.".to_string(),
        ];
        overlay(stdout, width, height, &lines, Color::Cyan)
    }

    fn draw_scores(&self, stdout: &mut Stdout, width: u16, height: u16) -> std::io::Result<()> {
        let entries = self.scores.for_board(
            &self.game.config.name,
            self.game.config.width,
            self.game.config.height,
            self.game.config.mines,
        );
        let mut lines = vec![format!(
            " {} BEST TIMES ",
            self.game.config.name.to_uppercase()
        )];
        if entries.is_empty() {
            lines.push("No cleared minefield yet.".to_string());
        } else {
            lines.extend(entries.iter().enumerate().map(|(index, score)| {
                format!(
                    "{:>2}. {}  {}",
                    index + 1,
                    format_time(score.milliseconds),
                    score.date
                )
            }));
        }
        lines.push("Esc or S closes scores.".to_string());
        overlay(stdout, width, height, &lines, Color::Yellow)
    }

    fn mouse_cell(&self, column: u16, row: u16) -> Option<(usize, usize)> {
        let view = self.viewport;
        if column < view.board_x || row < view.board_y {
            return None;
        }
        let x = (column - view.board_x) / view.cell_width;
        let y = (row - view.board_y) / view.cell_height;
        ((x as usize) < view.columns && (y as usize) < view.rows)
            .then_some((view.x + x as usize, view.y + y as usize))
    }
}

fn layout(width: u16, height: u16, game: &Game) -> Viewport {
    let available_width = width.max(1);
    let available_height = height.saturating_sub(3).max(1);
    let cell_width = if available_width as usize >= game.config.width * 5 {
        5
    } else if available_width as usize >= game.config.width * 3 {
        3
    } else {
        2
    };
    let cell_height = if available_height as usize >= game.config.height * 2 {
        2
    } else {
        1
    };
    let columns = (available_width / cell_width)
        .max(1)
        .min(game.config.width as u16) as usize;
    let rows = (available_height / cell_height)
        .max(1)
        .min(game.config.height as u16) as usize;
    let x = game
        .cursor
        .0
        .saturating_sub(columns / 2)
        .min(game.config.width - columns);
    let y = game
        .cursor
        .1
        .saturating_sub(rows / 2)
        .min(game.config.height - rows);
    let board_width = columns as u16 * cell_width;
    let board_height = rows as u16 * cell_height;
    Viewport {
        x,
        y,
        columns,
        rows,
        cell_width,
        cell_height,
        board_x: width.saturating_sub(board_width) / 2,
        board_y: 1 + available_height.saturating_sub(board_height) / 2,
    }
}

fn overlay(
    stdout: &mut Stdout,
    width: u16,
    height: u16,
    lines: &[String],
    color: Color,
) -> std::io::Result<()> {
    let box_width = lines
        .iter()
        .map(String::len)
        .max()
        .unwrap_or(20)
        .min(width.saturating_sub(4) as usize)
        + 4;
    let box_height = lines.len() as u16 + 2;
    let x = width.saturating_sub(box_width as u16) / 2;
    let y = height.saturating_sub(box_height) / 2;
    for row in 0..box_height {
        queue!(
            stdout,
            MoveTo(x, y + row),
            SetBackgroundColor(Color::Black),
            Print(" ".repeat(box_width))
        )?;
    }
    for (index, line) in lines.iter().enumerate() {
        queue!(
            stdout,
            MoveTo(x, y + index as u16 + 1),
            SetForegroundColor(color),
            SetBackgroundColor(Color::Black),
            SetAttribute(if index == 0 {
                Attribute::Bold
            } else {
                Attribute::Reset
            }),
            Print(center(&truncate(line, box_width - 2), box_width)),
            ResetColor,
            SetAttribute(Attribute::Reset)
        )?;
    }
    Ok(())
}

fn center(text: &str, width: usize) -> String {
    let text = truncate(text, width);
    let left = width.saturating_sub(text.chars().count()) / 2;
    format!(
        "{}{}{}",
        " ".repeat(left),
        text,
        " ".repeat(width - left - text.chars().count())
    )
}

fn truncate(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

fn number(value: u8) -> &'static str {
    match value {
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        _ => " ",
    }
}

fn number_color(value: u8) -> Color {
    match value {
        1 => Color::Blue,
        2 => Color::Green,
        3 => Color::Red,
        4 => Color::Magenta,
        5 => Color::DarkRed,
        6 => Color::Cyan,
        7 => Color::Grey,
        _ => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expert_board_fits_standard_terminal() {
        let game = Game::new(GameConfig::expert());
        let view = layout(120, 30, &game);
        assert_eq!(view.columns, 30);
        assert_eq!(view.rows, 16);
        assert!(view.cell_width >= 3);
    }

    #[test]
    fn tiny_terminal_uses_cursor_centered_viewport() {
        let mut game = Game::new(GameConfig::expert());
        game.cursor = (29, 15);
        let view = layout(20, 10, &game);
        assert_eq!(view.x + view.columns, 30);
        assert_eq!(view.y + view.rows, 16);
    }

    #[test]
    fn centered_text_has_exact_width() {
        assert_eq!(center("MINES", 11).chars().count(), 11);
    }
}
