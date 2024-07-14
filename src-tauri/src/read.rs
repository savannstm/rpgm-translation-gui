use fancy_regex::Regex;
use indexmap::IndexSet;
use rayon::prelude::*;
use serde_json::{from_str, Value};
use std::{
    collections::HashMap,
    fs::{read_dir, read_to_string, write, DirEntry},
    hash::BuildHasherDefault,
    path::Path,
};
use xxhash_rust::xxh3::Xxh3;

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn parse_parameter<'a>(
    code: u16,
    mut parameter: &'a str,
    game_type: Option<&str>,
) -> Option<&'a str> {
    if let Some(game_type) = game_type {
        match code {
            401 | 405 => match game_type {
                // Implement custom parsing
                _ => {}
            },
            102 => match game_type {
                // Implement custom parsing
                _ => {}
            },
            356 => match game_type {
                "termina" => {
                    if !parameter.starts_with("GabText")
                        && (!parameter.starts_with("choice_text") || parameter.ends_with("????"))
                    {
                        return None;
                    }
                }
                // Implement custom parsing
                _ => {}
            },
            _ => unreachable!(),
        }
    }

    Some(parameter)
}

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn parse_variable(
    mut variable: &str,
    name: &str,
    filename: &str,
    game_type: Option<&str>,
) -> Option<String> {
    if let Some(game_type) = game_type {
        match name {
            "name" => match game_type {
                _ => {}
            },
            "nickname" => match game_type {
                _ => {}
            },
            "description" => match game_type {
                _ => {}
            },
            "note" => match game_type {
                "termina" => {
                    if !filename.starts_with("Common") && !filename.starts_with("Troops") {
                        if filename.starts_with("Items") {
                            for string in [
                                "<Menu Category: Items>",
                                "<Menu Category: Food>",
                                "<Menu Category: Healing>",
                                "<Menu Category: Body bag>",
                            ] {
                                if variable.contains(string) {
                                    return Some(string.to_string());
                                }
                            }
                        }

                        if filename.starts_with("Classes") {
                            return Some(variable.to_string());
                        }

                        if filename.starts_with("Armors") && !variable.starts_with("///") {
                            return Some(variable.to_string());
                        }

                        return None;
                    }
                }
                _ => {}
            },
            _ => unreachable!(),
        }
    }

    Some(variable.to_string())
}

