use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, path::PathBuf, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub difficulty: String,
    pub width: usize,
    pub height: usize,
    pub mines: usize,
    pub milliseconds: u128,
    pub date: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Scores {
    pub entries: Vec<Score>,
}

impl Scores {
    pub fn load() -> Self {
        path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn record(
        &mut self,
        difficulty: &str,
        width: usize,
        height: usize,
        mines: usize,
        elapsed: Duration,
    ) -> io::Result<()> {
        self.entries.push(Score {
            difficulty: difficulty.to_string(),
            width,
            height,
            mines,
            milliseconds: elapsed.as_millis(),
            date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });
        self.entries.sort_by_key(|entry| entry.milliseconds);
        self.retain_best();
        self.save()
    }

    fn retain_best(&mut self) {
        let mut counts = HashMap::new();
        self.entries.retain(|entry| {
            let key = (
                entry.difficulty.clone(),
                entry.width,
                entry.height,
                entry.mines,
            );
            let count = counts.entry(key).or_insert(0);
            *count += 1;
            *count <= 10
        });
    }

    pub fn for_board(
        &self,
        difficulty: &str,
        width: usize,
        height: usize,
        mines: usize,
    ) -> Vec<&Score> {
        self.entries
            .iter()
            .filter(|entry| {
                entry.difficulty == difficulty
                    && entry.width == width
                    && entry.height == height
                    && entry.mines == mines
            })
            .take(10)
            .collect()
    }

    fn save(&self) -> io::Result<()> {
        let Some(path) = path() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)
    }
}

fn path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|directory| directory.join("mines").join("scores.json"))
}

pub fn format_time(milliseconds: u128) -> String {
    let seconds = milliseconds / 1_000;
    format!(
        "{:02}:{:02}.{:03}",
        seconds / 60,
        seconds % 60,
        milliseconds % 1_000
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_format_is_stable() {
        assert_eq!(format_time(125_042), "02:05.042");
    }

    #[test]
    fn rankings_remain_independent_per_field() {
        let mut scores = Scores::default();
        for index in 0..12 {
            scores.entries.push(Score {
                difficulty: "beginner".into(),
                width: 9,
                height: 9,
                mines: 10,
                milliseconds: index,
                date: String::new(),
            });
            scores.entries.push(Score {
                difficulty: "expert".into(),
                width: 30,
                height: 16,
                mines: 99,
                milliseconds: 100 + index,
                date: String::new(),
            });
        }
        scores.entries.sort_by_key(|entry| entry.milliseconds);
        scores.retain_best();
        assert_eq!(scores.for_board("beginner", 9, 9, 10).len(), 10);
        assert_eq!(scores.for_board("expert", 30, 16, 99).len(), 10);
    }
}
