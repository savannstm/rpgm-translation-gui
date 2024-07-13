#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use regex::escape;
use serde_json::{from_str, Value};
use std::{
    fs::{create_dir_all, read_to_string},
    path::{Path, PathBuf},
};
use tauri::{command, generate_context, generate_handler, App, Builder, Event, Manager, Window};

mod write;
use write::*;

mod read;
use read::*;

struct Paths {
    original: PathBuf,
    output: PathBuf,
    maps: PathBuf,
    other: PathBuf,
    plugins: PathBuf,
    plugins_output: PathBuf,
}

fn get_game_type(system_file_path: &Path) -> Option<&'static str> {
    let system_obj: Value = from_str(&read_to_string(system_file_path).unwrap()).unwrap();
    let game_title: String = system_obj["gameTitle"]
        .as_str()
        .unwrap()
        .to_string()
        .to_lowercase();

    if game_title.contains("termina") {
        return Some("termina");
    }

    None
}

#[command]
fn escape_text(text: &str) -> String {
    escape(text)
}

#[command]
fn read(project_dir: PathBuf, original_dir: PathBuf, disable_custom_parsing: bool) {
    let system_file_path: PathBuf = original_dir.join("System.json");
    let game_type: Option<&str> = if disable_custom_parsing {
        None
    } else {
        get_game_type(&system_file_path)
    };

    let maps_path: PathBuf = project_dir.join("translation/maps");
    let other_path: PathBuf = project_dir.join("translation/other");

    create_dir_all(&maps_path).unwrap();
    create_dir_all(&other_path).unwrap();

    read_map(&original_dir, &maps_path, game_type);
    read_other(&original_dir, &other_path, game_type);
    read_system(&system_file_path, &other_path);
}

fn main() {
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
                    let path: PathBuf = event.payload().unwrap().replace('"', "").into();

                    let paths: Paths = Paths {
                        original: path.join("original"),
                        maps: path.join("translation/maps"),
                        other: path.join("translation/other"),
                        plugins: path.join("translation/plugins"),
                        output: path.join("output/data"),
                        plugins_output: path.join("output/js"),
                    };

                    let system_file_path: PathBuf = paths.other.join("System.json");

                    let disable_custom_parsing: bool = false;
                    let game_type: Option<&str> = if disable_custom_parsing {
                        None
                    } else {
                        get_game_type(&system_file_path)
                    };

                    create_dir_all(&paths.output).unwrap();
                    create_dir_all(&paths.plugins_output).unwrap();

                    write_maps(&paths.maps, &paths.original, &paths.output, game_type);
                    write_other(&paths.other, &paths.original, &paths.output, game_type);
                    write_system(&system_file_path, &paths.other, &paths.output);
                    write_plugins(
                        &paths.plugins.join("Plugins.json"),
                        &paths.plugins,
                        &paths.plugins_output,
                    );

                    main_window.emit("compile-finished", "").unwrap();
                });
            Ok(())
        })
        .run(generate_context!())
        .expect("error while running tauri application");
}
