#![allow(clippy::too_many_arguments)]
use crate::{romanize_string, Code, EngineType, GameType, Variable, ENDS_WITH_IF_RE, LISA_PREFIX_RE, SELECT_WORDS_RE};
use fastrand::shuffle;
use marshal_rs::{dump::dump, load::load};
use rayon::prelude::*;
use regex::{Captures, Match};
use sonic_rs::{
    from_str, from_value, json, to_string, Array, JsonContainerTrait, JsonValueMutTrait, JsonValueTrait, Object, Value,
};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs::{read, read_dir, read_to_string, write, DirEntry},
    hash::BuildHasherDefault,
    path::Path,
    str::from_utf8_unchecked,
};
use xxhash_rust::xxh3::Xxh3;

pub fn shuffle_words(string: &str) -> String {
    let mut words: Vec<&str> = SELECT_WORDS_RE.find_iter(string).map(|m: Match| m.as_str()).collect();

    shuffle(&mut words);

    SELECT_WORDS_RE
        .replace_all(string, |_: &Captures| words.pop().unwrap_or(""))
        .into_owned()
}

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn get_translated_parameter<'a>(
    code: Code,
    mut parameter: &'a str,
    hashmap: &'a HashMap<String, String, BuildHasherDefault<Xxh3>>,
    game_type: &Option<GameType>,
    engine_type: &EngineType,
) -> Option<String> {
    let mut remaining_strings: Vec<String> = Vec::new();
    // bool indicates insert whether at start or at end
    // true inserts at end
    // false inserts at start
    let mut insert_positions: Vec<bool> = Vec::new();

    #[allow(clippy::collapsible_match)]
    if let Some(game_type) = game_type {
        match game_type {
            GameType::Termina => match code {
                Code::System => {
                    if !parameter.starts_with("Gab")
                        && (!parameter.starts_with("choice_text") || parameter.ends_with("????"))
                    {
                        return None;
                    }
                }
                _ => {}
            },
            GameType::LisaRPG => match code {
                Code::Dialogue => {
                    if let Some(re_match) = LISA_PREFIX_RE.find(parameter) {
                        parameter = &parameter[re_match.end()..];
                        remaining_strings.push(re_match.as_str().to_string());
                        insert_positions.push(false);
                    }
                }
                _ => {}
            },
            _ => {} // custom processing for other games
        }
    }

    if engine_type != EngineType::New {
        if let Some(re_match) = ENDS_WITH_IF_RE.find(parameter) {
            parameter = &parameter[re_match.start()..];
            remaining_strings.push(re_match.as_str().to_string());
            insert_positions.push(true);
        }
    }

    let translated: Option<String> = hashmap.get(parameter).map(|translated: &String| {
        let mut result: String = translated.to_owned();
        result
    });

    if let Some(mut translated) = translated {
        if translated.is_empty() {
            return None;
        }

        for (string, position) in remaining_strings.into_iter().zip(insert_positions.into_iter()) {
            match position {
                false => translated = string + &translated,
                true => translated += &string,
            }
        }

        Some(translated)
    } else {
        translated
    }
}

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn get_translated_variable(
    mut variable_text: String,
    note_text: Option<&str>, // note_text is some only when getting description
    variable_type: Variable,
    filename: &str,
    hashmap: &HashMap<String, String, BuildHasherDefault<Xxh3>>,
    game_type: &Option<GameType>,
    engine_type: &EngineType,
) -> Option<String> {
    let mut remaining_strings: Vec<String> = Vec::new();
    let mut insert_positions: Vec<bool> = Vec::new();

    if engine_type != EngineType::New {
        variable_text = variable_text.replace("\r\n", "\n");
    }

    #[allow(clippy::collapsible_match)]
    if let Some(game_type) = game_type {
        match game_type {
            GameType::Termina => match variable_type {
                Variable::Description => match note_text {
                    Some(mut note) => {
                        let mut note_string: String = String::from(note);

                        let mut note_chars: std::str::Chars = note.chars();
                        let mut is_continuation_of_description: bool = false;

                        if !note.starts_with("flesh puppetry") {
                            if let Some(first_char) = note_chars.next() {
                                if let Some(second_char) = note_chars.next() {
                                    if ((first_char == '\n' && second_char != '\n')
                                        || (first_char.is_ascii_alphabetic()
                                            || first_char == '"'
                                            || note.starts_with("4 sticks")))
                                        && !['.', '!', '/', '?'].contains(&first_char)
                                    {
                                        is_continuation_of_description = true;
                                    }
                                }
                            }
                        }

                        if is_continuation_of_description {
                            if let Some((mut left, _)) = note.trim_start().split_once('\n') {
                                left = left.trim();

                                if left.ends_with(['.', '%', '!', '"']) {
                                    note_string = "\n".to_string() + left;
                                }
                            } else if note.ends_with(['.', '%', '!', '"']) {
                                note_string = note.to_string();
                            }

                            if !note_string.is_empty() {
                                variable_text = variable_text + &note_string;
                            }
                        }
                    }
                    None => {}
                },
                Variable::Message1 | Variable::Message2 | Variable::Message3 | Variable::Message4 => {
                    return None;
                }
                Variable::Note => {
                    if filename.starts_with("It") {
                        for string in [
                            "<Menu Category: Items>",
                            "<Menu Category: Food>",
                            "<Menu Category: Healing>",
                            "<Menu Category: Body bag>",
                        ] {
                            if variable_text.contains(string) {
                                variable_text = variable_text.replace(string, &hashmap[string]);
                            }
                        }
                    }

                    if !filename.starts_with("Cl") {
                        let mut variable_text_chars: std::str::Chars = variable_text.chars();
                        let mut is_continuation_of_description: bool = false;

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

                        if is_continuation_of_description {
                            if let Some((_, right)) = variable_text.trim_start().split_once('\n') {
                                return Some(right.to_string());
                            } else {
                                return Some(String::new());
                            }
                        } else {
                            return Some(variable_text);
                        }
                    }
                }
                _ => {}
            },
            _ => {} // custom processing for other games
        }
    }

    let translated: Option<String> = hashmap.get(&variable_text).map(|translated: &String| {
        let mut result: String = translated.to_owned();

        for (string, position) in remaining_strings.into_iter().zip(insert_positions.into_iter()) {
            match position {
                true => {
                    result.push_str(&string);
                }
                false => result = string.to_owned() + &result,
            }
        }

        if matches!(
            variable_type,
            Variable::Message1 | Variable::Message2 | Variable::Message3 | Variable::Message4
        ) {
            result = " ".to_owned() + &result;
        }

        #[allow(clippy::collapsible_if, clippy::collapsible_match)]
        if let Some(game_type) = game_type {
            match game_type {
                GameType::Termina => {
                    match variable_type {
                        Variable::Note => {
                            if let Some(first_char) = result.chars().next() {
                                if first_char != '\n' {
                                    result = "\n".to_owned() + &result
                                }
                            }
                        }
                        _ => {}
                    }
                    if variable_type == Variable::Note {}
                }
                _ => {}
            }
        }
        result
    });

    if let Some(ref translated) = translated {
        if translated.is_empty() {
            return None;
        }
    }

    translated
}

