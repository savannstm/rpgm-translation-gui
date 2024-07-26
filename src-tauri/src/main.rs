#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use fastrand::seed;
use regex::escape;
use std::{fs::create_dir_all, path::PathBuf, time::Instant};
use tauri::{command, generate_context, generate_handler, App, Builder, Manager, Window};

mod read;
mod write;

use read::*;
use write::*;

#[derive(PartialEq)]
enum GameType {
    Termina,
}

#[derive(PartialEq)]
enum ProcessingMode {
    Force,
    Append,
    Default,
}

impl AsRef<ProcessingMode> for ProcessingMode {
    fn as_ref(&self) -> &ProcessingMode {
        self
    }
}

impl PartialEq<ProcessingMode> for &ProcessingMode {
    fn eq(&self, other: &ProcessingMode) -> bool {
        *self == other
    }
}

enum Code {
    Dialogue, // also goes for credit
    Choice,
    System,
    Unknown,
}

enum Variable {
    Name,
    Nickname,
    Description,
    Note,
}

trait IntoRSplit {
    fn into_rsplit_once(self, delimiter: char) -> Option<(String, String)>;
}

// genius implementation
impl IntoRSplit for String {
    fn into_rsplit_once(self, delimiter: char) -> Option<(String, String)> {
        if let Some(pos) = self.rfind(delimiter) {
            let (left, right) = self.split_at(pos);
            Some((left.to_string(), right[delimiter.len_utf8()..].to_string()))
        } else {
            None
        }
    }
}

pub fn romanize_string<T>(string: T) -> String
where
    T: AsRef<str>,
    std::string::String: std::convert::From<T>,
{
    let mut i: usize = 0;
    let mut actual_string: String = String::from(string);

    while i < actual_string.len() {
        let replacement: &str = match &actual_string[i..].chars().next() {
            Some('。') => ".",
            Some('、') => ",",
            Some('・') => "·",
            Some('゠') => "–",
            Some('＝') => "—",
            Some('…') | Some('‥') => {
                if i + 3 <= actual_string.len() {
                    i += 2;
                    "..."
                } else {
                    i += 1;
                    continue;
                }
            }
            Some('「') | Some('」') | Some('〈') | Some('〉') => "'",
            Some('『') | Some('』') | Some('《') | Some('》') => "\"",
            Some('（') | Some('〔') | Some('｟') | Some('〘') => "(",
            Some('）') | Some('〕') | Some('｠') | Some('〙') => ")",
            Some('｛') => "{",
            Some('｝') => "}",
            Some('［') | Some('【') | Some('〖') | Some('〚') => "[",
            Some('］') | Some('】') | Some('〗') | Some('〛') => "]",
            Some('〜') => "~",
            Some('？') => "?",
            Some('！') => "!",
            Some('：') => ":",
            _ => {
                i += 1;
                continue;
            }
        };

        actual_string.replace_range(i..i + replacement.len(), replacement);
        i += 1;
    }

    actual_string
}

fn get_game_type(game_title: &str) -> Option<GameType> {
    if game_title.contains("termina") {
        return Some(GameType::Termina);
    }

    None
}

#[command]
fn escape_text(text: &str) -> String {
    escape(text)
}

#[command(async)]
fn compile(
    project_path: PathBuf,
    original_path: PathBuf,
    output_path: PathBuf,
    game_title: &str,
    romanize: bool,
    shuffle_level: u64,
    disable_processing: [bool; 4],
    logging: bool,
) -> f64 {
    let start_time: Instant = Instant::now();

    let maps_path: PathBuf = project_path.join("translation/maps");
    let other_path: PathBuf = project_path.join("translation/other");
    let plugins_path: PathBuf = project_path.join("translation/plugins");
    let output_path: PathBuf = output_path.join("output/data");
    let plugins_output_path: PathBuf = output_path.join("output/js");

    let disable_custom_parsing: bool = false;
    let game_type: Option<GameType> = if disable_custom_parsing {
        None
    } else {
        get_game_type(game_title)
    };

    create_dir_all(&output_path).unwrap();
    create_dir_all(&plugins_output_path).unwrap();

    if !disable_processing[0] {
        write_maps(
            &maps_path,
            &original_path,
            &output_path,
            romanize,
            shuffle_level,
            logging,
            "",
            &game_type,
        );
    }

    if !disable_processing[1] {
        write_other(
            &other_path,
            &original_path,
            &output_path,
            romanize,
            shuffle_level,
            logging,
            "",
            &game_type,
        );
    }

    if !disable_processing[2] {
        write_system(
            &original_path.join("System.json"),
            &other_path,
            &output_path,
            romanize,
            shuffle_level,
            logging,
            "",
        );
    }

    if !disable_processing[3]
        && game_type.is_some()
        && game_type.unwrap() == GameType::Termina
        && plugins_path.join("Plugins.json").exists()
    {
        write_plugins(
            &plugins_path.join("Plugins.json"),
            &plugins_path,
            &plugins_output_path,
            shuffle_level,
            logging,
            "",
        );
    }

    start_time.elapsed().as_secs_f64()
}

#[command(async)]
fn read(
    project_path: PathBuf,
    original_path: PathBuf,
    game_title: &str,
    romanize: bool,
    disable_custom_parsing: bool,
    disable_processing: [bool; 3],
    logging: bool,
    processing_mode: u8,
) {
    let processing_type_member: ProcessingMode = match processing_mode {
        0 => ProcessingMode::Default,
        1 => ProcessingMode::Append,
        2 => ProcessingMode::Force,
        _ => unreachable!(),
    };

    let game_type: Option<GameType> = if disable_custom_parsing {
        None
    } else {
        get_game_type(game_title)
    };

    let maps_path: PathBuf = project_path.join("translation/maps");
    let other_path: PathBuf = project_path.join("translation/other");

    create_dir_all(&maps_path).unwrap();
    create_dir_all(&other_path).unwrap();

    if !disable_processing[0] {
        read_map(
            &original_path,
            &maps_path,
            romanize,
            logging,
            "",
            "",
            "",
            &game_type,
            &processing_type_member,
        );
    }

    if !disable_processing[1] {
        read_other(
            &original_path,
            &other_path,
            romanize,
            logging,
            "",
            "",
            "",
            &game_type,
            &processing_type_member,
        );
    }

    if !disable_processing[2] {
        read_system(
            &original_path.join("System.json"),
            &other_path,
            romanize,
            logging,
            "",
            "",
            "",
            &processing_type_member,
        );
    }
}

fn main() {
    seed(69);

    Builder::default()
        .invoke_handler(generate_handler![escape_text, read, compile])
        .setup(|app: &mut App| {
            let main_window: Window = app.get_window("main").unwrap();

            #[cfg(debug_assertions)]
            {
                main_window.open_devtools();
            }

            Ok(())
        })
        .run(generate_context!())
        .expect("error while running tauri application");
}
