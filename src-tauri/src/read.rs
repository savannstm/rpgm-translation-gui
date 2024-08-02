#![allow(clippy::too_many_arguments)]
use crate::{romanize_string, Code, EngineType, GameType, ProcessingMode, Variable, STRING_IS_ONLY_SYMBOLS_RE};
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

trait Join {
    fn join(&self, delimiter: &str) -> String;
}

impl<T: ToString + AsRef<str>, S: std::hash::BuildHasher> Join for IndexSet<T, S> {
    fn join(&self, delimiter: &str) -> String {
        let mut joined: String = String::new();
        joined.push_str(self.get_index(0).unwrap().as_ref());

        for item in self.iter().skip(1) {
            joined.push_str(delimiter);
            joined.push_str(item.as_ref());
        }

        joined
    }
}

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn parse_parameter(code: Code, mut parameter: &str, game_type: &Option<GameType>) -> Option<String> {
    if STRING_IS_ONLY_SYMBOLS_RE.is_match(parameter) {
        return None;
    }

    if let Some(game_type) = game_type {
        match game_type {
            GameType::Termina => {
                if parameter
                    .chars()
                    .all(|char: char| char.is_ascii_lowercase() || char.is_ascii_punctuation())
                {
                    return None;
                }

                match code {
                    Code::System => {
                        if !parameter.starts_with("Gab")
                            && (!parameter.starts_with("choice_text") || parameter.ends_with("????"))
                        {
                            return None;
                        }
                    }
                    _ => {}
                }
            } // custom processing for other games
        }
    }

    Some(parameter.to_string())
}

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn parse_variable(
    mut variable_text: String,
    variable_name: &Variable,
    filename: &str,
    game_type: &Option<GameType>,
) -> Option<(String, bool)> {
    if STRING_IS_ONLY_SYMBOLS_RE.is_match(&variable_text) {
        return None;
    }

    let mut is_continuation_of_description: bool = false;

    if let Some(game_type) = game_type {
        match game_type {
            GameType::Termina => {
                if variable_text.contains("---") || variable_text.starts_with("///") {
                    return None;
                }

                match variable_name {
                    Variable::Name | Variable::Nickname => {
                        if filename.starts_with("Ac") {
                            if ![
                                "Levi",
                                "Marina",
                                "Daan",
                                "Abella",
                                "O'saa",
                                "Blood golem",
                                "Marcoh",
                                "Karin",
                                "Olivia",
                                "Ghoul",
                                "Villager",
                                "August",
                                "Caligura",
                                "Henryk",
                                "Pav",
                                "Tanaka",
                                "Samarie",
                            ]
                            .contains(&variable_text.as_str())
                            {
                                return None;
                            }
                        } else if filename.starts_with("Ar") {
                            if variable_text.starts_with("test_armor") {
                                return None;
                            }
                        } else if filename.starts_with("Cl") {
                            if [
                                "Girl",
                                "Kid demon",
                                "Captain",
                                "Marriage",
                                "Marriage2",
                                "Baby demon",
                                "Buckman",
                                "Nas'hrah",
                                "Skeleton",
                            ]
                            .contains(&variable_text.as_str())
                            {
                                return None;
                            }
                        } else if filename.starts_with("En") {
                            if ["Spank Tank", "giant", "test"].contains(&variable_text.as_str()) {
                                return None;
                            }
                        } else if filename.starts_with("It") {
                            if [
                                "Torch",
                                "Flashlight",
                                "Stick",
                                "Quill",
                                "Empty scroll",
                                "Soul stone_NOT_USE",
                                "Cube of depths",
                                "Worm juice",
                                "Silver shilling",
                                "Coded letter #1 - UNUSED",
                                "Black vial",
                                "Torturer's notes 1",
                                "Purple vial",
                                "Orange vial",
                                "Red vial",
                                "Green vial",
                                "Pinecone pig instructions",
                                "Grilled salmonsnake meat",
                                "Empty scroll",
                                "Water vial",
                                "Blood vial",
                                "Devil's Grass",
                                "Stone",
                                "Codex #1",
                                "The Tale of the Pocketcat I",
                                "The Tale of the Pocketcat II",
                                "New poems of love and torment",
                            ]
                            .contains(&variable_text.as_str())
                                || variable_text.starts_with("The Fellowship")
                                || variable_text.starts_with("Studies of")
                                || variable_text.starts_with("Blueish")
                                || variable_text.starts_with("Skeletal")
                                || variable_text.ends_with("soul")
                                || variable_text.ends_with("schematics")
                            {
                                return None;
                            }
                        } else if filename.starts_with("We") && variable_text == "makeshift2" {
                            return None;
                        }
                    }
                    Variable::Note => {
                        let mut variable_text_chars: std::str::Chars = variable_text.chars();

                        if let Some(first_char) = variable_text_chars.next() {
                            if let Some(second_char) = variable_text_chars.next() {
                                if ((first_char == '\n' && second_char != '\n')
                                    || (first_char.is_ascii_alphabetic() || first_char == '"'))
                                    && !['.', '!', '/', '?'].contains(&first_char)
                                {
                                    is_continuation_of_description = true;
                                }
                            }
                        }

                        if is_continuation_of_description {
                            if let Some((left, _)) = variable_text.trim_start().split_once('\n') {
                                if !left.ends_with('.') && !left.ends_with('%') {
                                    return None;
                                }

                                variable_text = r"\#".to_string() + left;
                            } else {
                                if !variable_text.ends_with('.') && !variable_text.ends_with('%') {
                                    return None;
                                }

                                variable_text = r"\#".to_string() + &variable_text
                            }
                        } else {
                            return None;
                        }

                        if filename.starts_with("Ac") {
                            return None;
                        }
                    }
                    _ => {}
                }
            } // custom processing for other games
        }
    }

    Some((variable_text, is_continuation_of_description))
}

