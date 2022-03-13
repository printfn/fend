use crate::file_paths;
use std::{cmp, error, fs, io, path};

pub struct History {
    entries: Vec<String>,
    hist_file_length: usize,
}

impl History {
    pub fn load(hist_file_length: usize) -> Result<Self, Box<dyn error::Error>> {
        let location = Self::location()?;
        if !location.is_file() {
            return Ok(Self {
                entries: vec![],
                hist_file_length,
            });
        }
        let file = fs::read_to_string(location)?;
        let mut entries = vec![];
        for line in file.lines() {
            entries.push(line.replace("\\n", "\n").replace("\\\\", "\\"));
        }
        Ok(Self {
            entries,
            hist_file_length,
        })
    }

    pub fn add_entry(&mut self, entry: &str) {
        if entry.starts_with(' ') {
            return;
        }
        self.entries.push(entry.to_string());
    }

    pub fn _get(&self, idx: usize) -> &str {
        &self.entries[idx]
    }

    pub fn _len(&self) -> usize {
        self.entries.len()
    }

    pub fn location() -> Result<path::PathBuf, file_paths::HomeDirError> {
        let mut history_path = file_paths::get_state_dir()?;
        history_path.push("history");
        Ok(history_path)
    }

    pub fn write(&self) -> Result<(), Box<dyn error::Error>> {
        let mut history_path = file_paths::get_state_dir()?;
        if !history_path.exists() {
            fs::create_dir_all(&history_path)?;
        }
        history_path.push("history");
        let mut f = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&history_path)?;
        let start_idx = cmp::max(self.entries.len(), self.hist_file_length) - self.hist_file_length;
        for line in &self.entries[start_idx..] {
            let mut line = line.replace("\\", "\\\\").replace("\n", "\\n");
            line.push('\n');
            io::Write::write(&mut f, line.as_bytes())?;
        }
        Ok(())
    }
}
