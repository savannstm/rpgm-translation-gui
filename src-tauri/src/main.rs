#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::too_many_arguments)]

use fastrand::seed;
use lazy_static::lazy_static;
use regex::escape;
use regex::Regex;
use std::path::Path;
use std::{fs::create_dir_all, path::PathBuf, time::Instant};
#[cfg(debug_assertions)]
use tauri::Manager;
use tauri::{command, generate_context, generate_handler, App, Builder};

mod read;
mod write;

use read::*;
use write::*;

#[derive(PartialEq)]
enum GameType {
    Termina,
    LisaRPG,
}

#[derive(PartialEq)]
enum ProcessingMode {
    Default,
    Append,
    Force,
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

#[derive(PartialEq)]
enum EngineType {
    XP,
    VX,
    VXAce,
    New,
}

impl AsRef<EngineType> for EngineType {
    fn as_ref(&self) -> &EngineType {
        self
    }
}

impl PartialEq<EngineType> for &EngineType {
    fn eq(&self, other: &EngineType) -> bool {
        *self == other
    }
}

enum Code {
    Dialogue, // also goes for credit
    Choice,
    System,
    Unknown,
}

#[derive(PartialEq)]
enum Variable {
    Name,
    Nickname,
    Description,
    Note,
}

lazy_static! {
    pub static ref STRING_IS_ONLY_SYMBOLS_RE: Regex = Regex::new(r#"^[.()+\-:;\[\]^~%&!№$@`*\/→×？?ｘ％▼|♥♪！：〜『』「」〽。…‥＝゠、，【】［］｛｝（）〔〕｟｠〘〙〈〉《》・\\#'"<>=_ー※▶ⅠⅰⅡⅱⅢⅲⅣⅳⅤⅴⅥⅵⅦⅶⅧⅷⅨⅸⅩⅹⅪⅺⅫⅻⅬⅼⅭⅽⅮⅾⅯⅿ\s0-9]+$"#).unwrap();
    pub static ref ENDS_WITH_IF_RE: Regex = Regex::new(r" if\(.*\)$").unwrap();
    pub static ref LISA_PREFIX_RE: Regex = Regex::new(r"^(\\et\[[0-9]+\]|\\nbt)").unwrap();
    pub static ref INVALID_MULTILINE_VARIABLE_RE: Regex = Regex::new(r"^#? ?<.*>.?$|^[a-z][0-9]$").unwrap();
    pub static ref INVALID_VAIRIABLE_RE: Regex = Regex::new(r"^[+-]?[0-9]+$|^///|---|restrict eval").unwrap();
    pub static ref SELECT_WORDS_RE: Regex = Regex::new(r"\S+").unwrap();
}

pub fn romanize_string<T>(string: T) -> String
where
    T: AsRef<str>,
    std::string::String: std::convert::From<T>,
{
    let actual_string: String = String::from(string);
    let mut result: String = String::new();

    for char in actual_string.chars() {
        let replacement: &str = match char {
            '。' => ".",
            '、' | '，' => ",",
            '・' => "·",
            '゠' => "–",
            '＝' | 'ー' => "—",
            '「' | '」' | '〈' | '〉' => "'",
            '『' | '』' | '《' | '》' => "\"",
            '（' | '〔' | '｟' | '〘' => "(",
            '）' | '〕' | '｠' | '〙' => ")",
            '｛' => "{",
            '｝' => "}",
            '［' | '【' | '〖' | '〚' => "[",
            '］' | '】' | '〗' | '〛' => "]",
            '〜' => "~",
            '？' => "?",
            '！' => "!",
            '：' => ":",
            '※' => "·",
            '…' | '‥' => "...",
            '　' => " ",
            'Ⅰ' => "I",
            'ⅰ' => "i",
            'Ⅱ' => "II",
            'ⅱ' => "ii",
            'Ⅲ' => "III",
            'ⅲ' => "iii",
            'Ⅳ' => "IV",
            'ⅳ' => "iv",
            'Ⅴ' => "V",
            'ⅴ' => "v",
            'Ⅵ' => "VI",
            'ⅵ' => "vi",
            'Ⅶ' => "VII",
            'ⅶ' => "vii",
            'Ⅷ' => "VIII",
            'ⅷ' => "viii",
            'Ⅸ' => "IX",
            'ⅸ' => "ix",
            'Ⅹ' => "X",
            'ⅹ' => "x",
            'Ⅺ' => "XI",
            'ⅺ' => "xi",
            'Ⅻ' => "XII",
            'ⅻ' => "xii",
            'Ⅼ' => "L",
            'ⅼ' => "l",
            'Ⅽ' => "C",
            'ⅽ' => "c",
            'Ⅾ' => "D",
            'ⅾ' => "d",
            'Ⅿ' => "M",
            'ⅿ' => "m",
            _ => {
                result.push(char);
                continue;
            }
        };

        result.push_str(replacement);
    }

    result
}

fn get_game_type(game_title: &str) -> Option<GameType> {
    let lowercased: String = game_title.to_lowercase();

    if Regex::new(r"\btermina\b").unwrap().is_match(&lowercased) {
        Some(GameType::Termina)
    } else if Regex::new(r"\blisa\b").unwrap().is_match(&lowercased) {
        Some(GameType::LisaRPG)
    } else {
        None
    }
}

#[command]
fn escape_text(text: &str) -> String {
    escape(text)
}

#[command(async)]
fn compile(
    project_path: PathBuf,
    original_dir: PathBuf,
    output_path: PathBuf,
    game_title: &str,
    romanize: bool,
    shuffle_level: u64,
    disable_custom_processing: bool,
    disable_processing: [bool; 4],
    logging: bool,
    engine_type: u8,
) -> f64 {
    let start_time: Instant = Instant::now();

    let engine_type: &EngineType = &match engine_type {
        0 => EngineType::XP,
        1 => EngineType::VX,
        2 => EngineType::VXAce,
        3 => EngineType::New,
        _ => unreachable!(),
    };

    let data_dir: &Path = &PathBuf::from(".rpgm-translation-gui");
    let original_path: &Path = &project_path.join(original_dir);
    let maps_path: &Path = &project_path.join(data_dir).join("translation/maps");
    let other_path: &Path = &project_path.join(data_dir).join("translation/other");
    let plugins_path: &Path = &project_path.join(data_dir).join("translation/plugins");
    let data_output_path: &Path = &output_path.join(data_dir).join("output/data");
    let plugins_output_path: &Path = &output_path.join(data_dir).join("output/js");

    let game_type: &Option<GameType> = &if disable_custom_processing {
        None
    } else {
        get_game_type(game_title)
    };

    if engine_type == EngineType::New {
        println!("{}", data_output_path.display());
        create_dir_all(data_output_path).unwrap();
        create_dir_all(plugins_output_path).unwrap();
    }

    if !disable_processing[0] {
        write_maps(
            maps_path,
            original_path,
            data_output_path,
            romanize,
            shuffle_level,
            logging,
            "",
            game_type,
            engine_type,
        );
    }

    if !disable_processing[1] {
        write_other(
            other_path,
            original_path,
            data_output_path,
            romanize,
            shuffle_level,
            logging,
            "",
            game_type,
            engine_type,
        );
    }

    if !disable_processing[2] {
        write_system(
            &original_path.join("System.json"),
            other_path,
            data_output_path,
            romanize,
            shuffle_level,
            logging,
            "",
            engine_type,
        );
    }

    let plugins_file_path: &Path = &plugins_path.join("plugins.json");

    if !disable_processing[3]
        && game_type.is_some()
        && *game_type.as_ref().unwrap() == GameType::Termina
        && plugins_path.exists()
    {
        write_plugins(
            plugins_file_path,
            plugins_path,
            plugins_output_path,
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
    original_dir: PathBuf,
    game_title: &str,
    romanize: bool,
    disable_custom_processing: bool,
    disable_processing: [bool; 3],
    logging: bool,
    processing_mode: u8,
    engine_type: u8,
) {
    let processing_mode: &ProcessingMode = &match processing_mode {
        0 => ProcessingMode::Default,
        1 => ProcessingMode::Append,
        2 => ProcessingMode::Force,
        _ => unreachable!(),
    };

    let engine_type: &EngineType = &match engine_type {
        0 => EngineType::XP,
        1 => EngineType::VX,
        2 => EngineType::VXAce,
        3 => EngineType::New,
        _ => unreachable!(),
    };

    let game_type: &Option<GameType> = &if disable_custom_processing {
        None
    } else {
        get_game_type(game_title)
    };

    let data_dir: &Path = &PathBuf::from(".rpgm-translation-gui");
    let original_path: &Path = &project_path.join(original_dir);
    let maps_path: &Path = &project_path.join(data_dir).join("translation/maps");
    let other_path: &Path = &project_path.join(data_dir).join("translation/other");

    create_dir_all(maps_path).unwrap();
    create_dir_all(other_path).unwrap();

    if !disable_processing[0] {
        read_map(
            original_path,
            maps_path,
            romanize,
            logging,
            "",
            "",
            "",
            game_type,
            processing_mode,
            engine_type,
        );
    }

    if !disable_processing[1] {
        read_other(
            original_path,
            other_path,
            romanize,
            logging,
            "",
            "",
            "",
            game_type,
            processing_mode,
            engine_type,
        );
    }

    if !disable_processing[2] {
        read_system(
            &original_path.join("System.json"),
            other_path,
            romanize,
            logging,
            "",
            "",
            "",
            processing_mode,
            engine_type,
        );
    }
}

fn main() {
    seed(69);

    Builder::default()
        .invoke_handler(generate_handler![escape_text, read, compile])
        .setup(|_app: &mut App| {
            #[cfg(debug_assertions)]
            _app.get_window("main").unwrap().open_devtools();

            Ok(())
        })
        .run(generate_context!())
        .expect("error while running tauri application");
}
