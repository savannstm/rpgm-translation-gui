use crate::{Code, GameType, ProcessingType, Variable};
use indexmap::{IndexMap, IndexSet};
use rayon::prelude::*;
use sonic_rs::{from_str, Array, JsonContainerTrait, JsonValueTrait, Object, Value};
use std::{
    ffi::OsString,
    fs::{read_dir, read_to_string, write, DirEntry},
    hash::BuildHasherDefault,
    path::Path,
    str::from_utf8_unchecked,
};
use xxhash_rust::xxh3::Xxh3;

pub static mut LOG_MSG: &str = "";
pub static mut FILE_ALREADY_PARSED: &str = "";
pub static mut FILE_IS_NOT_PARSED: &str = "";

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn parse_parameter(
    code: Code,
    mut parameter: &str,
    game_type: &Option<GameType>,
) -> Option<String> {
    if let Some(game_type) = game_type {
        match code {
            Code::Dialogue => match game_type {
                // Implement custom parsing
                _ => {}
            },
            Code::Choice => match game_type {
                // Implement custom parsing
                _ => {}
            },
            Code::System => match game_type {
                GameType::Termina => {
                    if !parameter.starts_with("Gab")
                        && (!parameter.starts_with("choice_text") || parameter.ends_with("????"))
                    {
                        return None;
                    }
                }
            },
            Code::Unknown => {}
        }
    }

    Some(parameter.to_string())
}

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn parse_variable(
    mut variable: &str,
    name: Variable,
    filename: &str,
    game_type: &Option<GameType>,
) -> Option<String> {
    if let Some(game_type) = game_type {
        match name {
            Variable::Name => match game_type {
                _ => {}
            },
            Variable::Nickname => match game_type {
                _ => {}
            },
            Variable::Description => match game_type {
                _ => {}
            },
            Variable::Note => match game_type {
                GameType::Termina => {
                    if !filename.starts_with("Co") && !filename.starts_with("Tr") {
                        if filename.starts_with("It") {
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
                        } else if filename.starts_with("Cl")
                            || (filename.starts_with("Ar") && !variable.starts_with("///"))
                        {
                            return Some(variable.to_string());
                        }

                        return None;
                    }
                }
            },
        }
    }

    Some(variable.to_string())
}