// ! In current implementation, function performs extremely inefficient inserting of owned string to both hashmap and a hashset
/// Reads all Map .json files of maps_path and parses them into .txt files in output_path.
/// # Parameters
/// * `maps_path` - path to directory than contains .json game files
/// * `output_path` - path to output directory
/// * `romanize` - whether to romanize text
/// * `logging` - whether to log
/// * `file_parsed_msg` - message to log when file is parsed
/// * `file_already_parsed_msg` - message to log when file that's about to be parsed already exists (default processing mode)
/// * `file_is_not_parsed_msg` - message to log when file that's about to be parsed not exist (append processing mode)
/// * `game_type` - game type for custom parsing
/// * `processing_mode` - whether to read in default mode, force rewrite or append new text to existing files
pub fn read_map(
    maps_path: &Path,
    output_path: &Path,
    romanize: bool,
    logging: bool,
    file_parsed_msg: &str,
    file_already_parsed_msg: &str,
    file_is_not_parsed_msg: &str,
    game_type: &Option<GameType>,
    mut processing_mode: &ProcessingMode,
    engine_type: &EngineType,
) {
    let maps_output_path: &Path = &output_path.join("maps.txt");
    let maps_trans_output_path: &Path = &output_path.join("maps_trans.txt");
    let names_output_path: &Path = &output_path.join("names.txt");
    let names_trans_output_path: &Path = &output_path.join("names_trans.txt");

    if processing_mode == ProcessingMode::Default && maps_trans_output_path.exists() {
        println!("maps_trans.txt {file_already_parsed_msg}");
        return;
    }

    let maps_files: Vec<DirEntry> = read_dir(maps_path)
        .unwrap()
        .filter_map(|entry: Result<DirEntry, _>| match entry {
            Ok(entry) => {
                let filename_os_string: OsString = entry.file_name();
                let filename: &str = unsafe { from_utf8_unchecked(filename_os_string.as_encoded_bytes()) };

                let slice: char;
                unsafe {
                    slice = *filename.as_bytes().get_unchecked(4) as char;
                }

                if filename.starts_with("Map") && slice.is_ascii_digit() && filename.ends_with("json") {
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

    let mut maps_translation_map: IndexMap<String, String, BuildHasherDefault<Xxh3>> = IndexMap::default();
    let mut names_translation_map: IndexMap<String, String, BuildHasherDefault<Xxh3>> = IndexMap::default();

    if processing_mode == ProcessingMode::Append {
        if maps_trans_output_path.exists() {
            for (original, translated) in read_to_string(maps_output_path)
                .unwrap()
                .par_split('\n')
                .collect::<Vec<_>>()
                .into_iter()
                .zip(
                    read_to_string(maps_trans_output_path)
                        .unwrap()
                        .par_split('\n')
                        .collect::<Vec<_>>()
                        .into_iter(),
                )
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
            println!("{file_is_not_parsed_msg}");
            processing_mode = &ProcessingMode::Default;
        }
    }

    // 401 - dialogue lines
    // 102 - dialogue choices array
    // 356 - system lines (special texts)
    // 324 - i don't know what is it but it's some used in-game lines
    const ALLOWED_CODES: [u64; 4] = [401, 102, 356, 324];

    for (filename, obj) in maps_obj_vec.into_iter() {
        if let Some(display_name) = obj[if *engine_type == EngineType::New {
            "displayName"
        } else {
            "display_name"
        }]
        .as_str()
        {
            if !display_name.is_empty() {
                let mut display_name_string: String = display_name.to_string();

                if romanize {
                    display_name_string = romanize_string(display_name_string);
                }

                if processing_mode == ProcessingMode::Append
                    && !names_translation_map.contains_key(&display_name_string)
                {
                    names_translation_map.shift_insert(names_lines.len(), display_name_string.clone(), "".into());
                }

                names_lines.insert(display_name_string);
            }
        }

        let mut events_arr: Option<Vec<Value>> = None;
        if *engine_type != EngineType::New {
            events_arr = Some(
                obj["events"]
                    .as_object()
                    .unwrap()
                    .into_iter()
                    .map(|(_, value)| value.to_owned())
                    .collect(),
            );
        }

        let iterator: std::iter::Skip<std::slice::Iter<Value>> = match *engine_type {
            EngineType::New => obj["events"].as_array().unwrap().iter().skip(1),
            _ => events_arr.as_ref().unwrap().iter().skip(1),
        };

        for event in iterator {
            if !event["pages"].is_array() {
                continue;
            }

            for page in event["pages"].as_array().unwrap().iter() {
                let mut in_sequence: bool = false;
                let mut line: Vec<String> = Vec::with_capacity(4);

                for list in page["list"].as_array().unwrap() {
                    let code: u64 = list["code"].as_u64().unwrap();

                    if in_sequence && code != 401 {
                        if !line.is_empty() {
                            let mut joined: String = line.join("\n").trim().replace('\n', r"\#");

                            if romanize {
                                joined = romanize_string(joined);
                            }

                            let parsed: Option<String> = parse_parameter(Code::Dialogue, &joined, game_type);

                            if let Some(parsed) = parsed {
                                if processing_mode == ProcessingMode::Append
                                    && !maps_translation_map.contains_key(&joined)
                                {
                                    maps_translation_map.shift_insert(maps_lines.len(), parsed.clone(), "".into());
                                }

                                maps_lines.insert(parsed);
                            }

                            line.clear();
                        }

                        in_sequence = false;
                    }

                    if !ALLOWED_CODES.contains(&code) {
                        continue;
                    }

                    let parameters: &Array = list["parameters"].as_array().unwrap();

                    if code == 401 {
                        if let Some(parameter_str) = parameters[0].as_str() {
                            if !parameter_str.is_empty() {
                                in_sequence = true;
                                line.push(parameter_str.trim().to_string());
                                // Maybe this shouldn't be trimmed
                            }
                        }
                    } else if parameters[0].is_array() {
                        for i in 0..parameters[0].as_array().unwrap().len() {
                            if let Some(subparameter_str) = parameters[0][i].as_str() {
                                if !subparameter_str.is_empty() {
                                    let parsed: Option<String> =
                                        parse_parameter(Code::Choice, subparameter_str, game_type);

                                    if let Some(mut parsed) = parsed {
                                        if romanize {
                                            parsed = romanize_string(parsed);
                                        }

                                        if processing_mode == ProcessingMode::Append
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
                            let parsed: Option<String> = parse_parameter(Code::System, parameter_str, game_type);

                            if let Some(mut parsed) = parsed {
                                if romanize {
                                    parsed = romanize_string(parsed);
                                }

                                if processing_mode == ProcessingMode::Append
                                    && !maps_translation_map.contains_key(&parsed)
                                {
                                    maps_translation_map.shift_insert(maps_lines.len(), parsed.clone(), "".into());
                                }

                                maps_lines.insert(parsed);
                            }
                        }
                    } else if let Some(parameter_str) = parameters[1].as_str() {
                        if !parameter_str.is_empty() {
                            let parsed: Option<String> = parse_parameter(Code::Unknown, parameter_str, game_type);

                            if let Some(mut parsed) = parsed {
                                if romanize {
                                    parsed = romanize_string(parsed);
                                }

                                if processing_mode == ProcessingMode::Append
                                    && !maps_translation_map.contains_key(&parsed)
                                {
                                    maps_translation_map.shift_insert(maps_lines.len(), parsed.clone(), "".into());
                                }

                                maps_lines.insert(parsed);
                            }
                        }
                    }
                }
            }
        }

        if logging {
            println!("{file_parsed_msg} {filename}.");
        }
    }

    let (maps_original_content, maps_translated_content, names_original_content, names_translated_content) =
        if processing_mode == ProcessingMode::Append {
            let maps_collected: (Vec<String>, Vec<String>) = maps_translation_map.into_iter().unzip();
            let names_collected: (Vec<String>, Vec<String>) = names_translation_map.into_iter().unzip();
            (
                maps_collected.0.join("\n"),
                maps_collected.1.join("\n"),
                names_collected.0.join("\n"),
                names_collected.1.join("\n"),
            )
        } else {
            (
                maps_lines.join("\n"),
                "\n".repeat(maps_lines.len() - 1),
                names_lines.join("\n"),
                "\n".repeat(names_lines.len() - 1),
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
/// * `other_path` - path to directory than contains .json game files
/// * `output_path` - path to output directory
/// * `romanize` - whether to romanize text
/// * `logging` - whether to log
/// * `file_parsed_msg` - message to log when file is parsed
/// * `file_already_parsed_msg` - message to log when file that's about to be parsed already exists (default processing mode)
/// * `file_is_not_parsed_msg` - message to log when file that's about to be parsed not exist (append processing mode)
/// * `game_type` - game type for custom parsing
/// * `processing_mode` - whether to read in default mode, force rewrite or append new text to existing files
pub fn read_other(
    other_path: &Path,
    output_path: &Path,
    romanize: bool,
    logging: bool,
    file_parsed_msg: &str,
    file_already_parsed_msg: &str,
    file_is_not_parsed_msg: &str,
    game_type: &Option<GameType>,
    processing_mode: &ProcessingMode,
) {
    let other_files: Vec<DirEntry> = read_dir(other_path)
        .unwrap()
        .filter_map(|entry: Result<DirEntry, _>| match entry {
            Ok(entry) => {
                let filename_os_string: OsString = entry.file_name();
                let filename: &str = unsafe { from_utf8_unchecked(filename_os_string.as_encoded_bytes()) };
                let (real_name, extension) = filename.split_once('.').unwrap();

                if !real_name.starts_with("Map")
                    && !matches!(real_name, "Tilesets" | "Animations" | "States" | "System" | "Scripts")
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

    let mut inner_processing_type: &ProcessingMode = processing_mode;

    // 401 - dialogue lines
    // 405 - credits lines
    // 102 - dialogue choices array
    // 356 - system lines (special texts)
    // 324 - i don't know what is it but it's some used in-game lines
    const ALLOWED_CODES: [u64; 5] = [401, 405, 356, 102, 324];

    for (filename, obj_arr) in other_obj_arr_map.into_iter() {
        let other_processed_filename: String = filename[0..filename.rfind('.').unwrap()].to_lowercase();

        let other_output_path: &Path = &output_path.join(other_processed_filename.clone() + ".txt");
        let other_trans_output_path: &Path = &output_path.join(other_processed_filename + "_trans.txt");

        if processing_mode == ProcessingMode::Default && other_trans_output_path.exists() {
            println!("{} {file_already_parsed_msg}", unsafe {
                from_utf8_unchecked(other_trans_output_path.file_name().unwrap().as_encoded_bytes())
            });
            continue;
        }

        let mut other_lines: IndexSet<String, BuildHasherDefault<Xxh3>> = IndexSet::default();
        let mut other_translation_map: IndexMap<String, String, BuildHasherDefault<Xxh3>> = IndexMap::default();

        if processing_mode == ProcessingMode::Append {
            if other_trans_output_path.exists() {
                for (original, translated) in read_to_string(other_output_path)
                    .unwrap()
                    .par_split('\n')
                    .collect::<Vec<_>>()
                    .into_iter()
                    .zip(
                        read_to_string(other_trans_output_path)
                            .unwrap()
                            .par_split('\n')
                            .collect::<Vec<_>>()
                            .into_iter(),
                    )
                {
                    other_translation_map.insert(original.to_string(), translated.to_string());
                }
            } else {
                println!("{file_is_not_parsed_msg}");
                inner_processing_type = &ProcessingMode::Default;
            }
        }

        // Other files except CommonEvents.json and Troops.json have the structure that consists
        // of name, nickname, description and note
        if !filename.starts_with("Co") && !filename.starts_with("Tr") {
            if filename.starts_with("It") {
                for string in [
                    "<Menu Category: Items>",
                    "<Menu Category: Food>",
                    "<Menu Category: Healing>",
                    "<Menu Category: Body bag>",
                ] {
                    other_lines.insert(string.to_string());
                }
            }

            'obj: for obj in obj_arr {
                let mut prev_variable_name: Option<Variable> = None;

                for (variable_text, variable_name) in [
                    (obj["name"].as_str(), Variable::Name),
                    (obj["nickname"].as_str(), Variable::Nickname),
                    (obj["description"].as_str(), Variable::Description),
                    (obj["note"].as_str(), Variable::Note),
                ] {
                    if let Some(mut variable_str) = variable_text {
                        variable_str = variable_str.trim();

                        if !variable_str.is_empty() {
                            let parsed: Option<(String, bool)> =
                                parse_variable(variable_str.to_string(), &variable_name, &filename, game_type);

                            if let Some((mut parsed, is_continuation_of_description)) = parsed {
                                if is_continuation_of_description {
                                    if prev_variable_name != Some(Variable::Description) {
                                        continue;
                                    }

                                    if let Some(last) = other_lines.pop() {
                                        other_lines.insert(last + &parsed);
                                    }

                                    if inner_processing_type == ProcessingMode::Append {
                                        if let Some((key, value)) = other_translation_map.pop() {
                                            other_translation_map.insert(key, value + &parsed);
                                        }
                                    }

                                    continue;
                                }

                                prev_variable_name = Some(variable_name);

                                if romanize {
                                    parsed = romanize_string(parsed);
                                }

                                let replaced: String = parsed
                                    .split('\n')
                                    .map(|line: &str| line.trim())
                                    .collect::<Vec<_>>()
                                    .join(r"\#");

                                if inner_processing_type == ProcessingMode::Append
                                    && !other_translation_map.contains_key(&replaced)
                                {
                                    other_translation_map.shift_insert(other_lines.len(), replaced.clone(), "".into());
                                }

                                other_lines.insert(replaced);
                            } else if variable_name == Variable::Name {
                                continue 'obj;
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
                let pages_length: usize = if filename.starts_with("Tr") {
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

                    for item in list.as_array().unwrap() {
                        let code: u64 = item["code"].as_u64().unwrap();

                        if in_sequence && ![401, 405].contains(&code) {
                            if !line.is_empty() {
                                let mut joined: String = line.join("\n").trim().replace('\n', r"\#");

                                if romanize {
                                    joined = romanize_string(joined);
                                }

                                let parsed: Option<String> = parse_parameter(Code::Dialogue, &joined, game_type);

                                if let Some(parsed) = parsed {
                                    if processing_mode == ProcessingMode::Append
                                        && !other_translation_map.contains_key(&joined)
                                    {
                                        other_translation_map.shift_insert(
                                            other_lines.len(),
                                            parsed.clone(),
                                            "".into(),
                                        );
                                    }

                                    other_lines.insert(parsed);
                                }

                                line.clear();
                            }

                            in_sequence = false;
                        }

                        if !ALLOWED_CODES.contains(&code) {
                            continue;
                        }

                        let parameters: &Array = item["parameters"].as_array().unwrap();

                        if [401, 405].contains(&code) {
                            if let Some(parameter_str) = parameters[0].as_str() {
                                if !parameter_str.is_empty() {
                                    in_sequence = true;
                                    // Maybe this shouldn't be trimmed
                                    line.push(parameter_str.trim().to_string());
                                }
                            }
                        } else if parameters[0].is_array() {
                            for i in 0..parameters[0].as_array().unwrap().len() {
                                if let Some(subparameter_str) = parameters[0][i].as_str() {
                                    if !subparameter_str.is_empty() {
                                        let parsed: Option<String> =
                                            parse_parameter(Code::Choice, subparameter_str, game_type);

                                        if let Some(mut parsed) = parsed {
                                            if romanize {
                                                parsed = romanize_string(parsed);
                                            }

                                            if processing_mode == ProcessingMode::Append
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
                                let parsed: Option<String> = parse_parameter(Code::System, parameter_str, game_type);

                                if let Some(mut parsed) = parsed {
                                    if romanize {
                                        parsed = romanize_string(parsed);
                                    }

                                    if processing_mode == ProcessingMode::Append
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
                                let parsed: Option<String> = parse_parameter(Code::Unknown, parameter_str, game_type);

                                if let Some(mut parsed) = parsed {
                                    if romanize {
                                        parsed = romanize_string(parsed);
                                    }

                                    if processing_mode == ProcessingMode::Append
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

        let (original_content, translation_content) = if processing_mode == ProcessingMode::Append {
            let collected: (Vec<String>, Vec<String>) = other_translation_map.into_iter().unzip();
            (collected.0.join("\n"), collected.1.join("\n"))
        } else {
            let length: usize = other_lines.len() - 1;
            (other_lines.join("\n"), "\n".repeat(length))
        };

        write(other_output_path, original_content).unwrap();
        write(other_trans_output_path, translation_content).unwrap();

        if logging {
            println!("{file_parsed_msg} {filename}");
        }
    }
}

// ! In current implementation, function performs extremely inefficient inserting of owned string to both hashmap and a hashset
/// Reads System .json file of system_file_path and parses it into .txt file of output_path.
/// # Parameters
/// * `system_file_path` - path to directory than contains .json files
/// * `output_path` - path to output directory
/// * `romanize` - whether to romanize text
/// * `logging` - whether to log
/// * `file_parsed_msg` - message to log when file is parsed
/// * `file_already_parsed_msg` - message to log when file that's about to be parsed already exists (default processing mode)
/// * `file_is_not_parsed_msg` - message to log when file that's about to be parsed not exist (append processing mode)
/// * `processing_mode` - whether to read in default mode, force rewrite or append new text to existing files
pub fn read_system(
    system_file_path: &Path,
    output_path: &Path,
    romanize: bool,
    logging: bool,
    file_parsed_msg: &str,
    file_already_parsed_msg: &str,
    file_is_not_parsed_msg: &str,
    mut processing_mode: &ProcessingMode,
    engine_type: &EngineType,
) {
    let system_output_path: &Path = &output_path.join("system.txt");
    let system_trans_output_path: &Path = &output_path.join("system_trans.txt");

    if processing_mode == ProcessingMode::Default && system_trans_output_path.exists() {
        println!("system_trans.txt {file_already_parsed_msg}");
        return;
    }

    let system_obj: Object = from_str(&read_to_string(system_file_path).unwrap()).unwrap();

    let mut system_lines: IndexSet<String, BuildHasherDefault<Xxh3>> = IndexSet::default();
    let mut system_translation_map: IndexMap<String, String, BuildHasherDefault<Xxh3>> = IndexMap::default();

    if processing_mode == ProcessingMode::Append {
        if system_trans_output_path.exists() {
            for (original, translated) in read_to_string(system_output_path)
                .unwrap()
                .par_split('\n')
                .collect::<Vec<_>>()
                .into_iter()
                .zip(
                    read_to_string(system_trans_output_path)
                        .unwrap()
                        .par_split('\n')
                        .collect::<Vec<_>>()
                        .into_iter(),
                )
            {
                system_translation_map.insert(original.to_string(), translated.to_string());
            }
        } else {
            println!("{file_is_not_parsed_msg}");
            processing_mode = &ProcessingMode::Default;
        }
    }

    // Armor types names
    // Normally it's system strings, but might be needed for some purposes
    for string in system_obj[if *engine_type == EngineType::New {
        "armorTypes"
    } else {
        "armor_types"
    }]
    .as_array()
    .unwrap()
    {
        let str: &str = string.as_str().unwrap().trim();

        if !str.is_empty() {
            let mut string: String = str.to_string();

            if romanize {
                string = romanize_string(string)
            }

            if processing_mode == ProcessingMode::Append && !system_translation_map.contains_key(&string) {
                system_translation_map.shift_insert(system_lines.len(), string.clone(), "".into());
            }

            system_lines.insert(string);
        }
    }

    // Element types names
    // Normally it's system strings, but might be needed for some purposes
    for string in system_obj["elements"].as_array().unwrap() {
        let str: &str = string.as_str().unwrap().trim();

        if !str.is_empty() {
            let mut string: String = str.to_string();

            if romanize {
                string = romanize_string(string)
            }

            if processing_mode == ProcessingMode::Append && !system_translation_map.contains_key(&string) {
                system_translation_map.shift_insert(system_lines.len(), string.clone(), "".into());
            }

            system_lines.insert(string);
        }
    }

    // Names of equipment slots
    if *engine_type == EngineType::New {
        for string in system_obj["equipTypes"].as_array().unwrap() {
            let str: &str = string.as_str().unwrap().trim();

            if !str.is_empty() {
                let mut string: String = str.to_string();

                if romanize {
                    string = romanize_string(string)
                }

                if processing_mode == ProcessingMode::Append && !system_translation_map.contains_key(&string) {
                    system_translation_map.shift_insert(system_lines.len(), string.clone(), "".into());
                }

                system_lines.insert(string);
            }
        }
    }

    // Names of battle options
    for string in system_obj[if *engine_type == EngineType::New {
        "skillTypes"
    } else {
        "skill_types"
    }]
    .as_array()
    .unwrap()
    {
        let str: &str = string.as_str().unwrap().trim();

        if !str.is_empty() {
            let mut string: String = str.to_string();

            if romanize {
                string = romanize_string(string)
            }

            if processing_mode == ProcessingMode::Append && !system_translation_map.contains_key(&string) {
                system_translation_map.shift_insert(system_lines.len(), string.clone(), "".into());
            }

            system_lines.insert(string);
        }
    }

    // Game terms vocabulary
    for (key, value) in system_obj[if *engine_type != EngineType::XP {
        "terms"
    } else {
        "words"
    }]
    .as_object()
    .unwrap()
    {
        if key.starts_with("__") {
            continue;
        }

        if key != "messages" {
            for string in value.as_array().unwrap() {
                if let Some(mut str) = string.as_str() {
                    str = str.trim();

                    if !str.is_empty() {
                        let mut string: String = str.to_string();

                        if romanize {
                            string = romanize_string(string)
                        }

                        if processing_mode == ProcessingMode::Append && !system_translation_map.contains_key(&string) {
                            system_translation_map.shift_insert(system_lines.len(), string.clone(), "".into());
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
                let str: &str = message_string.as_str().unwrap().trim();

                if !str.is_empty() {
                    let mut string: String = str.to_string();

                    if romanize {
                        string = romanize_string(string)
                    }

                    if processing_mode == ProcessingMode::Append && !system_translation_map.contains_key(&string) {
                        system_translation_map.shift_insert(system_lines.len(), string.clone(), "".into());
                    }

                    system_lines.insert(string);
                }
            }
        }
    }

    // Weapon types names
    // Normally it's system strings, but might be needed for some purposes
    for string in system_obj[if *engine_type == EngineType::New {
        "weaponTypes"
    } else {
        "weapon_types"
    }]
    .as_array()
    .unwrap()
    {
        let str: &str = string.as_str().unwrap().trim();

        if !str.is_empty() {
            let mut string: String = str.to_string();

            if romanize {
                string = romanize_string(string)
            }

            if processing_mode == ProcessingMode::Append && !system_translation_map.contains_key(&string) {
                system_translation_map.shift_insert(system_lines.len(), string.clone(), "".into());
            }

            system_lines.insert(string);
        }
    }

    // Game title, parsed just for fun
    // Translators may add something like "ELFISH TRANSLATION v1.0.0" to the title
    {
        let mut game_title_string: String = system_obj[if *engine_type == EngineType::New {
            "gameTitle"
        } else {
            "game_title"
        }]
        .as_str()
        .unwrap()
        .trim()
        .to_string();

        if romanize {
            game_title_string = romanize_string(game_title_string)
        }

        if processing_mode == ProcessingMode::Append && !system_translation_map.contains_key(&game_title_string) {
            system_translation_map.shift_insert(system_lines.len(), game_title_string.clone(), "".into());
        }

        system_lines.insert(game_title_string);
    }

    let (original_content, translated_content) = if processing_mode == ProcessingMode::Append {
        let collected: (Vec<String>, Vec<String>) = system_translation_map.into_iter().unzip();
        (collected.0.join("\n"), collected.1.join("\n"))
    } else {
        let length: usize = system_lines.len() - 1;
        (system_lines.join("\n"), "\n".repeat(length))
    };

    write(system_output_path, original_content).unwrap();
    write(system_trans_output_path, translated_content).unwrap();

    if logging {
        println!("{file_parsed_msg} System.json.");
    }
}
