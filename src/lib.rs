use clap::Parser;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::fs::Metadata;
use std::io;
use std::path::Path;
use std::{fs, path::PathBuf};
use strum::Display;
use tabled::{
    Table, Tabled,
    settings::{
        Color, Style,
        object::{Columns, Rows},
    },
};

#[derive(Debug, Display, Serialize)]
pub enum EntryType {
    File,
    Directory,
}

#[derive(Debug, Tabled, Serialize)]
pub struct FileEntry {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    e_type: EntryType,
    #[tabled(rename = "Size")]
    byte_size: String,
    #[tabled(rename = "Modified")]
    modified: String,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = "Best CLI tool for managing your files")]
pub struct CLI {
    path: Option<PathBuf>,
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Output shows hidden '.' files"
    )]
    all: bool,
    #[arg(short, long, default_value_t = false, help = "Output in JSON format")]
    json: bool,
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Recursively calculate the size of directories"
    )]
    recursive: bool,
}

impl CLI {
    pub fn run(&self) {
        let path = self.get_path();

        if let Ok(does_exist) = fs::exists(&path) {
            if does_exist {
                if self.json {
                    self.print_json()
                } else {
                    self.print_table()
                }
            } else {
                eprintln!("{}", "Path does not exist".red())
            }
        } else {
            eprintln!("{}", "Error checking path existence".red())
        }
    }
    fn get_path(&self) -> PathBuf {
        self.path.clone().unwrap_or(PathBuf::from("."))
    }
    fn print_json(&self) {
        let files = self.get_files();
        let json_output = serde_json::to_string_pretty(&files)
            .unwrap_or_else(|_| "Error serializing to JSON".into());
        println!("{}", json_output);
    }

    fn print_table(&self) {
        let files = self.get_files();
        let mut table = Table::new(&files);
        table.with(Style::modern());
        table.modify(Columns::first(), Color::FG_BRIGHT_CYAN);
        table.modify(Columns::one(2), Color::FG_BRIGHT_MAGENTA);
        table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
        table.modify(Rows::first(), Color::FG_BRIGHT_GREEN);
        println!("{}", table);
    }
    pub fn get_files(&self) -> Vec<FileEntry> {
        let mut data = Vec::default();
        if let Ok(read_dir) = fs::read_dir(self.get_path()) {
            for file in read_dir.flatten() {
                self.map_data(&mut data, file);
            }
        };

        data
    }

    fn map_data(&self, data: &mut Vec<FileEntry>, file: fs::DirEntry) {
        if let Ok(metadata) = fs::metadata(file.path()) {
            let name = file.file_name().into_string().unwrap_or("unknown".into());

            if !self.all && name.starts_with('.') {
                return;
            }

            let e_type = if metadata.is_dir() {
                EntryType::Directory
            } else {
                EntryType::File
            };

            let byte_size = Self::format_bytes(if metadata.is_dir() && self.recursive {
                Self::get_dir_size(file.path()).unwrap_or(0)
            } else {
                metadata.len()
            });

            data.push(FileEntry {
                name,
                e_type,
                byte_size,
                modified: self.get_modified(metadata),
            });
        }
    }

    pub fn format_bytes(bytes: u64) -> String {
        match bytes as f64 {
            1e12.. => format!("{:.2} TB", bytes as f64 / 1e12),
            1e9.. => format!("{:.2} GB", bytes as f64 / 1e9),
            1e6.. => format!("{:.2} MB", bytes as f64 / 1e6),
            1e3.. => format!("{:.2} KB", bytes as f64 / 1e3),
            _ => format!("{} B", bytes),
        }
    }

    fn get_dir_size(path: impl AsRef<Path>) -> io::Result<u64> {
        let mut total_size: u64 = 0;

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                total_size += Self::get_dir_size(entry.path())?;
            } else {
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }

    fn get_modified(&self, metadata: Metadata) -> String {
        chrono::DateTime::<chrono::Local>::from(
            metadata
                .modified()
                .unwrap_or_else(|_| std::time::SystemTime::now()),
        )
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn show_hidden_files() {
        let cli = CLI {
            path: None,
            all: true,
            json: false,
            recursive: false,
        };
        let files: Vec<String> = cli.get_files().iter().map(|f| f.name.clone()).collect();
        assert!(files.contains(&String::from(".git")));

        let cli = CLI {
            path: None,
            all: false,
            json: false,
            recursive: false,
        };
        let files: Vec<String> = cli.get_files().iter().map(|f| f.name.clone()).collect();
        assert!(!files.contains(&String::from(".git")));
    }
    #[test]
    fn format_size_string() {
        assert_eq!("0 B", CLI::format_bytes(0));
        assert_eq!("504 B", CLI::format_bytes(504));
        assert_eq!("16.69 KB", CLI::format_bytes(16.69e3 as u64));
        assert_eq!("16.69 MB", CLI::format_bytes(16.69e6 as u64));
        assert_eq!("16.69 GB", CLI::format_bytes(16.69e9 as u64));
        assert_eq!("16.69 TB", CLI::format_bytes(16.69e12 as u64));
    }
}
