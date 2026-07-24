use clap::Parser;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use strum::Display;
use tabled::{
    Table, Tabled,
    settings::{
        Color, Style,
        object::{Columns, Rows},
    },
};

#[derive(Debug, Display, Serialize)]
enum EntryType {
    File,
    Directory,
}

#[derive(Debug, Tabled, Serialize)]
struct FileEntry {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    e_type: EntryType,
    #[tabled(rename = "Size")]
    bytes: u64,
    #[tabled(rename = "Modified")]
    modified: String,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = "Best CLI tool for managing your files")]
struct CLI {
    path: Option<PathBuf>,
    #[arg(short, long, default_value_t = false, help = "Output in JSON format")]
    json: bool,
}

fn main() {
    let cli = CLI::parse();

    let path = cli.path.unwrap_or(PathBuf::from("."));

    if let Ok(does_exist) = fs::exists(&path) {
        if does_exist {
            if cli.json {
                let files = get_files(&path);
                let json_output = serde_json::to_string_pretty(&files).unwrap_or_else(|_| "Error serializing to JSON".into());
                println!("{}", json_output);
            } else {
                print_table(path);
            }
        } else {
            println!("{}", "Path does not exist".red());
        }
    } else {
        println!("{}", "Error checking path existence".red());
    }
}

fn print_table(path: PathBuf) {
    let files = get_files(&path);
    let mut table = Table::new(&files);
    table.with(Style::modern());
    table.modify(Columns::first(), Color::FG_BRIGHT_CYAN);
    table.modify(Columns::one(2), Color::FG_BRIGHT_MAGENTA);
    table.modify(Columns::one(3), Color::FG_BRIGHT_YELLOW);
    table.modify(Rows::first(), Color::FG_BRIGHT_GREEN);
    println!("{}", table);
}

fn get_files(path: &Path) -> Vec<FileEntry> {
    let mut data = Vec::default();

    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir {
            if let Ok(file) = entry {
                map_data(&mut data, file);
            }
        }
    };

    data
}

fn map_data(data: &mut Vec<FileEntry>, file: fs::DirEntry) {
    if let Ok(metadata) = fs::metadata(&file.path()) {
        data.push(FileEntry {
            name: file.file_name().into_string().unwrap_or("unknown".into()),
            e_type: if metadata.is_dir() {
                EntryType::Directory
            } else {
                EntryType::File
            },
            bytes: metadata.len(),
            modified: chrono::DateTime::<chrono::Local>::from(
                metadata
                    .modified()
                    .unwrap_or_else(|_| std::time::SystemTime::now()),
            )
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        });
    }
}
