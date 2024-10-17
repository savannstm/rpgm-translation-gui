#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::too_many_arguments)]

mod read;
mod write;

use lazy_static::lazy_static;
use read::*;
use regex::{escape, Regex};
use sonic_rs::{prelude::*, Object};
use std::{
    fs::{create_dir_all, File},
    io::{Read, Seek, SeekFrom},
    mem::transmute,
    path::{Path, PathBuf},
    time::Instant,
};
#[cfg(debug_assertions)]
use tauri::Manager;
use tauri::{command, generate_context, generate_handler, App, Builder};
use write::*;

static NEW_LINE: &str = r"\#";
static LINES_SEPARATOR: &str = "<#>";
static mut EXTENSION: &str = "";

lazy_static! {
    static ref STRING_IS_ONLY_SYMBOLS_RE: Regex = Regex::new(r#"^[.()+\-:;\[\]^~%&!№$@`*\/→×？?ｘ％▼|♥♪！：〜『』「」〽。…‥＝゠、，【】［］｛｝（）〔〕｟｠〘〙〈〉《》・\\#'"<>=_ー※▶ⅠⅰⅡⅱⅢⅲⅣⅳⅤⅴⅥⅵⅦⅶⅧⅷⅨⅸⅩⅹⅪⅺⅫⅻⅬⅼⅭⅽⅮⅾⅯⅿ\s0-9]+$"#).unwrap();
    static ref ENDS_WITH_IF_RE: Regex = Regex::new(r" if\(.*\)$").unwrap();
    static ref LISA_PREFIX_RE: Regex = Regex::new(r"^(\\et\[[0-9]+\]|\\nbt)").unwrap();
    static ref INVALID_MULTILINE_VARIABLE_RE: Regex = Regex::new(r"^#? ?<.*>.?$|^[a-z][0-9]$").unwrap();
    static ref INVALID_VARIABLE_RE: Regex = Regex::new(r"^[+-]?[0-9]+$|^///|---|restrict eval").unwrap();
    static ref SELECT_WORDS_RE: Regex = Regex::new(r"\S+").unwrap();
}

#[derive(PartialEq, Clone, Copy)]
enum GameType {
    Termina,
    LisaRPG,
}

#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
#[allow(dead_code)]
enum ProcessingMode {
    Force,
    Append,
    Default,
}

#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
#[allow(dead_code)]
enum EngineType {
    New,
    VXAce,
    VX,
    XP,
}

#[derive(PartialEq)]
enum Code {
    Dialogue, // also goes for credit
    Choice,
    System,
    Misc,
}

#[derive(PartialEq, Clone, Copy)]
enum Variable {
    Name,
    Nickname,
    Description,
    Message1,
    Message2,
    Message3,
    Message4,
    Note,
}

#[derive(PartialEq, Clone, Copy)]
#[repr(u8)]
#[allow(dead_code)]
enum MapsProcessingMode {
    Default = 0,
    Separate = 1,
    Preserve = 2,
}

trait EachLine {
    fn each_line(&self) -> Vec<String>;
}

// Return a Vec of strings splitted by lines (inclusive), akin to each_line in Ruby
impl EachLine for str {
    fn each_line(&self) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let mut current_line: String = String::new();

        for char in self.chars() {
            current_line.push(char);

            if char == '\n' {
                result.push(std::mem::take(&mut current_line));
            }
        }

        if !current_line.is_empty() {
            result.push(std::mem::take(&mut current_line));
        }

        result
    }
}

