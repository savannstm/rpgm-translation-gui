use fancy_regex::Regex;
use fnv::{FnvBuildHasher, FnvHashMap};
use indexmap::IndexSet;
use rayon::prelude::*;
use serde_json::{from_str, Value};
use std::{
    fs::{read_dir, read_to_string, write, DirEntry},
    path::{Path, PathBuf},
};

/// Reads all Map .json files of input_path and parses them into .txt files in output_path.
/// # Parameters
/// * `input_path` - path to directory than contains .json files
/// * `output_path` - path to output directory
pub fn read_map(input_path: &Path, output_path: &Path) {
    let re: Regex = Regex::new(r"^Map[0-9].*json$").unwrap();

    let files: Vec<DirEntry> = read_dir(input_path)
        .unwrap()
        .flatten()
        .filter(|entry: &DirEntry| {
            let filename: String = entry.file_name().into_string().unwrap();
            re.is_match(&filename).unwrap()
        })
        .collect();

    let maps_arr: Vec<Value> = files
        .iter()
        .map(|entry: &DirEntry| from_str(&read_to_string(entry.path()).unwrap()).unwrap())
        .collect();

    let mut lines: IndexSet<String, FnvBuildHasher> = IndexSet::default();
    let mut names_lines: IndexSet<String, FnvBuildHasher> = IndexSet::default();

    for obj in maps_arr {
        if let Some(display_name) = obj["displayName"].as_str() {
            if !display_name.is_empty() {
                names_lines.insert(display_name.to_string());
            }
        }

        //Skipping first element in array as it is null
        for event in obj["events"].as_array().unwrap().iter().skip(1) {
            if !event["pages"].is_array() {
                continue;
            }

            for page in event["pages"].as_array().unwrap().iter() {
                let mut in_seq: bool = false;
                let mut line: Vec<String> = Vec::new();

                for list in page["list"].as_array().unwrap() {
                    //401 - dialogue lines
                    //102 - dialogue choices
                    //356 - system lines (special texts)
                    let code: u16 = list["code"].as_u64().unwrap() as u16;

                    for parameter_value in list["parameters"].as_array().unwrap() {
                        if code == 401 {
                            in_seq = true;

                            if parameter_value.is_string() {
                                let parameter: &str = parameter_value.as_str().unwrap();

                                if !parameter.is_empty() {
                                    line.push(parameter.to_string());
                                }
                            }
                        } else {
                            if in_seq {
                                lines.insert(line.join(r"\#"));
                                line.clear();
                                in_seq = false;
                            }

                            match code {
                                102 => {
                                    if parameter_value.is_array() {
                                        for param_value in parameter_value.as_array().unwrap() {
                                            if param_value.is_string() {
                                                let param: &str = param_value.as_str().unwrap();

                                                if !param.is_empty() {
                                                    lines.insert(param.to_string());
                                                }
                                            }
                                        }
                                    }
                                }

                                356 => {
                                    if parameter_value.is_string() {
                                        let parameter: &str = parameter_value.as_str().unwrap();

                                        if !parameter.is_empty() {
                                            lines.insert(parameter.to_string());
                                        }
                                    }
                                }

                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    write(
        output_path.join("maps.txt"),
        lines.iter().cloned().collect::<Vec<String>>().join("\n"),
    )
    .unwrap();

    write(
        output_path.join("names.txt"),
        names_lines
            .iter()
            .cloned()
            .collect::<Vec<String>>()
            .join("\n"),
    )
    .unwrap();

    write(
        output_path.join("maps_trans.txt"),
        "\n".repeat(lines.len() - 1),
    )
    .unwrap();

    write(
        output_path.join("names_trans.txt"),
        "\n".repeat(names_lines.len() - 1),
    )
    .unwrap();
}

/// Reads all Other .json files of input_path and parses them into .txt files in output_path.
/// # Parameters
/// * `input_path` - path to directory than contains .json files
/// * `output_path` - path to output directory
pub fn read_other(input_path: &Path, output_path: &Path) {
    let re: Regex = Regex::new(r"^(?!Map|Tilesets|Animations|States|System).*json$").unwrap();

    let files: Vec<DirEntry> = read_dir(input_path)
        .unwrap()
        .flatten()
        .filter(|entry: &DirEntry| {
            re.is_match(&entry.file_name().into_string().unwrap())
                .unwrap()
        })
        .collect();

    let obj_arr_map: FnvHashMap<String, Vec<Value>> = files
        .par_iter()
        .map(|entry: &DirEntry| {
            (
                entry.file_name().into_string().unwrap(),
                from_str(&read_to_string(entry.path()).unwrap()).unwrap(),
            )
        })
        .collect();

    for (filename, obj_arr) in obj_arr_map {
        let processed_filename: String = filename[0..filename.rfind('.').unwrap()].to_lowercase();
        let mut lines: IndexSet<String, FnvBuildHasher> = IndexSet::default();

        // Other files except CommonEvents.json and Troops.json have the structure that consists
        // of name, nickname, description and note
        if !filename.starts_with("Common") && !filename.starts_with("Troops") {
            for obj in obj_arr {
                if obj["name"].is_string() {
                    let name: &str = obj["name"].as_str().unwrap();

                    if !name.is_empty() {
                        lines.insert(name.to_string());
                    }
                }

                if obj["description"].is_string() {
                    let description: &str = obj["description"].as_str().unwrap();

                    if !description.is_empty() {
                        lines.insert(description.replace('\n', r"\#"));
                    }
                }

                if obj["nickname"].is_string() {
                    let nickname: &str = obj["nickname"].as_str().unwrap();

                    if !nickname.is_empty() {
                        lines.insert(nickname.to_string());
                    }
                }

                if obj["note"].is_string() {
                    let note: &str = obj["note"].as_str().unwrap();

                    if !note.is_empty() {
                        lines.insert(note.replace('\n', r"\#"));
                    }
                }
            }
        }
        //Other files have the structure somewhat similar to Maps.json files
        else {
            //Skipping first element in array as it is null
            for obj in obj_arr.iter().skip(1) {
                //CommonEvents doesn't have pages, so we can just check if it's Troops
                let pages_length: usize = if filename.starts_with("Troops") {
                    obj["pages"].as_array().unwrap().len()
                } else {
                    1
                };

                for i in 0..pages_length {
                    let list: &Value = if pages_length != 1 {
                        &obj["pages"][i]["list"]
                    } else {
                        &obj["list"]
                    };

                    if !list.is_array() {
                        continue;
                    }

                    let mut in_seq: bool = false;
                    let mut line: Vec<String> = Vec::with_capacity(256);

                    for list in list.as_array().unwrap() {
                        //401 - dialogue lines
                        //102 - dialogue choices
                        //356 - system lines (special texts)
                        //405 - credits lines
                        let code: u16 = list["code"].as_u64().unwrap() as u16;

                        for parameter_value in list["parameters"].as_array().unwrap() {
                            if code == 401 || code == 405 {
                                in_seq = true;

                                if parameter_value.is_string() {
                                    let parameter: &str = parameter_value.as_str().unwrap();

                                    if !parameter.is_empty() {
                                        line.push(parameter.to_string());
                                    }
                                }
                            } else {
                                if in_seq {
                                    lines.insert(line.join(r"\#"));
                                    line.clear();
                                    in_seq = false;
                                }

                                match code {
                                    102 => {
                                        if parameter_value.is_array() {
                                            for param_value in parameter_value.as_array().unwrap() {
                                                if param_value.is_string() {
                                                    let param: &str = param_value.as_str().unwrap();

                                                    if !param.is_empty() {
                                                        lines.insert(param.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    356 => {
                                        if parameter_value.is_string() {
                                            let parameter: &str = parameter_value.as_str().unwrap();

                                            if !parameter.is_empty() {
                                                lines.insert(parameter.to_string());
                                            }
                                        }
                                    }

                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        write(
            output_path.join(format!("{processed_filename}.txt")),
            lines.iter().cloned().collect::<Vec<String>>().join("\n"),
        )
        .unwrap();
        write(
            output_path.join(format!("{processed_filename}_trans.txt")),
            "\n".repeat(lines.len() - 1),
        )
        .unwrap();
    }
}

/// Reads System .json file of input_path and parses it into .txt file in output_path.
/// # Parameters
/// * `input_path` - path to directory than contains .json files
/// * `output_path` - path to output directory
pub fn read_system(input_path: &Path, output_path: &Path) {
    let system_file: PathBuf = input_path.join("System.json");

    let obj: Value = from_str(&read_to_string(system_file).unwrap()).unwrap();

    let mut lines: IndexSet<String, FnvBuildHasher> = IndexSet::default();

    // Armor types names
    // Normally it's system strings, but might be needed for some purposes
    for string in obj["armorTypes"].as_array().unwrap() {
        let slice_ref: &str = string.as_str().unwrap();

        if !slice_ref.is_empty() {
            lines.insert(slice_ref.to_string());
        }
    }

    // Element types names
    // Normally it's system strings, but might be needed for some purposes
    for string in obj["elements"].as_array().unwrap() {
        let slice_ref: &str = string.as_str().unwrap();

        if !slice_ref.is_empty() {
            lines.insert(slice_ref.to_string());
        }
    }

    // Names of equipment slots
    for string in obj["equipTypes"].as_array().unwrap() {
        let slice_ref: &str = string.as_str().unwrap();

        if !slice_ref.is_empty() {
            lines.insert(slice_ref.to_string());
        }
    }

    // Game title, parsed just for fun
    // Translators may add something like "ELFISH TRANSLATION v1.0.0" to the title
    lines.insert(obj["gameTitle"].as_str().unwrap().to_string());

    // Names of battle options
    for string in obj["skillTypes"].as_array().unwrap() {
        let slice_ref: &str = string.as_str().unwrap();

        if !slice_ref.is_empty() {
            lines.insert(string.to_string());
        }
    }

    // Game terms vocabulary
    for (key, value) in obj["terms"].as_object().unwrap() {
        if key != "messages" {
            for string in value.as_array().unwrap() {
                if string.is_string() {
                    let slice_ref: &str = string.as_str().unwrap();

                    if !slice_ref.is_empty() {
                        lines.insert(slice_ref.to_string());
                    }
                }
            }
        } else {
            if !value.is_object() {
                continue;
            }

            for message_string in value.as_object().unwrap().values() {
                let slice_ref: &str = message_string.as_str().unwrap();

                if !slice_ref.is_empty() {
                    lines.insert(slice_ref.to_string());
                }
            }
        }
    }

    // Weapon types names
    // Normally it's system strings, but might be needed for some purposes
    for string in obj["weaponTypes"].as_array().unwrap() {
        let slice_ref: &str = string.as_str().unwrap();

        if !slice_ref.is_empty() {
            lines.insert(slice_ref.to_string());
        }
    }

    write(
        output_path.join("system.txt"),
        lines.iter().cloned().collect::<Vec<String>>().join("\n"),
    )
    .unwrap();
    write(
        output_path.join("system_trans.txt"),
        "\n".repeat(lines.len() - 1),
    )
    .unwrap();
}

// read_plugins is not implemented and will NEVER be, as plugins can differ from each other incredibly.
// Change plugins.js with your own hands.