// ! In current implementation, function performs extremely inefficient inserting of owned string to both hashmap and a hashset
/// Reads all Map .json files of map_path and parses them into .txt files in output_path.
/// # Parameters
/// * `map_path` - path to directory than contains .json files
/// * `output_path` - path to output directory
/// * `logging` - whether to log or not
/// * `game_type` - game type for custom parsing
/// * `processing_type` - whether to read in default mode, force rewrite or append new text to existing files
pub fn read_map(
    maps_path: &Path,
    output_path: &Path,
    logging: bool,
    game_type: &Option<GameType>,
    mut processing_type: &ProcessingType,
) {
    let maps_output_path: &Path = &output_path.join("maps.txt");
    let maps_trans_output_path: &Path = &output_path.join("maps_trans.txt");
    let names_output_path: &Path = &output_path.join("names.txt");
    let names_trans_output_path: &Path = &output_path.join("names_trans.txt");

    if processing_type == ProcessingType::Default && maps_trans_output_path.exists() {
        println!("maps_trans.txt {}", unsafe { FILE_ALREADY_PARSED });
        return;
    }

    let maps_files: Vec<DirEntry> = read_dir(maps_path)
        .unwrap()
        .filter_map(|entry: Result<DirEntry, _>| match entry {
            Ok(entry) => {
                let filename_os_string = entry.file_name();
                let filename: &str =
                    unsafe { from_utf8_unchecked(filename_os_string.as_encoded_bytes()) };

                let slice: char;
                unsafe {
                    slice = *filename.as_bytes().get_unchecked(4) as char;
                }

                if filename.starts_with("Map")
                    && slice.is_ascii_digit()
                    && filename.ends_with("json")
                {
                    Some(entry)
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect();

    let maps_obj_vec: Vec<(String, Object)> = maps_files
        .into_iter()
        .map(|entry: DirEntry| {
            (
                unsafe { from_utf8_unchecked(entry.file_name().as_encoded_bytes()).to_string() },
                from_str(&read_to_string(entry.path()).unwrap()).unwrap(),
            )
        })
        .collect();

    let mut maps_lines: IndexSet<String, BuildHasherDefault<Xxh3>> = IndexSet::default();
    let mut names_lines: IndexSet<String, BuildHasherDefault<Xxh3>> = IndexSet::default();

    let mut maps_translation_map: IndexMap<String, String, BuildHasherDefault<Xxh3>> =
        IndexMap::default();
    let mut names_translation_map: IndexMap<String, String, BuildHasherDefault<Xxh3>> =
        IndexMap::default();

    if processing_type == ProcessingType::Append {
        if maps_trans_output_path.exists() {
            for (original, translated) in read_to_string(maps_output_path)
                .unwrap()
                .split('\n')
                .zip(read_to_string(maps_trans_output_path).unwrap().split('\n'))
            {
                maps_translation_map.insert(original.to_string(), translated.to_string());
            }

            for (original, translated) in read_to_string(names_output_path)
                .unwrap()
                .split('\n')
                .zip(read_to_string(names_trans_output_path).unwrap().split('\n'))
            {
                names_translation_map.insert(original.to_string(), translated.to_string());
            }
        } else {
            println!("{}", unsafe { FILE_IS_NOT_PARSED });
            processing_type = &ProcessingType::Default;
        }
    }

    // 401 - dialogue lines
    // 102 - dialogue choices array
    // 356 - system lines (special texts)
    // 324 - i don't know what is it but it's some used in-game lines
    const ALLOWED_CODES: [u64; 4] = [401, 102, 356, 324];

    for (filename, obj) in maps_obj_vec.into_iter() {
        if let Some(display_name) = obj["displayName"].as_str() {
            if !display_name.is_empty() {
                let display_name_string: String = display_name.to_string();

                if processing_type == ProcessingType::Append
                    && !names_translation_map.contains_key(&display_name_string)
                {
                    names_translation_map.shift_insert(
                        names_lines.len(),
                        display_name_string.clone(),
                        "".into(),
                    );
                }

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
                    let code: u64 = list["code"].as_u64().unwrap();

                    if !ALLOWED_CODES.contains(&code) {
                        if in_sequence {
                            let joined: String = line.join(r"\#").trim().to_string();

                            if processing_type == ProcessingType::Append
                                && !maps_translation_map.contains_key(&joined)
                            {
                                maps_translation_map.shift_insert(
                                    maps_lines.len(),
                                    joined.clone(),
                                    "".into(),
                                );
                            }

                            maps_lines.insert(joined);

                            line.clear();
                            in_sequence = false;
                        }
                        continue;
                    }

                    let parameters: &Array = list["parameters"].as_array().unwrap();

                    if code == 401 {
                        if let Some(parameter_str) = parameters[0].as_str() {
                            if !parameter_str.is_empty() {
                                let parsed: Option<String> =
                                    parse_parameter(Code::Dialogue, parameter_str, game_type);

                                if let Some(parsed) = parsed {
                                    in_sequence = true;
                                    line.push(parsed);
                                }
                            }
                        }
                    } else if parameters[0].is_array() {
                        for i in 0..parameters[0].as_array().unwrap().len() {
                            if let Some(subparameter_str) = parameters[0][i].as_str() {
                                if !subparameter_str.is_empty() {
                                    let parsed: Option<String> =
                                        parse_parameter(Code::Choice, subparameter_str, game_type);

                                    if let Some(parsed) = parsed {
                                        if processing_type == ProcessingType::Append
                                            && !maps_translation_map.contains_key(&parsed)
                                        {
                                            maps_translation_map.shift_insert(
                                                maps_lines.len(),
                                                parsed.clone(),
                                                "".into(),
                                            );
                                        }

                                        maps_lines.insert(parsed);
                                    }
                                }
                            }
                        }
                    } else if let Some(parameter_str) = parameters[0].as_str() {
                        if !parameter_str.is_empty() {
                            let parsed: Option<String> =
                                parse_parameter(Code::System, parameter_str, game_type);

                            if let Some(parsed) = parsed {
                                if processing_type == ProcessingType::Append
                                    && !maps_translation_map.contains_key(&parsed)
                                {
                                    maps_translation_map.shift_insert(
                                        maps_lines.len(),
                                        parsed.clone(),
                                        "".into(),
                                    );
                                }

                                maps_lines.insert(parsed);
                            }
                        }
                    } else if let Some(parameter_str) = parameters[1].as_str() {
                        if !parameter_str.is_empty() {
                            let parsed: Option<String> =
                                parse_parameter(Code::Unknown, parameter_str, game_type);

                            if let Some(parsed) = parsed {
                                if processing_type == ProcessingType::Append
                                    && !maps_translation_map.contains_key(&parsed)
                                {
                                    maps_translation_map.shift_insert(
                                        maps_lines.len(),
                                        parsed.clone(),
                                        "".into(),
                                    );
                                }

                                maps_lines.insert(parsed);
                            }
                        }
                    }
                }
            }
        }

        if logging {
            println!("{} {filename}.", unsafe { LOG_MSG });
        }
    }

    let (
        maps_original_content,
        maps_translated_content,
        names_original_content,
        names_translated_content,
    ) = if processing_type == ProcessingType::Append {
        let maps_collected: (Vec<String>, Vec<String>) = maps_translation_map.into_iter().unzip();
        let names_collected: (Vec<String>, Vec<String>) = names_translation_map.into_iter().unzip();
        (
            maps_collected.0.join("\n"),
            maps_collected.1.join("\n"),
            names_collected.0.join("\n"),
            names_collected.1.join("\n"),
        )
    } else {
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

// ! In current implementation, function performs extremely inefficient inserting of owned string to both hashmap and a hashset
/// Reads all Other .json files of other_path and parses them into .txt files in output_path.
/// # Parameters
/// * `other_path` - path to directory than contains .json files
/// * `output_path` - path to output directory
/// * `logging` - whether to log or not
/// * `game_type` - game type for custom parsing
/// * `processing_type` - whether to read in default mode, force rewrite or append new text to existing files
pub fn read_other(
    other_path: &Path,
    output_path: &Path,
    logging: bool,
    game_type: &Option<GameType>,
    processing_type: &ProcessingType,
) {
    let other_files: Vec<DirEntry> = read_dir(other_path)
        .unwrap()
        .filter_map(|entry: Result<DirEntry, _>| match entry {
            Ok(entry) => {
                let filename_os_string: OsString = entry.file_name();
                let filename: &str =
                    unsafe { from_utf8_unchecked(filename_os_string.as_encoded_bytes()) };
                let (real_name, extension) = filename.split_once('.').unwrap();

                if !real_name.starts_with("Map")
                    && !matches!(real_name, "Tilesets" | "Animations" | "States" | "System")
                    && extension == "json"
                {
                    Some(entry)
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect();

    let other_obj_arr_map: Vec<(String, Array)> = other_files
        .into_par_iter()
        .map(|entry: DirEntry| {
            (
                unsafe { from_utf8_unchecked(entry.file_name().as_encoded_bytes()).to_string() },
                from_str(&read_to_string(entry.path()).unwrap()).unwrap(),
            )
        })
        .collect();

    let mut inner_processing_type: &ProcessingType = processing_type;

    // 401 - dialogue lines
    // 405 - credits lines
    // 102 - dialogue choices array
    // 356 - system lines (special texts)
    // 324 - i don't know what is it but it's some used in-game lines
    const ALLOWED_CODES: [u64; 5] = [401, 405, 356, 102, 324];

    for (filename, obj_arr) in other_obj_arr_map.into_iter() {
        let other_processed_filename: String =
            filename[0..filename.rfind('.').unwrap()].to_lowercase();

        let other_output_path: &Path = &output_path.join(other_processed_filename.clone() + ".txt");
        let other_trans_output_path: &Path =
            &output_path.join(other_processed_filename + "_trans.txt");

        if processing_type == ProcessingType::Default && other_trans_output_path.exists() {
            println!(
                "{} {}",
                unsafe {
                    from_utf8_unchecked(
                        other_trans_output_path
                            .file_name()
                            .unwrap()
                            .as_encoded_bytes(),
                    )
                },
                unsafe { FILE_ALREADY_PARSED }
            );
            continue;
        }

        let mut other_lines: IndexSet<String, BuildHasherDefault<Xxh3>> = IndexSet::default();
        let mut other_translation_map: IndexMap<String, String, BuildHasherDefault<Xxh3>> =
            IndexMap::default();

        if processing_type == ProcessingType::Append {
            if other_trans_output_path.exists() {
                for (original, translated) in read_to_string(other_output_path)
                    .unwrap()
                    .split('\n')
                    .zip(read_to_string(other_trans_output_path).unwrap().split('\n'))
                {
                    other_translation_map.insert(original.to_string(), translated.to_string());
                }
            } else {
                println!("{}", unsafe { FILE_IS_NOT_PARSED });
                inner_processing_type = &ProcessingType::Default;
            }
        }

        // Other files except CommonEvents.json and Troops.json have the structure that consists
        // of name, nickname, description and note
        if !filename.starts_with("Co") && !filename.starts_with("Tr") {
            for obj in obj_arr {
                for (variable, name) in [
                    (obj["name"].as_str(), Variable::Name),
                    (obj["nickname"].as_str(), Variable::Nickname),
                    (obj["description"].as_str(), Variable::Description),
                    (obj["note"].as_str(), Variable::Note),
                ] {
                    if let Some(mut variable_str) = variable {
                        variable_str = variable_str.trim();

                        if !variable_str.is_empty() {
                            let parsed: Option<String> =
                                parse_variable(variable_str, name, &filename, game_type);

                            if let Some(parsed) = parsed {
                                let replaced: String = parsed.replace('\n', r"\#");

                                if inner_processing_type == ProcessingType::Append
                                    && !other_translation_map.contains_key(&replaced)
                                {
                                    other_translation_map.shift_insert(
                                        other_lines.len(),
                                        replaced.clone(),
                                        "".into(),
                                    );
                                }

                                other_lines.insert(replaced);
                            }
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

                    let mut in_sequence: bool = false;
                    let mut line: Vec<String> = Vec::with_capacity(256); // well it's 256 because of credits and i won't change it

                    for list in list.as_array().unwrap() {
                        let code: u64 = list["code"].as_u64().unwrap();

                        if !ALLOWED_CODES.contains(&code) {
                            if in_sequence {
                                let joined: String = line.join(r"\#").trim().to_string();

                                if processing_type == ProcessingType::Append
                                    && !other_translation_map.contains_key(&joined)
                                {
                                    other_translation_map.shift_insert(
                                        other_lines.len(),
                                        joined.clone(),
                                        "".into(),
                                    );
                                }

                                other_lines.insert(joined);

                                line.clear();
                                in_sequence = false;
                            }
                            continue;
                        }

                        let parameters: &Array = list["parameters"].as_array().unwrap();

                        if [401, 405].contains(&code) {
                            if let Some(parameter_str) = parameters[0].as_str() {
                                if !parameter_str.is_empty() {
                                    let parsed: Option<String> =
                                        parse_parameter(Code::Dialogue, parameter_str, game_type);

                                    if let Some(parsed) = parsed {
                                        in_sequence = true;
                                        line.push(parsed);
                                    }
                                }
                            }
                        } else if parameters[0].is_array() {
                            for i in 0..parameters[0].as_array().unwrap().len() {
                                if let Some(subparameter_str) = parameters[0][i].as_str() {
                                    if !subparameter_str.is_empty() {
                                        let parsed: Option<String> = parse_parameter(
                                            Code::Choice,
                                            subparameter_str,
                                            game_type,
                                        );

                                        if let Some(parsed) = parsed {
                                            if processing_type == ProcessingType::Append
                                                && !other_translation_map.contains_key(&parsed)
                                            {
                                                other_translation_map.shift_insert(
                                                    other_lines.len(),
                                                    parsed.clone(),
                                                    "".into(),
                                                );
                                            }

                                            other_lines.insert(parsed);
                                        }
                                    }
                                }
                            }
                        } else if let Some(parameter_str) = parameters[0].as_str() {
                            if !parameter_str.is_empty() {
                                let parsed: Option<String> =
                                    parse_parameter(Code::System, parameter_str, game_type);

                                if let Some(parsed) = parsed {
                                    if processing_type == ProcessingType::Append
                                        && !other_translation_map.contains_key(&parsed)
                                    {
                                        other_translation_map.shift_insert(
                                            other_lines.len(),
                                            parsed.clone(),
                                            "".into(),
                                        );
                                    }

                                    other_lines.insert(parsed);
                                }
                            }
                        } else if let Some(parameter_str) = parameters[1].as_str() {
                            if !parameter_str.is_empty() {
                                let parsed: Option<String> =
                                    parse_parameter(Code::Unknown, parameter_str, game_type);

                                if let Some(parsed) = parsed {
                                    if processing_type == ProcessingType::Append
                                        && !other_translation_map.contains_key(&parsed)
                                    {
                                        other_translation_map.shift_insert(
                                            other_lines.len(),
                                            parsed.clone(),
                                            "".into(),
                                        );
                                    }

                                    other_lines.insert(parsed);
                                }
                            }
                        }
                    }
                }
            }
        }

        let (original_content, translation_content) = if processing_type == ProcessingType::Append {
            let collected: (Vec<String>, Vec<String>) = other_translation_map.into_iter().unzip();
            (collected.0.join("\n"), collected.1.join("\n"))
        } else {
            let length: usize = other_lines.len() - 1;
            (
                other_lines.into_iter().collect::<Vec<String>>().join("\n"),
                "\n".repeat(length),
            )
        };

        write(other_output_path, original_content).unwrap();
        write(other_trans_output_path, translation_content).unwrap();

        if logging {
            println!("{} {filename}", unsafe { LOG_MSG });
        }
    }
}

// ! In current implementation, function performs extremely inefficient inserting of owned string to both hashmap and a hashset
/// Reads System .json file of other_path and parses it into .txt file in output_path.
/// # Parameters
/// * `other_path` - path to directory than contains .json files
/// * `output_path` - path to output directory
/// * `logging` - whether to log or not
/// * `processing_type` - whether to read in default mode, force rewrite or append new text to existing files
pub fn read_system(
    system_file_path: &Path,
    output_path: &Path,
    logging: bool,
    mut processing_type: &ProcessingType,
) {
    let system_output_path: &Path = &output_path.join("system.txt");
    let system_trans_output_path: &Path = &output_path.join("system_trans.txt");

    if processing_type == ProcessingType::Default && system_trans_output_path.exists() {
        println!("system_trans.txt {}", unsafe { FILE_ALREADY_PARSED });
        return;
    }

    let system_obj: Object = from_str(&read_to_string(system_file_path).unwrap()).unwrap();

    let mut system_lines: IndexSet<String, BuildHasherDefault<Xxh3>> = IndexSet::default();
    let mut system_translation_map: IndexMap<String, String, BuildHasherDefault<Xxh3>> =
        IndexMap::default();

    if processing_type == ProcessingType::Append {
        if system_trans_output_path.exists() {
            for (original, translated) in
                read_to_string(system_output_path).unwrap().split('\n').zip(
                    read_to_string(system_trans_output_path)
                        .unwrap()
                        .split('\n'),
                )
            {
                system_translation_map.insert(original.to_string(), translated.to_string());
            }
        } else {
            println!("{}", unsafe { FILE_IS_NOT_PARSED });
            processing_type = &ProcessingType::Default;
        }
    }

    // Armor types names
    // Normally it's system strings, but might be needed for some purposes
    for string in system_obj["armorTypes"].as_array().unwrap() {
        let str: &str = string.as_str().unwrap();

        if !str.is_empty() {
            let string: String = str.to_string();

            if processing_type == ProcessingType::Append
                && !system_translation_map.contains_key(&string)
            {
                system_translation_map.shift_insert(system_lines.len(), string.clone(), "".into());
            }

            system_lines.insert(string);
        }
    }

    // Element types names
    // Normally it's system strings, but might be needed for some purposes
    for string in system_obj["elements"].as_array().unwrap() {
        let str: &str = string.as_str().unwrap();

        if !str.is_empty() {
            let string: String = str.to_string();

            if processing_type == ProcessingType::Append
                && !system_translation_map.contains_key(&string)
            {
                system_translation_map.shift_insert(system_lines.len(), string.clone(), "".into());
            }

            system_lines.insert(string);
        }
    }

    // Names of equipment slots
    for string in system_obj["equipTypes"].as_array().unwrap() {
        let str: &str = string.as_str().unwrap();

        if !str.is_empty() {
            let string: String = str.to_string();

            if processing_type == ProcessingType::Append
                && !system_translation_map.contains_key(&string)
            {
                system_translation_map.shift_insert(system_lines.len(), string.clone(), "".into());
            }

            system_lines.insert(string);
        }
    }

    // Names of battle options
    for string in system_obj["skillTypes"].as_array().unwrap() {
        let str: &str = string.as_str().unwrap();

        if !str.is_empty() {
            let string: String = str.to_string();

            if processing_type == ProcessingType::Append
                && !system_translation_map.contains_key(&string)
            {
                system_translation_map.shift_insert(system_lines.len(), string.clone(), "".into());
            }

            system_lines.insert(string);
        }
    }

    // Game terms vocabulary
    for (key, value) in system_obj["terms"].as_object().unwrap() {
        if key != "messages" {
            for string in value.as_array().unwrap() {
                if let Some(str) = string.as_str() {
                    if !str.is_empty() {
                        let string: String = str.to_string();

                        if processing_type == ProcessingType::Append
                            && !system_translation_map.contains_key(&string)
                        {
                            system_translation_map.shift_insert(
                                system_lines.len(),
                                string.clone(),
                                "".into(),
                            );
                        }

                        system_lines.insert(string);
                    }
                }
            }
        } else {
            if !value.is_object() {
                continue;
            }

            for (_, message_string) in value.as_object().unwrap().iter() {
                let str: &str = message_string.as_str().unwrap();

                if !str.is_empty() {
                    let string: String = str.to_string();

                    if processing_type == ProcessingType::Append
                        && !system_translation_map.contains_key(&string)
                    {
                        system_translation_map.shift_insert(
                            system_lines.len(),
                            string.clone(),
                            "".into(),
                        );
                    }

                    system_lines.insert(string);
                }
            }
        }
    }

    // Weapon types names
    // Normally it's system strings, but might be needed for some purposes
    for string in system_obj["weaponTypes"].as_array().unwrap() {
        let str: &str = string.as_str().unwrap();

        if !str.is_empty() {
            let string: String = str.to_string();

            if processing_type == ProcessingType::Append
                && !system_translation_map.contains_key(&string)
            {
                system_translation_map.shift_insert(system_lines.len(), string.clone(), "".into());
            }

            system_lines.insert(string);
        }
    }

    // Game title, parsed just for fun
    // Translators may add something like "ELFISH TRANSLATION v1.0.0" to the title
    {
        let game_title_string: String = system_obj["gameTitle"].as_str().unwrap().to_string();

        if processing_type == ProcessingType::Append
            && !system_translation_map.contains_key(&game_title_string)
        {
            system_translation_map.shift_insert(
                system_lines.len(),
                game_title_string.clone(),
                "".into(),
            );
        }

        system_lines.insert(game_title_string);
    }

    let (original_content, translated_content) = if processing_type == ProcessingType::Append {
        let collected: (Vec<String>, Vec<String>) = system_translation_map.into_iter().unzip();
        (collected.0.join("\n"), collected.1.join("\n"))
    } else {
        let length: usize = system_lines.len() - 1;
        (
            system_lines.into_iter().collect::<Vec<String>>().join("\n"),
            "\n".repeat(length),
        )
    };

    write(system_output_path, original_content).unwrap();
    write(system_trans_output_path, translated_content).unwrap();

    if logging {
        println!("{} System.json.", unsafe { LOG_MSG });
    }
}

// read_plugins is not implemented and will NEVER be, as plugins can differ from each other incredibly.
// Change plugins.js with your own hands.