fn write_list(
    list: &mut Array,
    allowed_codes: &[u64],
    romanize: bool,
    game_type: &Option<GameType>,
    engine_type: &EngineType,
    map: &HashMap<String, String, BuildHasherDefault<Xxh3>>,
) {
    let list_length: usize = list.len();

    let mut in_sequence: bool = false;
    let mut line: Vec<String> = Vec::with_capacity(4);
    let mut item_indices: Vec<usize> = Vec::with_capacity(4);

    for it in 0..list_length {
        let code: u64 = list[it][if engine_type == EngineType::New {
            "code"
        } else {
            "__symbol__code"
        }]
        .as_u64()
        .unwrap();

        let string_type: bool = !match code {
            320 | 324 | 356 | 401 | 405 => list[it][if engine_type == EngineType::New {
                "parameters"
            } else {
                "__symbol__parameters"
            }][0]
                .is_object(),
            102 => list[it][if engine_type == EngineType::New {
                "parameters"
            } else {
                "__symbol__parameters"
            }][0][0]
                .is_object(),
            402 => list[it][if engine_type == EngineType::New {
                "parameters"
            } else {
                "__symbol__parameters"
            }][1]
                .is_object(),
            _ => false,
        };

        if in_sequence && ![401, 405].contains(&code) {
            if !line.is_empty() {
                let mut joined: String = line.join("\n").trim().to_string();

                if romanize {
                    joined = romanize_string(joined);
                }

                let translated: Option<String> =
                    get_translated_parameter(Code::Dialogue, &joined, map, game_type, engine_type);

                if let Some(translated) = translated {
                    let split: Vec<&str> = translated.split('\n').collect();
                    let split_length: usize = split.len();
                    let line_length: usize = line.len();

                    for (i, &index) in item_indices.iter().enumerate() {
                        if i < split_length {
                            list[index][if engine_type == EngineType::New {
                                "parameters"
                            } else {
                                "__symbol__parameters"
                            }][0] = if !string_type {
                                json!({
                                    "__type": "bytes",
                                    "data": Array::from(split[i].as_bytes())
                                })
                            } else {
                                Value::from(split[i])
                            };
                        } else {
                            list[index][if engine_type == EngineType::New {
                                "parameters"
                            } else {
                                "__symbol__parameters"
                            }][0] = Value::from_static_str(" ");
                        }
                    }

                    if split_length > line_length {
                        let remaining: String = split[line_length - 1..].join("\n");

                        list[*item_indices.last().unwrap()][if engine_type == EngineType::New {
                            "parameters"
                        } else {
                            "__symbol__parameters"
                        }][0] = Value::from(&remaining);
                    }
                }

                line.clear();
                item_indices.clear();
            }

            in_sequence = false;
        }

        if !allowed_codes.contains(&code) {
            continue;
        }

        match code {
            401 | 405 => {
                let parameter_string: String = list[it][if engine_type == EngineType::New {
                    "parameters"
                } else {
                    "__symbol__parameters"
                }][0]
                    .as_str()
                    .map(|str: &str| str.to_string())
                    .unwrap_or_else(|| {
                        if let Some(parameter_obj) = list[it][if engine_type == EngineType::New {
                            "parameters"
                        } else {
                            "__symbol__parameters"
                        }][0]
                            .as_object()
                        {
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
                    line.push(parameter_string);
                    item_indices.push(it);
                    in_sequence = true;
                }
            }
            102 => {
                for i in 0..list[it][if engine_type == EngineType::New {
                    "parameters"
                } else {
                    "__symbol__parameters"
                }][0]
                    .as_array()
                    .unwrap()
                    .len()
                {
                    let mut subparameter_string: String = list[it][if engine_type == EngineType::New {
                        "parameters"
                    } else {
                        "__symbol__parameters"
                    }][0][i]
                        .as_str()
                        .map(|str: &str| str.to_string())
                        .unwrap_or_else(|| {
                            if let Some(parameter_obj) = list[it][if engine_type == EngineType::New {
                                "parameters"
                            } else {
                                "__symbol__parameters"
                            }][0][i]
                                .as_object()
                            {
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

                    if romanize {
                        subparameter_string = romanize_string(subparameter_string);
                    }

                    let translated: Option<String> =
                        get_translated_parameter(Code::Choice, &subparameter_string, map, game_type, engine_type);

                    if let Some(translated) = translated {
                        list[it][if engine_type == EngineType::New {
                            "parameters"
                        } else {
                            "__symbol__parameters"
                        }][0][i] = json!({"__type": "bytes", "data": Array::from(translated.as_bytes())});
                    }
                }
            }
            356 => {
                let mut parameter_string: String = list[it][if engine_type == EngineType::New {
                    "parameters"
                } else {
                    "__symbol__parameters"
                }][0]
                    .as_str()
                    .map(|str: &str| str.to_string())
                    .unwrap_or_else(|| {
                        if let Some(parameter_obj) = list[it][if engine_type == EngineType::New {
                            "parameters"
                        } else {
                            "__symbol__parameters"
                        }][0]
                            .as_object()
                        {
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

                if romanize {
                    parameter_string = romanize_string(parameter_string);
                }

                let translated: Option<String> =
                    get_translated_parameter(Code::System, &parameter_string, map, game_type, engine_type);

                if let Some(translated) = translated {
                    list[it][if engine_type == EngineType::New {
                        "parameters"
                    } else {
                        "__symbol__parameters"
                    }][0] = json!({"__type": "bytes", "data": Array::from(translated.as_bytes())});
                }
            }
            320 | 324 | 402 => {
                let mut parameter_string: String = list[it][if engine_type == EngineType::New {
                    "parameters"
                } else {
                    "__symbol__parameters"
                }][1]
                    .as_str()
                    .map(|str: &str| str.to_string())
                    .unwrap_or_else(|| {
                        if let Some(parameter_obj) = list[it][if engine_type == EngineType::New {
                            "parameters"
                        } else {
                            "__symbol__parameters"
                        }][1]
                            .as_object()
                        {
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

                if romanize {
                    parameter_string = romanize_string(parameter_string);
                }

                let translated: Option<String> =
                    get_translated_parameter(Code::Unknown, &parameter_string, map, game_type, engine_type);

                if let Some(translated) = translated {
                    list[it][if engine_type == EngineType::New {
                        "parameters"
                    } else {
                        "__symbol__parameters"
                    }][1] = json!({"__type": "bytes", "data": Array::from(translated.as_bytes())});
                }
            }
            _ => unreachable!(),
        }
    }
}

/// Writes .txt files from maps folder back to their initial form.
/// # Parameters
/// * `maps_path` - path to the maps directory
/// * `original_path` - path to the original directory
/// * `output_path` - path to the output directory
/// * `romanize` - if files were read with romanize, this option will romanize original game text to compare with parsed
/// * `shuffle_level` - level of shuffle
/// * `logging` - whether to log or not
/// * `file_written_msg` - message to log when file is written
/// * `game_type` - game type for custom parsing
pub fn write_maps(
    maps_path: &Path,
    original_path: &Path,
    output_path: &Path,
    romanize: bool,
    shuffle_level: u64,
    logging: bool,
    file_written_msg: &str,
    game_type: &Option<GameType>,
    engine_type: &EngineType,
) {
    let maps_obj_vec =
        read_dir(original_path)
            .unwrap()
            .par_bridge()
            .filter_map(|entry: Result<DirEntry, std::io::Error>| {
                if let Ok(entry) = entry {
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
                } else {
                    None
                }
            });

    let maps_original_text_vec: Vec<String> = read_to_string(maps_path.join("maps.txt"))
        .unwrap()
        .par_split('\n')
        .map(|line: &str| line.replace(r"\#", "\n").trim().to_string())
        .collect();

    let mut maps_translated_text_vec: Vec<String> = read_to_string(maps_path.join("maps_trans.txt"))
        .unwrap()
        .split('\n')
        .map(|line: &str| line.replace(r"\#", "\n").trim().to_string())
        .collect();

    let names_original_text_vec: Vec<String> = read_to_string(maps_path.join("names.txt"))
        .unwrap()
        .par_split('\n')
        .map(|line: &str| line.trim().to_string())
        .collect();

    let mut names_translated_text_vec: Vec<String> = read_to_string(maps_path.join("names_trans.txt"))
        .unwrap()
        .split('\n')
        .map(|line: &str| line.trim().to_string())
        .collect();

    if shuffle_level > 0 {
        shuffle(&mut maps_translated_text_vec);
        shuffle(&mut names_translated_text_vec);

        if shuffle_level == 2 {
            for (translated_text, translated_name_text) in maps_translated_text_vec
                .iter_mut()
                .zip(names_translated_text_vec.iter_mut())
            {
                *translated_text = shuffle_words(translated_text);
                *translated_name_text = shuffle_words(translated_name_text);
            }
        }
    }

    let maps_translation_map: HashMap<String, String, BuildHasherDefault<Xxh3>> = maps_original_text_vec
        .into_par_iter()
        .zip(maps_translated_text_vec.into_par_iter())
        .fold(
            HashMap::default,
            |mut map: HashMap<String, String, BuildHasherDefault<Xxh3>>, (key, value): (String, String)| {
                map.insert(key, value);
                map
            },
        )
        .reduce(HashMap::default, |mut a, b| {
            a.extend(b);
            a
        });

    let names_translation_map: HashMap<String, String, BuildHasherDefault<Xxh3>> = HashMap::from_iter(
        names_original_text_vec
            .into_iter()
            .zip(names_translated_text_vec)
            .collect::<Vec<(String, String)>>(),
    );

    // 401 - dialogue lines
    // 102 - dialogue choices array
    // 402 - one of the dialogue choices from the array
    // 356 - system lines (special texts)
    // 324 - i don't know what is it but it's some used in-game lines
    const ALLOWED_CODES: [u64; 6] = [102, 320, 324, 356, 401, 402];

    maps_obj_vec.into_par_iter().for_each(|(filename, mut obj)| {
        {
            let display_name_label: &str = if engine_type == EngineType::New {
                "displayName"
            } else {
                "__symbol__display_name"
            };

            if let Some(display_name) = obj[display_name_label].as_str() {
                let mut display_name: String = display_name.to_string();

                if romanize {
                    display_name = romanize_string(display_name)
                }

                if let Some(location_name) = names_translation_map.get(&display_name) {
                    obj[display_name_label] = Value::from(location_name);
                }
            }
        }

        // Skipping first element in array as it is null
        let mut events_arr: Vec<&mut Value> = if engine_type == EngineType::New {
            obj["events"].as_array_mut().unwrap().par_iter_mut().skip(1).collect()
        } else {
            obj["__symbol__events"]
                .as_object_mut()
                .unwrap()
                .iter_mut()
                .par_bridge()
                .map(|(_, value)| value)
                .collect()
        };

        events_arr.par_iter_mut().for_each(|event: &mut &mut Value| {
            if event.is_null() {
                return;
            }

            event[if engine_type == EngineType::New {
                "pages"
            } else {
                "__symbol__pages"
            }]
            .as_array_mut()
            .unwrap()
            .par_iter_mut()
            .for_each(|page: &mut Value| {
                write_list(
                    page[if engine_type == EngineType::New {
                        "list"
                    } else {
                        "__symbol__list"
                    }]
                    .as_array_mut()
                    .unwrap(),
                    &ALLOWED_CODES,
                    romanize,
                    game_type,
                    engine_type,
                    &maps_translation_map,
                );
            });
        });

        let output_data: Vec<u8> = if engine_type == EngineType::New {
            to_string(&obj).unwrap().into_bytes()
        } else {
            dump(obj, Some(""))
        };

        if logging {
            println!("{file_written_msg} {filename}");
        }

        write(output_path.join(filename), output_data).unwrap();
    });
}

/// Writes .txt files from other folder back to their initial form.
/// # Parameters
/// * `other_path` - path to the other directory
/// * `original_path` - path to the original directory
/// * `output_path` - path to the output directory
/// * `romanize` - if files were read with romanize, this option will romanize original game text to compare with parsed
/// * `shuffle_level` - level of shuffle
/// * `logging` - whether to log or not
/// * `file_written_msg` - message to log when file is written
/// * `game_type` - game type for custom parsing
pub fn write_other(
    other_path: &Path,
    original_path: &Path,
    output_path: &Path,
    romanize: bool,
    shuffle_level: u64,
    logging: bool,
    file_written_msg: &str,
    game_type: &Option<GameType>,
    engine_type: &EngineType,
) {
    let other_obj_arr_vec =
        read_dir(original_path)
            .unwrap()
            .par_bridge()
            .filter_map(|entry: Result<DirEntry, std::io::Error>| {
                if let Ok(entry) = entry {
                    let filename_os_string: OsString = entry.file_name();
                    let filename: &str = unsafe { from_utf8_unchecked(filename_os_string.as_encoded_bytes()) };
                    let (real_name, extension) = filename.split_once('.').unwrap();

                    if !real_name.starts_with("Map")
                        && !matches!(real_name, "Tilesets" | "Animations" | "System" | "Scripts")
                        && ["json", "rvdata2", "rvdata", "rxdata"].contains(&extension)
                    {
                        if game_type
                            .as_ref()
                            .is_some_and(|game_type: &GameType| game_type == GameType::Termina)
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
                } else {
                    None
                }
            });

    // 401 - dialogue lines
    // 405 - credits lines
    // 102 - dialogue choices array
    // 402 - one of the dialogue choices from the array
    // 356 - system lines (special texts)
    // 324 - i don't know what is it but it's some used in-game lines
    const ALLOWED_CODES: [u64; 7] = [102, 320, 324, 356, 401, 402, 405];

    other_obj_arr_vec.into_par_iter().for_each(|(filename, mut obj_arr)| {
        let other_processed_filename: &str = filename.rsplit_once(".").unwrap().0;

        let other_original_text: Vec<String> =
            read_to_string(other_path.join(other_processed_filename.to_owned() + ".txt"))
                .unwrap()
                .par_split('\n')
                .map(|line: &str| line.replace(r"\#", "\n").trim().to_string())
                .collect();

        let mut other_translated_text: Vec<String> =
            read_to_string(other_path.join(other_processed_filename.to_owned() + "_trans.txt"))
                .unwrap()
                .par_split('\n')
                .map(|line: &str| line.replace(r"\#", "\n").trim().to_string())
                .collect();

        if shuffle_level > 0 {
            shuffle(&mut other_translated_text);
            if shuffle_level == 2 {
                for translated_text in other_translated_text.iter_mut() {
                    *translated_text = shuffle_words(translated_text);
                }
            }
        }

        let other_translation_map: HashMap<String, String, BuildHasherDefault<Xxh3>> = other_original_text
            .into_par_iter()
            .zip(other_translated_text.into_par_iter())
            .fold(
                HashMap::default,
                |mut map: HashMap<String, String, BuildHasherDefault<Xxh3>>, (key, value): (String, String)| {
                    map.insert(key, value);
                    map
                },
            )
            .reduce(HashMap::default, |mut a, b| {
                a.extend(b);
                a
            });

        // Other files except CommonEvents.json and Troops.json have the structure that consists
        // of name, nickname, description and note
        if !filename.starts_with("Co") && !filename.starts_with("Tr") {
            obj_arr
                .as_array_mut()
                .unwrap()
                .par_iter_mut()
                .skip(1) // Skipping first element in array as it is null
                .for_each(|obj: &mut Value| {
                    for (variable_label, variable_type) in [
                        (
                            if engine_type == EngineType::New {
                                "name"
                            } else {
                                "__symbol__name"
                            },
                            Variable::Name,
                        ),
                        (
                            if engine_type == EngineType::New {
                                "nickname"
                            } else {
                                "__symbol__nickname"
                            },
                            Variable::Nickname,
                        ),
                        (
                            if engine_type == EngineType::New {
                                "description"
                            } else {
                                "__symbol__description"
                            },
                            Variable::Description,
                        ),
                        (
                            if engine_type == EngineType::New {
                                "message1"
                            } else {
                                "__symbol__message1"
                            },
                            Variable::Message1,
                        ),
                        (
                            if engine_type == EngineType::New {
                                "message2"
                            } else {
                                "__symbol__message2"
                            },
                            Variable::Message2,
                        ),
                        (
                            if engine_type == EngineType::New {
                                "message3"
                            } else {
                                "__symbol__message3"
                            },
                            Variable::Message3,
                        ),
                        (
                            if engine_type == EngineType::New {
                                "message4"
                            } else {
                                "__symbol__message4"
                            },
                            Variable::Message4,
                        ),
                        (
                            if engine_type == EngineType::New {
                                "note"
                            } else {
                                "__symbol__note"
                            },
                            Variable::Note,
                        ),
                    ] {
                        if let Some(variable_str) = obj[variable_label].as_str() {
                            let mut variable_string: String = if variable_type != Variable::Note {
                                variable_str.trim().to_string()
                            } else {
                                variable_str.to_string()
                            };

                            if !variable_string.is_empty() {
                                if romanize {
                                    variable_string = romanize_string(variable_string)
                                }

                                variable_string = variable_string
                                    .split('\n')
                                    .map(|line: &str| line.trim())
                                    .collect::<Vec<_>>()
                                    .join("\n");

                                let note_text: Option<&str> = if game_type.is_some()
                                    && game_type.as_ref().unwrap() != GameType::Termina
                                    && variable_type != Variable::Description
                                {
                                    None
                                } else {
                                    match obj.get(if engine_type == EngineType::New {
                                        "note"
                                    } else {
                                        "__symbol__note"
                                    }) {
                                        Some(value) => value.as_str(),
                                        None => None,
                                    }
                                };

                                let translated: Option<String> = get_translated_variable(
                                    variable_string,
                                    note_text,
                                    variable_type,
                                    &filename,
                                    &other_translation_map,
                                    game_type,
                                    engine_type,
                                );

                                if let Some(translated) = translated {
                                    obj[variable_label] = Value::from(&translated);
                                }
                            }
                        }
                    }
                });
        } else {
            // Other files have the structure somewhat similar to Maps.json files
            obj_arr
                .as_array_mut()
                .unwrap()
                .par_iter_mut()
                .skip(1) // Skipping first element in array as it is null
                .for_each(|obj: &mut Value| {
                    // CommonEvents doesn't have pages, so we can just check if it's Troops
                    let pages_length: usize = if filename.starts_with("Troops") {
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
                        // If element has pages, then we'll iterate over them
                        // Otherwise we'll just iterate over the list
                        let list_value: &mut Value = if pages_length != 1 {
                            &mut obj[if engine_type == EngineType::New {
                                "pages"
                            } else {
                                "__symbol__pages"
                            }][i][if engine_type == EngineType::New {
                                "list"
                            } else {
                                "__symbol__list"
                            }]
                        } else {
                            &mut obj[if engine_type == EngineType::New {
                                "list"
                            } else {
                                "__symbol__list"
                            }]
                        };

                        if !list_value.is_array() {
                            continue;
                        }

                        write_list(
                            list_value.as_array_mut().unwrap(),
                            &ALLOWED_CODES,
                            romanize,
                            game_type,
                            engine_type,
                            &other_translation_map,
                        );
                    }
                });
        }
        let output_data: Vec<u8> = if engine_type == EngineType::New {
            to_string(&obj_arr).unwrap().into_bytes()
        } else {
            dump(obj_arr, Some(""))
        };

        if logging {
            println!("{file_written_msg} {filename}");
        }

        write(output_path.join(filename), output_data).unwrap();
    });
}

/// Writes system.txt file back to its initial form.
///
/// For inner code documentation, check read_system function.
/// # Parameters
/// * `system_file_path` - path to the original system file
/// * `other_path` - path to the other directory
/// * `output_path` - path to the output directory
/// * `romanize` - if files were read with romanize, this option will romanize original game text to compare with parsed
/// * `shuffle_level` - level of shuffle
/// * `logging` - whether to log or not
/// * `file_written_msg` - message to log when file is written
pub fn write_system(
    system_file_path: &Path,
    other_path: &Path,
    output_path: &Path,
    romanize: bool,
    shuffle_level: u64,
    logging: bool,
    file_written_msg: &str,
    engine_type: &EngineType,
) {
    let mut system_obj: Value = if engine_type == EngineType::New {
        from_str(&read_to_string(system_file_path).unwrap()).unwrap()
    } else {
        load(&read(system_file_path).unwrap(), None, Some(""))
    };

    let system_original_text: Vec<String> = read_to_string(other_path.join("system.txt"))
        .unwrap()
        .split('\n')
        .map(|line: &str| line.trim().to_string())
        .collect();

    let system_translated_text: (String, String) = read_to_string(other_path.join("system_trans.txt"))
        .unwrap()
        .rsplit_once('\n')
        .map(|(left, right)| (left.to_string(), right.to_string()))
        .unwrap();

    let game_title: String = system_translated_text.1;

    let mut system_translated_text: Vec<String> = system_translated_text
        .0
        .split('\n')
        .map(|line: &str| line.trim().to_string())
        .collect();

    if shuffle_level > 0 {
        shuffle(&mut system_translated_text);
        if shuffle_level == 2 {
            for translated_text in system_translated_text.iter_mut() {
                *translated_text = shuffle_words(translated_text);
            }
        }
    }

    let system_translation_map: HashMap<String, String, BuildHasherDefault<Xxh3>> = system_original_text
        .into_par_iter()
        .zip(system_translated_text.into_par_iter())
        .fold(
            HashMap::default,
            |mut map: HashMap<String, String, BuildHasherDefault<Xxh3>>, (key, value): (String, String)| {
                map.insert(key, value);
                map
            },
        )
        .reduce(HashMap::default, |mut a, b| {
            a.extend(b);
            a
        });

    if engine_type != EngineType::New {
        let mut string: String = system_obj["__symbol__currency_unit"]
            .as_str()
            .unwrap()
            .trim()
            .to_string();

        if romanize {
            string = romanize_string(string);
        }

        if let Some(translated) = system_translation_map.get(&string) {
            if !translated.is_empty() {
                system_obj["__symbol__currency_unit"] = Value::from(translated);
            }
        }
    }

    system_obj[if engine_type == EngineType::New {
        "armorTypes"
    } else {
        "__symbol__armor_types"
    }]
    .as_array_mut()
    .unwrap()
    .iter_mut()
    .for_each(|value: &mut Value| {
        let mut string: String = value.as_str().unwrap().trim().to_string();

        if romanize {
            string = romanize_string(string);
        }

        if let Some(translated) = system_translation_map.get(&string) {
            if translated.is_empty() {
                return;
            }

            *value = Value::from(translated);
        }
    });

    system_obj[if engine_type == EngineType::New {
        "elements"
    } else {
        "__symbol__elements"
    }]
    .as_array_mut()
    .unwrap()
    .iter_mut()
    .for_each(|value: &mut Value| {
        let mut string: String = value.as_str().unwrap().trim().to_string();

        if romanize {
            string = romanize_string(string);
        }

        if let Some(translated) = system_translation_map.get(&string) {
            if translated.is_empty() {
                return;
            }

            *value = Value::from(translated);
        }
    });

    if engine_type == EngineType::New {
        system_obj["equipTypes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .for_each(|value: &mut Value| {
                let mut string: String = value.as_str().unwrap().trim().to_string();

                if romanize {
                    string = romanize_string(string);
                }

                if let Some(translated) = system_translation_map.get(&string) {
                    if translated.is_empty() {
                        return;
                    }

                    *value = Value::from(translated);
                }
            });
    }

    system_obj[if engine_type == EngineType::New {
        "skillTypes"
    } else {
        "__symbol__skill_types"
    }]
    .as_array_mut()
    .unwrap()
    .iter_mut()
    .for_each(|value: &mut Value| {
        let mut string: String = value.as_str().unwrap().trim().to_string();

        if romanize {
            string = romanize_string(string);
        }

        if let Some(translated) = system_translation_map.get(&string) {
            if translated.is_empty() {
                return;
            }

            *value = Value::from(translated);
        }
    });

    system_obj[if engine_type == EngineType::XP {
        "__symbol__words"
    } else if engine_type == EngineType::New {
        "terms"
    } else {
        "__symbol__terms"
    }]
    .as_object_mut()
    .unwrap()
    .iter_mut()
    .for_each(|(key, value): (&str, &mut Value)| {
        if !key.starts_with("__symbol__") {
            return;
        }

        if key != "messages" {
            value
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .for_each(|subvalue: &mut Value| {
                    if let Some(str) = subvalue.as_str() {
                        let mut string: String = str.trim().to_string();

                        if romanize {
                            string = romanize_string(string);
                        }

                        if let Some(translated) = system_translation_map.get(&string) {
                            if translated.is_empty() {
                                return;
                            }

                            *subvalue = Value::from(translated);
                        }
                    }
                });
        } else {
            if !value.is_object() {
                return;
            }

            value.as_object_mut().unwrap().iter_mut().for_each(|(_, value)| {
                let mut string: String = value.as_str().unwrap().trim().to_string();

                if romanize {
                    string = romanize_string(string)
                }

                if let Some(translated) = system_translation_map.get(&string) {
                    if translated.is_empty() {
                        return;
                    }

                    *value = Value::from(translated);
                }
            });
        }
    });

    system_obj[if engine_type == EngineType::New {
        "weaponTypes"
    } else {
        "__symbol__weapon_types"
    }]
    .as_array_mut()
    .unwrap()
    .iter_mut()
    .for_each(|value: &mut Value| {
        let mut string: String = value.as_str().unwrap().trim().to_string();

        if romanize {
            string = romanize_string(string);
        }

        if let Some(translated) = system_translation_map.get(&string) {
            if translated.is_empty() {
                return;
            }

            *value = Value::from(translated);
        }
    });

    system_obj[if engine_type == EngineType::New {
        "gameTitle"
    } else {
        "__symbol__game_title"
    }] = Value::from(&game_title);

    let output_data: Vec<u8> = if engine_type == EngineType::New {
        to_string(&system_obj).unwrap().into_bytes()
    } else {
        dump(system_obj, Some(""))
    };

    let filename = system_file_path.file_name().unwrap().to_str().unwrap();

    write(output_path.join(filename), output_data).unwrap();

    if logging {
        println!("{file_written_msg} {filename}");
    }
}

/// Writes plugins.txt file back to its initial form. Currently works only if game_type is GameType::Termina.
/// # Parameters
/// * `plugins_file_path` - path to the original plugins file
/// * `plugins_path` - path to the plugins directory
/// * `output_path` - path to the output directory
/// * `shuffle_level` - level of shuffle
/// * `logging` - whether to log or not
/// * `file_written_msg` - message to log when file is written
pub fn write_plugins(
    plugins_file_path: &Path,
    plugins_path: &Path,
    output_path: &Path,
    shuffle_level: u64,
    logging: bool,
    file_written_msg: &str,
) {
    let mut obj_arr: Vec<Object> = from_str(&read_to_string(plugins_file_path).unwrap()).unwrap();

    let plugins_original_text: Vec<String> = read_to_string(plugins_path.join("plugins.txt"))
        .unwrap()
        .par_split('\n')
        .map(|line: &str| line.to_string())
        .collect();

    let mut plugins_translated_text: Vec<String> = read_to_string(plugins_path.join("plugins_trans.txt"))
        .unwrap()
        .par_split('\n')
        .map(|line: &str| line.to_string())
        .collect();

    if shuffle_level > 0 {
        shuffle(&mut plugins_translated_text);

        if shuffle_level == 2 {
            for translated_text in plugins_translated_text.iter_mut() {
                *translated_text = shuffle_words(translated_text);
            }
        }
    }

    let plugins_translation_map: HashMap<String, String, BuildHasherDefault<Xxh3>> = plugins_original_text
        .into_par_iter()
        .zip(plugins_translated_text.into_par_iter())
        .fold(
            HashMap::default,
            |mut map: HashMap<String, String, BuildHasherDefault<Xxh3>>, (key, value): (String, String)| {
                map.insert(key, value);
                map
            },
        )
        .reduce(HashMap::default, |mut a, b| {
            a.extend(b);
            a
        });

    obj_arr.par_iter_mut().for_each(|obj: &mut Object| {
        // For now, plugins writing only implemented for Fear & Hunger: Termina, so you should manually translate the plugins.js file if it's not Termina

        // Plugins with needed text
        let plugin_names: HashSet<&str, BuildHasherDefault<Xxh3>> = HashSet::from_iter([
            "YEP_BattleEngineCore",
            "YEP_OptionsCore",
            "SRD_NameInputUpgrade",
            "YEP_KeyboardConfig",
            "YEP_ItemCore",
            "YEP_X_ItemDiscard",
            "YEP_EquipCore",
            "YEP_ItemSynthesis",
            "ARP_CommandIcons",
            "YEP_X_ItemCategories",
            "Olivia_OctoBattle",
        ]);

        let name: &str = obj["name"].as_str().unwrap();

        // It it's a plugin with the needed text, proceed
        if plugin_names.contains(name) {
            // YEP_OptionsCore should be processed differently, as its parameters is a mess, that can't even be parsed to json
            if name == "YEP_OptionsCore" {
                obj["parameters"]
                    .as_object_mut()
                    .unwrap()
                    .iter_mut()
                    .par_bridge()
                    .for_each(|(key, value): (&str, &mut Value)| {
                        let mut string: String = value.as_str().unwrap().to_string();

                        if key == "OptionsCategories" {
                            for (text, translated) in
                                plugins_translation_map.keys().zip(plugins_translation_map.values())
                            {
                                string = string.replacen(text, translated, 1);
                            }

                            *value = Value::from(&string);
                        } else if let Some(translated) = plugins_translation_map.get(&string) {
                            *value = Value::from(translated);
                        }
                    });
            }
            // Everything else is an easy walk
            else {
                obj["parameters"]
                    .as_object_mut()
                    .unwrap()
                    .iter_mut()
                    .par_bridge()
                    .for_each(|(_, value)| {
                        if let Some(str) = value.as_str() {
                            if let Some(translated) = plugins_translation_map.get(str) {
                                *value = Value::from(translated);
                            }
                        }
                    });
            }
        }
    });

    write(
        output_path.join("plugins.js"),
        String::from("var $plugins =\n") + &to_string(&obj_arr).unwrap(),
    )
    .unwrap();

    if logging {
        println!("{file_written_msg} plugins.js");
    }
}
