use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub width: usize,
    pub height: usize,
    pub mines: usize,
    pub name: String,
    pub seed: Option<u64>,
}

impl GameConfig {
    pub fn new(width: usize, height: usize, mines: usize, name: &str) -> Result<Self, String> {
        if !(5..=100).contains(&width) || !(5..=100).contains(&height) {
            return Err("board dimensions must be between 5 and 100".into());
        }
        let safe_zone = 9.min(width * height);
        if mines == 0 || mines > width * height - safe_zone {
            return Err("mine count leaves no safe first-click zone".into());
        }
        Ok(Self {
            width,
            height,
            mines,
            name: name.to_string(),
            seed: None,
        })
    }

    pub fn beginner() -> Self {
        Self::new(9, 9, 10, "beginner").expect("valid preset")
    }

    pub fn intermediate() -> Self {
        Self::new(16, 16, 40, "intermediate").expect("valid preset")
    }

    pub fn expert() -> Self {
        Self::new(30, 16, 99, "expert").expect("valid preset")
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Cell {
    pub mine: bool,
    pub revealed: bool,
    pub flagged: bool,
    pub adjacent: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ready,
    Playing,
    Won,
    Lost,
}

pub struct Game {
    pub config: GameConfig,
    pub cells: Vec<Cell>,
    pub cursor: (usize, usize),
    pub status: Status,
    started: Option<Instant>,
    finished: Option<Duration>,
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        Self {
            cells: vec![Cell::default(); config.width * config.height],
            config,
            cursor: (0, 0),
            status: Status::Ready,
            started: None,
            finished: None,
        }
    }

    pub fn restart(&mut self) {
        *self = Self::new(self.config.clone());
    }

    pub fn cell(&self, x: usize, y: usize) -> Cell {
        self.cells[self.index(x, y)]
    }

    pub fn elapsed(&self) -> Duration {
        self.finished
            .or_else(|| self.started.map(|started| started.elapsed()))
            .unwrap_or_default()
    }

    pub fn flags(&self) -> usize {
        self.cells.iter().filter(|cell| cell.flagged).count()
    }

    pub fn remaining_mines(&self) -> isize {
        self.config.mines as isize - self.flags() as isize
    }

    pub fn move_cursor(&mut self, dx: isize, dy: isize) {
        self.cursor.0 =
            (self.cursor.0 as isize + dx).clamp(0, self.config.width as isize - 1) as usize;
        self.cursor.1 =
            (self.cursor.1 as isize + dy).clamp(0, self.config.height as isize - 1) as usize;
    }

    pub fn reveal_cursor(&mut self) {
        let (x, y) = self.cursor;
        self.reveal(x, y);
    }

    pub fn flag_cursor(&mut self) {
        let (x, y) = self.cursor;
        self.toggle_flag(x, y);
    }

    pub fn reveal(&mut self, x: usize, y: usize) {
        if matches!(self.status, Status::Won | Status::Lost) || self.cell(x, y).flagged {
            return;
        }
        if self.status == Status::Ready {
            self.generate(x, y);
            self.status = Status::Playing;
            self.started = Some(Instant::now());
        }
        let cell = self.cell(x, y);
        if cell.revealed {
            self.chord(x, y);
        } else if cell.mine {
            let index = self.index(x, y);
            self.cells[index].revealed = true;
            self.finish(Status::Lost);
        } else {
            self.flood_reveal(x, y);
            self.check_win();
        }
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize) {
        if matches!(self.status, Status::Won | Status::Lost) {
            return;
        }
        let index = self.index(x, y);
        if !self.cells[index].revealed {
            self.cells[index].flagged = !self.cells[index].flagged;
        }
    }

    fn generate(&mut self, safe_x: usize, safe_y: usize) {
        let mut candidates: Vec<usize> = (0..self.cells.len())
            .filter(|index| {
                let (x, y) = self.coordinates(*index);
                x.abs_diff(safe_x) > 1 || y.abs_diff(safe_y) > 1
            })
            .collect();
        let mut rng = match self.config.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };
        candidates.shuffle(&mut rng);
        for index in candidates.into_iter().take(self.config.mines) {
            self.cells[index].mine = true;
        }
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let count = self
                    .neighbors(x, y)
                    .filter(|(nx, ny)| self.cell(*nx, *ny).mine)
                    .count() as u8;
                let index = self.index(x, y);
                self.cells[index].adjacent = count;
            }
        }
    }

    fn flood_reveal(&mut self, x: usize, y: usize) {
        let mut queue = VecDeque::from([(x, y)]);
        while let Some((cx, cy)) = queue.pop_front() {
            let index = self.index(cx, cy);
            if self.cells[index].revealed || self.cells[index].flagged || self.cells[index].mine {
                continue;
            }
            self.cells[index].revealed = true;
            if self.cells[index].adjacent == 0 {
                for neighbor in self.neighbors(cx, cy) {
                    queue.push_back(neighbor);
                }
            }
        }
    }

    fn chord(&mut self, x: usize, y: usize) {
        let cell = self.cell(x, y);
        if cell.adjacent == 0 {
            return;
        }
        let flagged = self
            .neighbors(x, y)
            .filter(|(nx, ny)| self.cell(*nx, *ny).flagged)
            .count();
        if flagged != cell.adjacent as usize {
            return;
        }
        let neighbors: Vec<_> = self.neighbors(x, y).collect();
        for (nx, ny) in neighbors {
            if self.cell(nx, ny).mine && !self.cell(nx, ny).flagged {
                let index = self.index(nx, ny);
                self.cells[index].revealed = true;
                self.finish(Status::Lost);
                return;
            }
            self.flood_reveal(nx, ny);
        }
        self.check_win();
    }

    fn check_win(&mut self) {
        let safe = self.cells.len() - self.config.mines;
        let revealed = self.cells.iter().filter(|cell| cell.revealed).count();
        if revealed == safe {
            self.finish(Status::Won);
        }
    }

    fn finish(&mut self, status: Status) {
        self.finished = Some(self.elapsed());
        self.status = status;
        if status == Status::Lost {
            for cell in &mut self.cells {
                if cell.mine {
                    cell.revealed = true;
                }
            }
        } else {
            for cell in &mut self.cells {
                if cell.mine {
                    cell.flagged = true;
                }
            }
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.config.width + x
    }

    fn coordinates(&self, index: usize) -> (usize, usize) {
        (index % self.config.width, index / self.config.width)
    }

    fn neighbors(&self, x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
        let width = self.config.width as isize;
        let height = self.config.height as isize;
        (-1..=1).flat_map(move |dy| {
            (-1..=1).filter_map(move |dx| {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                (dx != 0 || dy != 0)
                    .then_some((nx, ny))
                    .filter(|(nx, ny)| *nx >= 0 && *ny >= 0 && *nx < width && *ny < height)
                    .map(|(nx, ny)| (nx as usize, ny as usize))
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded() -> Game {
        let mut config = GameConfig::beginner();
        config.seed = Some(42);
        Game::new(config)
    }

    #[test]
    fn first_click_and_neighbors_are_safe() {
        let mut game = seeded();
        game.reveal(4, 4);
        for y in 3..=5 {
            for x in 3..=5 {
                assert!(!game.cell(x, y).mine);
            }
        }
    }

    #[test]
    fn generated_board_has_exact_mine_count() {
        let mut game = seeded();
        game.reveal(0, 0);
        assert_eq!(game.cells.iter().filter(|cell| cell.mine).count(), 10);
    }

    #[test]
    fn flood_fill_reveals_more_than_one_cell() {
        let mut game = seeded();
        game.reveal(4, 4);
        assert!(game.cells.iter().filter(|cell| cell.revealed).count() > 1);
    }

    #[test]
    fn flag_prevents_reveal() {
        let mut game = seeded();
        game.toggle_flag(2, 2);
        game.reveal(2, 2);
        assert!(game.cell(2, 2).flagged);
        assert!(!game.cell(2, 2).revealed);
    }

    #[test]
    fn custom_board_validation_rejects_impossible_density() {
        assert!(GameConfig::new(5, 5, 17, "custom").is_err());
    }
}