/// Reads all Map .json files of input_path and parses them into .txt files in output_path.
/// # Parameters
/// * `input_path` - path to directory than contains .json files
/// * `output_path` - path to output directory
pub fn read_map(maps_path: &Path, output_path: &Path, game_type: Option<&str>) {
    let maps_output_path: &Path = &output_path.join("maps.txt");
    let maps_trans_output_path: &Path = &output_path.join("maps_trans.txt");
    let names_output_path: &Path = &output_path.join("names.txt");
    let names_trans_output_path: &Path = &output_path.join("names_trans.txt");

    let select_maps_re: Regex = Regex::new(r"^Map[0-9].*json$").unwrap();

    let maps_files: Vec<DirEntry> = read_dir(maps_path)
        .unwrap()
        .flatten()
        .filter(|entry: &DirEntry| {
            let filename: String = entry.file_name().into_string().unwrap();
            select_maps_re.is_match(&filename).unwrap()
        })
        .collect();

    let maps_obj_map: HashMap<String, Value, BuildHasherDefault<Xxh3>> = maps_files
        .into_iter()
        .map(|entry: DirEntry| {
            (
                entry.file_name().into_string().unwrap(),
                from_str(&read_to_string(entry.path()).unwrap()).unwrap(),
            )
        })
        .collect();

    let mut maps_lines: IndexSet<String, BuildHasherDefault<Xxh3>> = IndexSet::default();
    let mut names_lines: IndexSet<String, BuildHasherDefault<Xxh3>> = IndexSet::default();

    for obj in maps_obj_map.into_values() {
        if let Some(display_name) = obj["displayName"].as_str() {
            if !display_name.is_empty() {
                let display_name_string: String = display_name.to_string();
                names_lines.insert(display_name_string);
            }
        }

        //Skipping first element in array as it is null
        for event in obj["events"].as_array().unwrap().iter().skip(1) {
            if !event["pages"].is_array() {
                continue;
            }

            for page in event["pages"].as_array().unwrap().iter() {
                let mut in_sequence: bool = false;
                let mut line: Vec<String> = Vec::with_capacity(4);

                for list in page["list"].as_array().unwrap() {
                    //401 - dialogue lines
                    //102 - dialogue choices
                    //356 - system lines (special texts)
                    let code: u16 = list["code"].as_u64().unwrap() as u16;

                    for parameter_value in list["parameters"].as_array().unwrap() {
                        if code == 401 {
                            in_sequence = true;

                            if parameter_value.is_string() {
                                let parameter_str: &str = parameter_value.as_str().unwrap();

                                if !parameter_str.is_empty() {
                                    let parsed: Option<&str> =
                                        parse_parameter(code, parameter_str, game_type);

                                    if let Some(parsed) = parsed {
                                        line.push(parsed.to_string());
                                    }
                                }
                            }
                        } else {
                            if in_sequence {
                                let joined: String = line.join(r"\#").trim().to_string();
                                maps_lines.insert(joined);

                                line.clear();
                                in_sequence = false;
                            }

                            match code {
                                102 => {
                                    if parameter_value.is_array() {
                                        for subparameter_value in
                                            parameter_value.as_array().unwrap()
                                        {
                                            if subparameter_value.is_string() {
                                                let subparameter_str: &str =
                                                    subparameter_value.as_str().unwrap().trim();

                                                if !subparameter_str.is_empty() {
                                                    let parsed: Option<&str> = parse_parameter(
                                                        code,
                                                        subparameter_str,
                                                        game_type,
                                                    );

                                                    if let Some(parsed) = parsed {
                                                        let parsed_string: String =
                                                            parsed.to_string();

                                                        maps_lines.insert(parsed_string);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                356 => {
                                    if parameter_value.is_string() {
                                        let parameter_str: &str =
                                            parameter_value.as_str().unwrap().trim();

                                        if !parameter_str.is_empty() {
                                            let parsed: Option<&str> =
                                                parse_parameter(code, parameter_str, game_type);

                                            if let Some(parsed) = parsed {
                                                let parsed_string: String = parsed.to_string();

                                                maps_lines.insert(parsed_string);
                                            }
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

    let (
        maps_original_content,
        maps_translated_content,
        names_original_content,
        names_translated_content,
    ) = {
        let maps_length: usize = maps_lines.len() - 1;
        let names_length: usize = names_lines.len() - 1;
        (
            maps_lines.into_iter().collect::<Vec<String>>().join("\n"),
            "\n".repeat(maps_length),
            names_lines.into_iter().collect::<Vec<String>>().join("\n"),
            "\n".repeat(names_length),
        )
    };

    write(maps_output_path, maps_original_content).unwrap();
    write(maps_trans_output_path, maps_translated_content).unwrap();
    write(names_output_path, names_original_content).unwrap();
    write(names_trans_output_path, names_translated_content).unwrap();
}

/// Reads all Other .json files of input_path and parses them into .txt files in output_path.
/// # Parameters
/// * `input_path` - path to directory than contains .json files
/// * `output_path` - path to output directory
pub fn read_other(other_path: &Path, output_path: &Path, game_type: Option<&str>) {
    let select_other_re: Regex =
        Regex::new(r"^(?!Map|Tilesets|Animations|States|System).*json$").unwrap();

    let other_files: Vec<DirEntry> = read_dir(other_path)
        .unwrap()
        .flatten()
        .filter(|entry: &DirEntry| {
            select_other_re
                .is_match(&entry.file_name().into_string().unwrap())
                .unwrap()
        })
        .collect();

    let other_obj_arr_map: HashMap<String, Vec<Value>, BuildHasherDefault<Xxh3>> = other_files
        .par_iter()
        .map(|entry: &DirEntry| {
            (
                entry.file_name().into_string().unwrap(),
                from_str(&read_to_string(entry.path()).unwrap()).unwrap(),
            )
        })
        .collect();

    for (filename, obj_arr) in other_obj_arr_map {
        let other_processed_filename: String =
            filename[0..filename.rfind('.').unwrap()].to_lowercase();

        let other_output_path: &Path = &output_path.join(other_processed_filename.clone() + ".txt");
        let other_trans_output_path: &Path =
            &output_path.join(other_processed_filename + "_trans.txt");

        let mut other_lines: IndexSet<String, BuildHasherDefault<Xxh3>> = IndexSet::default();

        // Other files except CommonEvents.json and Troops.json have the structure that consists
        // of name, nickname, description and note
        if !filename.starts_with("Common") && !filename.starts_with("Troops") {
            for obj in obj_arr {
                for (variable, name) in [
                    (obj["name"].as_str(), "name"),
                    (obj["nickname"].as_str(), "nickname"),
                    (obj["description"].as_str(), "description"),
                    (obj["note"].as_str(), "note"),
                ] {
                    if variable.is_none() {
                        continue;
                    }

                    let variable_str: &str = variable.unwrap().trim();

                    if !variable_str.is_empty() {
                        let parsed: Option<String> =
                            parse_variable(variable_str, name, &filename, game_type);

                        if let Some(parsed) = parsed {
                            let replaced: String = parsed.replace('\n', r"\#");

                            other_lines.insert(replaced);
                        }
                    }
                }
            }
        }
        //Other files have the structure somewhat similar to Maps.json files
        else {
            //Skipping first element in array as it is null
            for obj in obj_arr.into_iter().skip(1) {
                //CommonEvents doesn't have pages, so we can just check if it's Troops
                let pages_length: u32 = if filename.starts_with("Troops") {
                    obj["pages"].as_array().unwrap().len() as u32
                } else {
                    1
                };

                for i in 0..pages_length {
                    let list: &Value = if pages_length != 1 {
                        &obj["pages"][i as usize]["list"]
                    } else {
                        &obj["list"]
                    };

                    if !list.is_array() {
                        continue;
                    }

                    let mut in_sequence: bool = false;
                    let mut line: Vec<String> = Vec::with_capacity(256);

                    for list in list.as_array().unwrap() {
                        //401 - dialogue lines
                        //102 - dialogue choices
                        //356 - system lines (special texts)
                        //405 - credits lines
                        let code: u16 = list["code"].as_u64().unwrap() as u16;

                        for parameter_value in list["parameters"].as_array().unwrap() {
                            if [401, 405].contains(&code) {
                                in_sequence = true;

                                if parameter_value.is_string() {
                                    let parameter_str: &str = parameter_value.as_str().unwrap();

                                    if !parameter_str.is_empty() {
                                        let parsed: Option<&str> =
                                            parse_parameter(code, parameter_str, game_type);

                                        if let Some(parsed) = parsed {
                                            line.push(parsed.to_string());
                                        }
                                    }
                                }
                            } else {
                                if in_sequence {
                                    let joined: String = line.join(r"\#").trim().to_string();

                                    other_lines.insert(joined);

                                    line.clear();
                                    in_sequence = false;
                                }

                                match code {
                                    102 => {
                                        if parameter_value.is_array() {
                                            for subparameter_value in
                                                parameter_value.as_array().unwrap()
                                            {
                                                if subparameter_value.is_string() {
                                                    let subparameter_str: &str =
                                                        subparameter_value.as_str().unwrap().trim();

                                                    if !subparameter_str.is_empty() {
                                                        let parsed: Option<&str> = parse_parameter(
                                                            code,
                                                            subparameter_str,
                                                            game_type,
                                                        );

                                                        if let Some(parsed) = parsed {
                                                            let parsed_string: String =
                                                                parsed.to_string();

                                                            other_lines.insert(parsed_string);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    356 => {
                                        if parameter_value.is_string() {
                                            let parameter_str: &str =
                                                parameter_value.as_str().unwrap().trim();

                                            if !parameter_str.is_empty() {
                                                let parsed: Option<&str> =
                                                    parse_parameter(code, parameter_str, game_type);

                                                if let Some(parsed) = parsed {
                                                    let parsed_string: String = parsed.to_string();

                                                    other_lines.insert(parsed_string);
                                                }
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

        let (original_content, translation_content) = {
            let length: usize = other_lines.len() - 1;
            (
                other_lines.into_iter().collect::<Vec<String>>().join("\n"),
                "\n".repeat(length),
            )
        };

        write(other_output_path, original_content).unwrap();
        write(other_trans_output_path, translation_content).unwrap();
    }
}

/// Reads System .json file of input_path and parses it into .txt file in output_path.
/// # Parameters
/// * `input_path` - path to directory than contains .json files
/// * `output_path` - path to output directory
pub fn read_system(system_file_path: &Path, output_path: &Path) {
    let system_output_path: &Path = &output_path.join("system.txt");
    let system_trans_output_path: &Path = &output_path.join("system_trans.txt");

    let system_obj: Value = from_str(&read_to_string(system_file_path).unwrap()).unwrap();

    let mut system_lines: IndexSet<String, BuildHasherDefault<Xxh3>> = IndexSet::default();

    // Armor types names
    // Normally it's system strings, but might be needed for some purposes
    for string in system_obj["armorTypes"].as_array().unwrap() {
        let str_slice: &str = string.as_str().unwrap();

        if !str_slice.is_empty() {
            let slice_string: String = str_slice.to_string();

            system_lines.insert(slice_string);
        }
    }

    // Element types names
    // Normally it's system strings, but might be needed for some purposes
    for string in system_obj["elements"].as_array().unwrap() {
        let str_slice: &str = string.as_str().unwrap();

        if !str_slice.is_empty() {
            let slice_string: String = str_slice.to_string();

            system_lines.insert(slice_string);
        }
    }

    // Names of equipment slots
    for string in system_obj["equipTypes"].as_array().unwrap() {
        let str_slice: &str = string.as_str().unwrap();

        if !str_slice.is_empty() {
            let slice_string: String = str_slice.to_string();

            system_lines.insert(slice_string);
        }
    }

    // Names of battle options
    for string in system_obj["skillTypes"].as_array().unwrap() {
        let str_slice: &str = string.as_str().unwrap();

        if !str_slice.is_empty() {
            let slice_string: String = str_slice.to_string();

            system_lines.insert(slice_string);
        }
    }

    // Game terms vocabulary
    for (key, value) in system_obj["terms"].as_object().unwrap() {
        if key != "messages" {
            for string in value.as_array().unwrap() {
                if string.is_string() {
                    let str_slice: &str = string.as_str().unwrap();

                    if !str_slice.is_empty() {
                        let slice_string: String = str_slice.to_string();

                        system_lines.insert(slice_string);
                    }
                }
            }
        } else {
            if !value.is_object() {
                continue;
            }

            for message_string in value.as_object().unwrap().values() {
                let str_slice: &str = message_string.as_str().unwrap();

                if !str_slice.is_empty() {
                    let slice_string: String = str_slice.to_string();

                    system_lines.insert(slice_string);
                }
            }
        }
    }

    // Weapon types names
    // Normally it's system strings, but might be needed for some purposes
    for string in system_obj["weaponTypes"].as_array().unwrap() {
        let str_slice: &str = string.as_str().unwrap();

        if !str_slice.is_empty() {
            let slice_string: String = str_slice.to_string();

            system_lines.insert(slice_string);
        }
    }

    // Game title, parsed just for fun
    // Translators may add something like "ELFISH TRANSLATION v1.0.0" to the title
    {
        let game_title_string: String = system_obj["gameTitle"].as_str().unwrap().to_string();
        system_lines.insert(game_title_string);
    }

    let (original_content, translated_content) = {
        let length: usize = system_lines.len() - 1;
        (
            system_lines.into_iter().collect::<Vec<String>>().join("\n"),
            "\n".repeat(length),
        )
    };

    write(system_output_path, original_content).unwrap();
    write(system_trans_output_path, translated_content).unwrap();
}

// read_plugins is not implemented and will NEVER be, as plugins can differ from each other incredibly.
// Change plugins.js with your own hands.
