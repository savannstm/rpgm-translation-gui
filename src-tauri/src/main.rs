#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use fastrand::seed;
use regex::escape;
use sonic_rs::{from_str, Array, JsonContainerTrait, JsonValueTrait};
use std::{fs::create_dir_all, path::PathBuf};
use tauri::{command, generate_context, generate_handler, App, Builder, Event, Manager, Window};

mod read;
mod write;

use read::*;
use write::*;

#[derive(PartialEq)]
enum GameType {
    Termina,
}

#[derive(PartialEq)]
enum ProcessingType {
    Force,
    Append,
    Default,
}

impl AsRef<ProcessingType> for ProcessingType {
    fn as_ref(&self) -> &ProcessingType {
        self
    }
}

impl PartialEq<ProcessingType> for &ProcessingType {
    fn eq(&self, other: &ProcessingType) -> bool {
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

#[command]
fn read(
    project_dir: PathBuf,
    game_title: &str,
    logging: bool,
    disable_custom_parsing: bool,
    processing_type: u8,
) {
    let processing_type_member: ProcessingType = match processing_type {
        0 => ProcessingType::Default,
        1 => ProcessingType::Append,
        2 => ProcessingType::Force,
        _ => unreachable!(),
    };

    let game_type: Option<GameType> = if disable_custom_parsing {
        None
    } else {
        get_game_type(game_title)
    };

    let original_path: PathBuf = project_dir.join("original");
    let maps_path: PathBuf = project_dir.join("translation/maps");
    let other_path: PathBuf = project_dir.join("translation/other");

    create_dir_all(&maps_path).unwrap();
    create_dir_all(&other_path).unwrap();

    read_map(
        &original_path,
        &maps_path,
        logging,
        &game_type,
        &processing_type_member,
    );

    read_other(
        &original_path,
        &other_path,
        logging,
        &game_type,
        &processing_type_member,
    );

    read_system(
        &original_path.join("System.json"),
        &other_path,
        logging,
        &processing_type_member,
    );
}

fn main() {
    seed(69);

    Builder::default()
        .invoke_handler(generate_handler![escape_text, read])
        .setup(|app: &mut App| {
            let main_window: Window = app.get_window("main").unwrap();
            #[cfg(debug_assertions)]
            {
                main_window.open_devtools();
            }

            app.get_window("main")
                .unwrap()
                .listen("compile", move |event: Event| {
                    let options_arr: Array = from_str(event.payload().unwrap()).unwrap();

                    let path: PathBuf = PathBuf::from(options_arr[0].as_str().unwrap());
                    let output_path: PathBuf = PathBuf::from(options_arr[1].as_str().unwrap());
                    let game_title: &str = options_arr[2].as_str().unwrap();
                    let logging: bool = options_arr[3].as_bool().unwrap();
                    let shuffle_level: u64 = options_arr[4].as_u64().unwrap();
                    let disable_processing: &Array = options_arr[5].as_array().unwrap();

                    let original_path: PathBuf = path.join("original");
                    let maps_path: PathBuf = path.join("translation/maps");
                    let other_path: PathBuf = path.join("translation/other");
                    let plugins_path: PathBuf = path.join("translation/plugins");
                    let output_path: PathBuf = output_path.join("output/data");
                    let plugins_output_path: PathBuf = output_path.join("output/js");

                    let system_file_path: PathBuf = other_path.join("System.json");

                    let disable_custom_parsing: bool = false;
                    let game_type: Option<GameType> = if disable_custom_parsing {
                        None
                    } else {
                        get_game_type(game_title)
                    };

                    create_dir_all(&output_path).unwrap();
                    create_dir_all(&plugins_output_path).unwrap();

                    if !disable_processing[0].as_bool().unwrap() {
                        write_maps(
                            &maps_path,
                            &original_path,
                            &output_path,
                            shuffle_level,
                            logging,
                            &game_type,
                        );
                    }

                    if !disable_processing[1].as_bool().unwrap() {
                        write_other(
                            &other_path,
                            &original_path,
                            &output_path,
                            shuffle_level,
                            logging,
                            &game_type,
                        );
                    }

                    if !disable_processing[2].as_bool().unwrap() {
                        write_system(
                            &system_file_path,
                            &other_path,
                            &output_path,
                            shuffle_level,
                            logging,
                        );
                    }

                    if !disable_processing[3].as_bool().unwrap() {
                        write_plugins(
                            &plugins_path.join("Plugins.json"),
                            &plugins_path,
                            &plugins_output_path,
                            shuffle_level,
                            logging,
                        );
                    }

                    main_window.emit("compile-finished", "").unwrap();
                });
            Ok(())
        })
        .run(generate_context!())
        .expect("error while running tauri application");
}
