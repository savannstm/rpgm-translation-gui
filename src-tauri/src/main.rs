#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use regex::escape;
use serde_json::from_str;
use std::{fs::create_dir_all, path::PathBuf};
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

#[command]
fn escape_text(text: String) -> String {
    escape(&text)
}

fn main() {
    Builder::default()
        .invoke_handler(generate_handler![escape_text])
        .setup(|app: &mut App| {
            let main_window: Window = app.get_window("main").unwrap();
            let main_window_clone: Window = app.get_window("main").unwrap();
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

                    create_dir_all(&paths.output).unwrap();
                    create_dir_all(&paths.plugins_output).unwrap();

                    write_maps(&paths.maps, &paths.original, &paths.output);
                    write_other(&paths.other, &paths.original, &paths.output);
                    write_system(
                        &paths.other.join("System.json"),
                        &paths.other,
                        &paths.output,
                    );
                    write_plugins(
                        &paths.plugins.join("Plugins.json"),
                        &paths.plugins,
                        &paths.plugins_output,
                    );

                    main_window.emit("compile-finished", "").unwrap();
                });

            app.get_window("main")
                .unwrap()
                .listen("read", move |event: Event| {
                    let paths: [PathBuf; 2] = from_str(event.payload().unwrap()).unwrap();

                    let path: &PathBuf = &paths[0];
                    let original_folder: &PathBuf = &paths[1];

                    let original_path: PathBuf = path.join(original_folder);
                    let maps_path: PathBuf = path.join("translation/maps");
                    let other_path: PathBuf = path.join("translation/other");

                    create_dir_all(&maps_path).unwrap();
                    create_dir_all(&other_path).unwrap();

                    read_map(&original_path, &maps_path);
                    read_other(&original_path, &other_path);
                    read_system(&original_path, &other_path);

                    main_window_clone.emit("read-finished", "").unwrap();
                });

            Ok(())
        })
        .run(generate_context!())
        .expect("error while running tauri application");
}
