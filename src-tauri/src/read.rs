#![allow(clippy::too_many_arguments)]
use crate::{
    romanize_string, Code, EngineType, GameType, ProcessingMode, Variable, ENDS_WITH_IF_RE,
    INVALID_MULTILINE_VARIABLE_RE, INVALID_VARIABLE_RE, LISA_PREFIX_RE, STRING_IS_ONLY_SYMBOLS_RE,
};
use core::str;
use indexmap::{IndexMap, IndexSet};
use marshal_rs::load::load;
use rayon::prelude::*;
use sonic_rs::{from_str, from_value, Array, JsonContainerTrait, JsonValueTrait, Value};
use std::{
    ffi::OsString,
    fs::{read, read_dir, read_to_string, write, DirEntry},
    hash::{BuildHasher, BuildHasherDefault},
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

        if self.is_empty() {
            return joined;
        }

        joined.push_str(self.get_index(0).unwrap().as_ref());

        for item in self.iter().skip(1) {
            joined.push_str(delimiter);
            joined.push_str(item.as_ref());
        }

        joined
    }
}

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn parse_parameter(
    code: Code,
    mut parameter: &str,
    game_type: &Option<GameType>,
    engine_type: &EngineType,
) -> Option<String> {
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
            }
            GameType::LisaRPG => match code {
                Code::Dialogue => {
                    if let Some(re_match) = LISA_PREFIX_RE.find(parameter) {
                        parameter = &parameter[re_match.end()..]
                    }

                    if STRING_IS_ONLY_SYMBOLS_RE.is_match(parameter) {
                        return None;
                    }
                }
                _ => {}
            }, // custom processing for other games
        }
    }

    if engine_type != EngineType::New {
        if let Some(re_match) = ENDS_WITH_IF_RE.find(parameter) {
            parameter = &parameter[re_match.start()..]
        }
    }

    Some(parameter.to_string())
}

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn parse_variable(
    mut variable_text: String,
    variable_type: &Variable,
    filename: &str,
    game_type: &Option<GameType>,
    engine_type: &EngineType,
) -> Option<(String, bool)> {
    if STRING_IS_ONLY_SYMBOLS_RE.is_match(&variable_text) {
        return None;
    }

    let mut is_continuation_of_description: bool = false;

    if engine_type != EngineType::New {
        if variable_text
            .split('\n')
            .all(|line: &str| line.is_empty() || INVALID_MULTILINE_VARIABLE_RE.is_match(line))
            || INVALID_VARIABLE_RE.is_match(&variable_text)
        {
            return None;
        };

        variable_text = variable_text.replace("\r\n", "\n");
    }

    #[allow(clippy::collapsible_match)]
    if let Some(game_type) = game_type {
        match game_type {
            GameType::Termina => {
                if variable_text.contains("---") || variable_text.starts_with("///") {
                    return None;
                }

                match variable_type {
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
                    Variable::Message1 | Variable::Message2 | Variable::Message3 | Variable::Message4 => {
                        return None;
                    }
                    Variable::Note => {
                        if filename.starts_with("Ac") {
                            return None;
                        }

                        if !filename.starts_with("Cl") {
                            let mut variable_text_chars: std::str::Chars = variable_text.chars();

                            if !variable_text.starts_with("flesh puppetry") {
                                if let Some(first_char) = variable_text_chars.next() {
                                    if let Some(second_char) = variable_text_chars.next() {
                                        if ((first_char == '\n' && second_char != '\n')
                                            || (first_char.is_ascii_alphabetic()
                                                || first_char == '"'
                                                || variable_text.starts_with("4 sticks")))
                                            && !['.', '!', '/', '?'].contains(&first_char)
                                        {
                                            is_continuation_of_description = true;
                                        }
                                    }
                                }
                            }

                            if is_continuation_of_description {
                                if let Some((mut left, _)) = variable_text.trim_start().split_once('\n') {
                                    left = left.trim();

                                    if !left.ends_with(['.', '%', '!', '"']) {
                                        return None;
                                    }

                                    variable_text = r"\#".to_string() + left;
                                } else {
                                    if !variable_text.ends_with(['.', '%', '!', '"']) {
                                        return None;
                                    }

                                    variable_text = r"\#".to_string() + &variable_text
                                }
                            } else {
                                return None;
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {} // custom processing for other games
        }
    }

    Some((variable_text, is_continuation_of_description))
}

fn parse_list<T: BuildHasher>(
    list: &Array,
    allowed_codes: &[u64],
    romanize: bool,
    game_type: &Option<GameType>,
    engine_type: &EngineType,
    processing_mode: &ProcessingMode,
    set: &mut IndexSet<String, T>,
    map: &mut IndexMap<String, String, T>,
) {
    let mut in_sequence: bool = false;
    let mut line: Vec<String> = Vec::with_capacity(4);

    for list in list {
        let code: u64 = list[if engine_type == EngineType::New {
            "code"
        } else {
            "__symbol__code"
        }]
        .as_u64()
        .unwrap();

        if in_sequence && ![401, 405].contains(&code) {
            if !line.is_empty() {
                let mut joined: String = line.join("\n").trim().replace('\n', r"\#");

                if romanize {
                    joined = romanize_string(joined);
                }

                let parsed: Option<String> = parse_parameter(Code::Dialogue, &joined, game_type, engine_type);

                if let Some(parsed) = parsed {
                    if processing_mode == ProcessingMode::Append && !map.contains_key(&joined) {
                        map.shift_insert(set.len(), parsed.clone(), String::new());
                    }

                    set.insert(parsed);
                }

                line.clear();
            }

            in_sequence = false;
        }

        if allowed_codes.binary_search(&code).is_err() {
            continue;
        }

        let parameters: &Array = list[if engine_type == EngineType::New {
            "parameters"
        } else {
            "__symbol__parameters"
        }]
        .as_array()
        .unwrap();

        match code {
            401 | 405 => {
                let parameter_string: String = parameters[0]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| {
                        if let Some(parameter_obj) = parameters[0].as_object() {
                            match parameter_obj.get(&"__type") {
                                Some(object_type) => {
                                    if object_type.as_str().unwrap() != "bytes" {
                                        String::new()
                                    } else {
                                        let bytes_array: Vec<u8> = from_value(&parameter_obj["data"]).unwrap();
                                        unsafe { String::from_utf8_unchecked(bytes_array) }
                                    }
                                }
                                None => String::new(),
                            }
                        } else {
                            String::new()
                        }
                    })
                    .trim()
                    .to_string();

                if !parameter_string.is_empty() {
                    in_sequence = true;
                    line.push(parameter_string);
                }
            }
            102 => {
                for i in 0..parameters[0].as_array().unwrap().len() {
                    let subparameter_string: String = parameters[0][i]
                        .as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| {
                            if let Some(parameter_obj) = parameters[0].as_object() {
                                match parameter_obj.get(&"__type") {
                                    Some(object_type) => {
                                        if object_type.as_str().unwrap() != "bytes" {
                                            String::new()
                                        } else {
                                            let bytes_array: Vec<u8> = from_value(&parameter_obj["data"]).unwrap();
                                            unsafe { String::from_utf8_unchecked(bytes_array) }
                                        }
                                    }
                                    None => String::new(),
                                }
                            } else {
                                String::new()
                            }
                        })
                        .trim()
                        .to_string();

                    if !subparameter_string.is_empty() {
                        let parsed: Option<String> =
                            parse_parameter(Code::Choice, &subparameter_string, game_type, engine_type);

                        if let Some(mut parsed) = parsed {
                            if romanize {
                                parsed = romanize_string(parsed);
                            }

                            if processing_mode == ProcessingMode::Append && !map.contains_key(&parsed) {
                                map.shift_insert(set.len(), parsed.clone(), String::new());
                            }

                            set.insert(parsed);
                        }
                    }
                }
            }
            356 => {
                let parameter_string: String = parameters[0]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| {
                        if let Some(parameter_obj) = parameters[0].as_object() {
                            match parameter_obj.get(&"__type") {
                                Some(object_type) => {
                                    if object_type.as_str().unwrap() != "bytes" {
                                        String::new()
                                    } else {
                                        let bytes_array: Vec<u8> = from_value(&parameter_obj["data"]).unwrap();
                                        unsafe { String::from_utf8_unchecked(bytes_array) }
                                    }
                                }
                                None => String::new(),
                            }
                        } else {
                            String::new()
                        }
                    })
                    .trim()
                    .to_string();

                if !parameter_string.is_empty() {
                    let parsed: Option<String> =
                        parse_parameter(Code::System, &parameter_string, game_type, engine_type);

                    if let Some(mut parsed) = parsed {
                        if romanize {
                            parsed = romanize_string(parsed);
                        }

                        if processing_mode == ProcessingMode::Append && !map.contains_key(&parsed) {
                            map.shift_insert(set.len(), parsed.clone(), String::new());
                        }

                        set.insert(parsed);
                    }
                }
            }
            324 | 320 => {
                let parameter_string: String = parameters[1]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| {
                        if let Some(parameter_obj) = parameters[1].as_object() {
                            match parameter_obj.get(&"__type") {
                                Some(object_type) => {
                                    if object_type.as_str().unwrap() != "bytes" {
                                        String::new()
                                    } else {
                                        let bytes_array: Vec<u8> = from_value(&parameter_obj["data"]).unwrap();
                                        unsafe { String::from_utf8_unchecked(bytes_array) }
                                    }
                                }
                                None => String::new(),
                            }
                        } else {
                            String::new()
                        }
                    })
                    .trim()
                    .to_string();

                if !parameter_string.is_empty() {
                    let parsed: Option<String> =
                        parse_parameter(Code::Unknown, &parameter_string, game_type, engine_type);

                    if let Some(mut parsed) = parsed {
                        if romanize {
                            parsed = romanize_string(parsed);
                        }

                        if processing_mode == ProcessingMode::Append && !map.contains_key(&parsed) {
                            map.shift_insert(set.len(), parsed.clone(), String::new());
                        }

                        set.insert(parsed);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
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

    let maps_obj_vec = read_dir(maps_path)
        .unwrap()
        .filter_map(|entry: Result<DirEntry, std::io::Error>| match entry {
            Ok(entry) => {
                let filename: OsString = entry.file_name();
                let filename_str: &str = unsafe { from_utf8_unchecked(filename.as_encoded_bytes()) };

                if filename_str.starts_with("Map")
                    && unsafe { (*filename_str.as_bytes().get_unchecked(3) as char).is_ascii_digit() }
                    && (filename_str.ends_with("json")
                        || filename_str.ends_with("rvdata2")
                        || filename_str.ends_with("rvdata")
                        || filename_str.ends_with("rxdata"))
                {
                    let json: Value = if engine_type == EngineType::New {
                        from_str(&read_to_string(entry.path()).unwrap()).unwrap()
                    } else {
                        load(&read(entry.path()).unwrap(), None, Some(""))
                    };

                    Some((filename_str.to_string(), json))
                } else {
                    None
                }
            }
            Err(_) => None,
        });

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
    const ALLOWED_CODES: [u64; 5] = [102, 320, 324, 356, 401];

    for (filename, obj) in maps_obj_vec {
        if let Some(display_name) = obj[if engine_type == EngineType::New {
            "displayName"
        } else {
            "__symbol__display_name"
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
                    names_translation_map.shift_insert(names_lines.len(), display_name_string.clone(), String::new());
                }

                names_lines.insert(display_name_string);
            }
        }

        let events_arr: Vec<&Value> = if engine_type == EngineType::New {
            obj["events"].as_array().unwrap().iter().skip(1).collect()
        } else {
            obj["__symbol__events"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(_, value)| value)
                .collect()
        };

        for event in events_arr.iter() {
            if !event[if engine_type == EngineType::New {
                "pages"
            } else {
                "__symbol__pages"
            }]
            .is_array()
            {
                continue;
            }

            for page in event[if engine_type == EngineType::New {
                "pages"
            } else {
                "__symbol__pages"
            }]
            .as_array()
            .unwrap()
            .iter()
            {
                parse_list(
                    page[if engine_type == EngineType::New {
                        "list"
                    } else {
                        "__symbol__list"
                    }]
                    .as_array()
                    .unwrap(),
                    &ALLOWED_CODES,
                    romanize,
                    game_type,
                    engine_type,
                    processing_mode,
                    &mut maps_lines,
                    &mut maps_translation_map,
                );
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
                "\n".repeat(maps_lines.len().saturating_sub(1)),
                names_lines.join("\n"),
                "\n".repeat(names_lines.len().saturating_sub(1)),
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
    engine_type: &EngineType,
) {
    let other_obj_arr_iter = read_dir(other_path)
        .unwrap()
        .filter_map(|entry: Result<DirEntry, std::io::Error>| match entry {
            Ok(entry) => {
                let filename_os_string: OsString = entry.file_name();
                let filename: &str = unsafe { from_utf8_unchecked(filename_os_string.as_encoded_bytes()) };
                let (real_name, extension) = filename.split_once('.').unwrap();

                if !real_name.starts_with("Map")
                    && !matches!(real_name, "Tilesets" | "Animations" | "System")
                    && ["json", "rvdata2", "rvdata", "rxdata"].contains(&extension)
                {
                    if game_type
                        .as_ref()
                        .is_some_and(|game_type| game_type == GameType::Termina)
                        && real_name == "States"
                    {
                        return None;
                    }

                    let json: Value = if engine_type == EngineType::New {
                        from_str(&read_to_string(entry.path()).unwrap()).unwrap()
                    } else {
                        load(&read(entry.path()).unwrap(), None, Some(""))
                    };

                    Some((filename.to_string(), json))
                } else {
                    None
                }
            }
            Err(_) => None,
        });

    let mut inner_processing_type: &ProcessingMode = processing_mode;

    // 401 - dialogue lines
    // 405 - credits lines
    // 102 - dialogue choices array
    // 356 - system lines (special texts)
    // 324 - i don't know what is it but it's some used in-game lines
    const ALLOWED_CODES: [u64; 6] = [102, 320, 324, 356, 401, 405];

    for (filename, obj_arr) in other_obj_arr_iter {
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
            if game_type
                .as_ref()
                .is_some_and(|game_type| game_type == GameType::Termina)
                && filename.starts_with("It")
            {
                for string in [
                    "<Menu Category: Items>",
                    "<Menu Category: Food>",
                    "<Menu Category: Healing>",
                    "<Menu Category: Body bag>",
                ] {
                    other_lines.insert(string.to_string());
                }
            }

            'obj: for obj in obj_arr.as_array().unwrap() {
                let mut prev_variable_type: Option<Variable> = None;

                for (variable_text, variable_type) in [
                    (
                        obj[if engine_type == EngineType::New {
                            "name"
                        } else {
                            "__symbol__name"
                        }]
                        .as_str(),
                        Variable::Name,
                    ),
                    (
                        obj[if engine_type == EngineType::New {
                            "nickname"
                        } else {
                            "__symbol__nickname"
                        }]
                        .as_str(),
                        Variable::Nickname,
                    ),
                    (
                        obj[if engine_type == EngineType::New {
                            "description"
                        } else {
                            "__symbol__description"
                        }]
                        .as_str(),
                        Variable::Description,
                    ),
                    (
                        obj[if engine_type == EngineType::New {
                            "message1"
                        } else {
                            "__symbol__message1"
                        }]
                        .as_str(),
                        Variable::Message1,
                    ),
                    (
                        obj[if engine_type == EngineType::New {
                            "message2"
                        } else {
                            "__symbol__message2"
                        }]
                        .as_str(),
                        Variable::Message2,
                    ),
                    (
                        obj[if engine_type == EngineType::New {
                            "message3"
                        } else {
                            "__symbol__message3"
                        }]
                        .as_str(),
                        Variable::Message3,
                    ),
                    (
                        obj[if engine_type == EngineType::New {
                            "message4"
                        } else {
                            "__symbol__message4"
                        }]
                        .as_str(),
                        Variable::Message4,
                    ),
                    (
                        obj[if engine_type == EngineType::New {
                            "note"
                        } else {
                            "__symbol__note"
                        }]
                        .as_str(),
                        Variable::Note,
                    ),
                ] {
                    if let Some(mut variable_str) = variable_text {
                        variable_str = variable_str.trim();

                        if !variable_str.is_empty() {
                            let parsed: Option<(String, bool)> = parse_variable(
                                variable_str.to_string(),
                                &variable_type,
                                &filename,
                                game_type,
                                engine_type,
                            );

                            if let Some((mut parsed, is_continuation_of_description)) = parsed {
                                if is_continuation_of_description {
                                    if prev_variable_type != Some(Variable::Description) {
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

                                prev_variable_type = Some(variable_type);

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
                                    other_translation_map.shift_insert(
                                        other_lines.len(),
                                        replaced.clone(),
                                        String::new(),
                                    );
                                }

                                other_lines.insert(replaced);
                            } else if variable_type == Variable::Name {
                                continue 'obj;
                            }
                        }
                    }
                }
            }
        }
        // Other files have the structure somewhat similar to Maps.json files
        else {
            // Skipping first element in array as it is null
            for obj in obj_arr.as_array().unwrap().iter().skip(1) {
                // CommonEvents doesn't have pages, so we can just check if it's Troops
                let pages_length: usize = if filename.starts_with("Tr") {
                    obj[if engine_type == EngineType::New {
                        "pages"
                    } else {
                        "__symbol__pages"
                    }]
                    .as_array()
                    .unwrap()
                    .len()
                } else {
                    1
                };

                for i in 0..pages_length {
                    let list: &Value = if pages_length != 1 {
                        &obj[if engine_type == EngineType::New {
                            "pages"
                        } else {
                            "__symbol__pages"
                        }][i][if engine_type == EngineType::New {
                            "list"
                        } else {
                            "__symbol__list"
                        }]
                    } else {
                        &obj[if engine_type == EngineType::New {
                            "list"
                        } else {
                            "__symbol__list"
                        }]
                    };

                    if !list.is_array() {
                        continue;
                    }

                    parse_list(
                        list.as_array().unwrap(),
                        &ALLOWED_CODES,
                        romanize,
                        game_type,
                        engine_type,
                        processing_mode,
                        &mut other_lines,
                        &mut other_translation_map,
                    );
                }
            }
        }

        let (original_content, translation_content) = if processing_mode == ProcessingMode::Append {
            let collected: (Vec<String>, Vec<String>) = other_translation_map.into_iter().unzip();
            (collected.0.join("\n"), collected.1.join("\n"))
        } else {
            let length: usize = other_lines.len().saturating_sub(1);
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

    let system_obj: Value = if engine_type == EngineType::New {
        from_str(&read_to_string(system_file_path).unwrap()).unwrap()
    } else {
        load(&read(system_file_path).unwrap(), None, Some(""))
    };

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

    if engine_type != EngineType::New {
        let str: &str = system_obj["__symbol__currency_unit"].as_str().unwrap().trim();

        if !str.is_empty() {
            let mut string: String = str.to_string();

            if romanize {
                string = romanize_string(string)
            }

            if processing_mode == ProcessingMode::Append && !system_translation_map.contains_key(&string) {
                system_translation_map.shift_insert(system_lines.len(), string.clone(), String::new());
            }

            system_lines.insert(string);
        }
    }

    // Armor types names
    // Normally it's system strings, but might be needed for some purposes
    for string in system_obj[if engine_type == EngineType::New {
        "armorTypes"
    } else {
        "__symbol__armor_types"
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
                system_translation_map.shift_insert(system_lines.len(), string.clone(), String::new());
            }

            system_lines.insert(string);
        }
    }

    // Element types names
    // Normally it's system strings, but might be needed for some purposes
    for string in system_obj[if engine_type == EngineType::New {
        "elements"
    } else {
        "__symbol__elements"
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
                system_translation_map.shift_insert(system_lines.len(), string.clone(), String::new());
            }

            system_lines.insert(string);
        }
    }

    // Names of equipment slots
    if engine_type == EngineType::New {
        for string in system_obj["equipTypes"].as_array().unwrap() {
            let str: &str = string.as_str().unwrap().trim();

            if !str.is_empty() {
                let mut string: String = str.to_string();

                if romanize {
                    string = romanize_string(string)
                }

                if processing_mode == ProcessingMode::Append && !system_translation_map.contains_key(&string) {
                    system_translation_map.shift_insert(system_lines.len(), string.clone(), String::new());
                }

                system_lines.insert(string);
            }
        }
    }

    // Names of battle options
    for string in system_obj[if engine_type == EngineType::New {
        "skillTypes"
    } else {
        "__symbol__skill_types"
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
                system_translation_map.shift_insert(system_lines.len(), string.clone(), String::new());
            }

            system_lines.insert(string);
        }
    }

    // Game terms vocabulary
    for (key, value) in system_obj[if engine_type == EngineType::XP {
        "__symbol__words"
    } else if engine_type == EngineType::New {
        "terms"
    } else {
        "__symbol__terms"
    }]
    .as_object()
    .unwrap()
    {
        if !key.starts_with("__symbol__") {
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
                            system_translation_map.shift_insert(system_lines.len(), string.clone(), String::new());
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
                        system_translation_map.shift_insert(system_lines.len(), string.clone(), String::new());
                    }

                    system_lines.insert(string);
                }
            }
        }
    }

    // Weapon types names
    // Normally it's system strings, but might be needed for some purposes
    for string in system_obj[if engine_type == EngineType::New {
        "weaponTypes"
    } else {
        "__symbol__weapon_types"
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
                system_translation_map.shift_insert(system_lines.len(), string.clone(), String::new());
            }

            system_lines.insert(string);
        }
    }

    // Game title, parsed just for fun
    // Translators may add something like "ELFISH TRANSLATION v1.0.0" to the title
    {
        let mut game_title_string: String = system_obj[if engine_type == EngineType::New {
            "gameTitle"
        } else {
            "__symbol__game_title"
        }]
        .as_str()
        .unwrap()
        .trim()
        .to_string();

        if romanize {
            game_title_string = romanize_string(game_title_string)
        }

        if processing_mode == ProcessingMode::Append && !system_translation_map.contains_key(&game_title_string) {
            system_translation_map.shift_insert(system_lines.len(), game_title_string.clone(), String::new());
        }

        system_lines.insert(game_title_string);
    }

    let (original_content, translated_content) = if processing_mode == ProcessingMode::Append {
        let collected: (Vec<String>, Vec<String>) = system_translation_map.into_iter().unzip();
        (collected.0.join("\n"), collected.1.join("\n"))
    } else {
        let length: usize = system_lines.len().saturating_sub(1);
        (system_lines.join("\n"), "\n".repeat(length))
    };

    write(system_output_path, original_content).unwrap();
    write(system_trans_output_path, translated_content).unwrap();

    if logging {
        println!("{file_parsed_msg} System.json.");
    }
}