pub fn romanize_string(string: String) -> String {
    let mut result: String = String::with_capacity(string.capacity());

    for char in string.chars() {
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

pub fn get_object_data(object: &Object) -> String {
    match object.get(&"__type") {
        Some(object_type) => {
            if object_type.as_str().is_some_and(|_type: &str| _type == "bytes") {
                unsafe { String::from_utf8_unchecked(sonic_rs::from_value(&object["data"]).unwrap_unchecked()) }
            } else {
                String::new()
            }
        }
        None => String::new(),
    }
}

pub fn extract_strings(
    ruby_code: &str,
    mode: bool,
) -> (
    indexmap::IndexSet<String, std::hash::BuildHasherDefault<xxhash_rust::xxh3::Xxh3>>,
    Vec<usize>,
) {
    fn is_escaped(index: usize, string: &str) -> bool {
        let mut backslash_count: u8 = 0;

        for char in string[..index].chars().rev() {
            if char == '\\' {
                backslash_count += 1;
            } else {
                break;
            }
        }

        backslash_count % 2 == 1
    }

    let mut strings: indexmap::IndexSet<String, std::hash::BuildHasherDefault<xxhash_rust::xxh3::Xxh3>> =
        indexmap::IndexSet::default();
    let mut indices: Vec<usize> = Vec::new();
    let mut inside_string: bool = false;
    let mut inside_multiline_comment: bool = false;
    let mut string_start_index: usize = 0;
    let mut current_quote_type: char = '\0';
    let mut global_index: usize = 0;

    for line in ruby_code.each_line() {
        let trimmed: &str = line.trim();

        if !inside_string {
            if trimmed.starts_with('#') {
                global_index += line.len();
                continue;
            }

            if trimmed.starts_with("=begin") {
                inside_multiline_comment = true;
            } else if trimmed.starts_with("=end") {
                inside_multiline_comment = false;
            }
        }

        if inside_multiline_comment {
            global_index += line.len();
            continue;
        }

        let char_indices: std::str::CharIndices = line.char_indices();

        for (i, char) in char_indices {
            if !inside_string && char == '#' {
                break;
            }

            if !inside_string && (char == '"' || char == '\'') {
                inside_string = true;
                string_start_index = global_index + i;
                current_quote_type = char;
            } else if inside_string && char == current_quote_type && !is_escaped(i, &line) {
                let extracted_string: String = ruby_code[string_start_index + 1..global_index + i]
                    .replace("\r\n", NEW_LINE)
                    .replace('\n', NEW_LINE);

                if !strings.contains(&extracted_string) {
                    strings.insert(extracted_string);
                }

                if mode {
                    indices.push(string_start_index + 1);
                }

                inside_string = false;
                current_quote_type = '\0';
            }
        }

        global_index += line.len();
    }

    (strings, indices)
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
    maps_processing_mode: u8,
    romanize: bool,
    disable_custom_processing: bool,
    disable_processing: [bool; 4],
    logging: bool,
    engine_type: u8,
) -> f64 {
    let start_time: Instant = Instant::now();

    let maps_processing_mode: MapsProcessingMode = unsafe { transmute(maps_processing_mode) };
    let engine_type: EngineType = unsafe { transmute(engine_type) };

    let extension: &str = match engine_type {
        EngineType::New => ".json",
        EngineType::VXAce => ".rvdata2",
        EngineType::VX => ".rvdata",
        EngineType::XP => ".rxdata",
    };

    unsafe { EXTENSION = extension };

    let data_dir: &Path = &PathBuf::from(".rpgm-translation-gui");
    let original_path: &Path = &project_path.join(original_dir);
    let translation_path: &Path = &project_path.join(data_dir).join("translation");
    let (data_output_path, plugins_output_path) = if engine_type == EngineType::New {
        let plugins_output_path: PathBuf = output_path.join(data_dir).join("output/js");
        create_dir_all(&plugins_output_path).unwrap();

        (
            &output_path.join(data_dir).join("output/data"),
            Some(plugins_output_path),
        )
    } else {
        (&output_path.join(data_dir).join("output/Data"), None)
    };

    create_dir_all(data_output_path).unwrap();

    let game_type: Option<GameType> = if disable_custom_processing {
        None
    } else {
        get_game_type(game_title)
    };

    if !disable_processing[0] {
        write_maps(
            translation_path,
            original_path,
            data_output_path,
            maps_processing_mode,
            romanize,
            logging,
            game_type,
            engine_type,
            "",
        );
    }

    if !disable_processing[1] {
        write_other(
            translation_path,
            original_path,
            data_output_path,
            romanize,
            logging,
            game_type,
            engine_type,
            "",
        );
    }

    if !disable_processing[2] {
        write_system(
            &original_path.join(String::from("System") + extension),
            translation_path,
            data_output_path,
            romanize,
            logging,
            engine_type,
            "",
        );
    }

    if !disable_processing[3] {
        let plugins_file_path: &Path = &translation_path.join("plugins.json");

        if game_type.is_some_and(|game_type: GameType| game_type == GameType::Termina) && plugins_file_path.exists() {
            write_plugins(
                plugins_file_path,
                translation_path,
                &unsafe { plugins_output_path.unwrap_unchecked() },
                logging,
                "",
            );
        }

        let scripts_file_path: &Path = &original_path.join(String::from("Scripts") + extension);

        if engine_type != EngineType::New && scripts_file_path.exists() {
            write_scripts(
                scripts_file_path,
                translation_path,
                data_output_path,
                romanize,
                logging,
                "",
            );
        }
    }

    start_time.elapsed().as_secs_f64()
}

#[command(async)]
fn read(
    project_path: PathBuf,
    original_dir: PathBuf,
    game_title: &str,
    maps_processing_mode: u8,
    romanize: bool,
    disable_custom_processing: bool,
    disable_processing: [bool; 4],
    logging: bool,
    processing_mode: u8,
    engine_type: u8,
) {
    let processing_mode: ProcessingMode = unsafe { transmute(processing_mode) };
    let engine_type: EngineType = unsafe { transmute(engine_type) };
    let maps_processing_mode: MapsProcessingMode = unsafe { transmute(maps_processing_mode) };

    let extension: &str = match engine_type {
        EngineType::New => ".json",
        EngineType::VXAce => ".rvdata2",
        EngineType::VX => ".rvdata",
        EngineType::XP => ".rxdata",
    };

    unsafe { EXTENSION = extension };

    let game_type: Option<GameType> = if disable_custom_processing {
        None
    } else {
        get_game_type(game_title)
    };

    let data_dir: &Path = &PathBuf::from(".rpgm-translation-gui");
    let original_path: &Path = &project_path.join(original_dir);
    let translation_path: &Path = &project_path.join(data_dir).join("translation");

    create_dir_all(translation_path).unwrap();

    if !disable_processing[0] {
        read_map(
            original_path,
            translation_path,
            maps_processing_mode,
            romanize,
            logging,
            game_type,
            engine_type,
            processing_mode,
            ("", "", ""),
        );
    }

    if !disable_processing[1] {
        read_other(
            original_path,
            translation_path,
            romanize,
            logging,
            game_type,
            processing_mode,
            engine_type,
            ("", "", ""),
        );
    }

    if !disable_processing[2] {
        read_system(
            &original_path.join(String::from("System") + extension),
            translation_path,
            romanize,
            logging,
            processing_mode,
            engine_type,
            ("", "", ""),
        );
    }

    if !disable_processing[3] && engine_type != EngineType::New {
        read_scripts(
            &original_path.join(String::from("Scripts") + extension),
            translation_path,
            romanize,
            logging,
            "",
        );
    }
}

#[command]
fn read_last_line(file_path: PathBuf) -> String {
    let mut file: File = File::open(file_path).unwrap();
    let mut buffer: Vec<u8> = Vec::new();

    let mut position: u64 = file.seek(SeekFrom::End(0)).unwrap();

    while position > 0 {
        position -= 1;
        file.seek(SeekFrom::Start(position)).unwrap();

        let mut byte = [0; 1];
        file.read_exact(&mut byte).unwrap();

        if byte == b"\n"[..] && !buffer.is_empty() {
            break;
        }

        buffer.push(byte[0]);
    }

    buffer.reverse();

    unsafe { String::from_utf8_unchecked(buffer) }
}

fn main() {
    Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(generate_handler![escape_text, read, compile, read_last_line,])
        .setup(|_app: &mut App| {
            #[cfg(debug_assertions)]
            _app.get_webview_window("main").unwrap().open_devtools();

            Ok(())
        })
        .run(generate_context!())
        .expect("error while running tauri application");
}
