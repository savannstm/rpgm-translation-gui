use crate::{romanize_string, Code, GameType, IntoRSplit, Variable};
use fancy_regex::{Captures, Error, Match, Regex};
use fastrand::shuffle;
use rayon::prelude::*;
use sonic_rs::{
    from_str, to_string, to_value, Array, JsonContainerTrait, JsonValueMutTrait, JsonValueTrait,
    Object, Value,
};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs::{read_dir, read_to_string, write, DirEntry},
    hash::BuildHasherDefault,
    path::Path,
    str::from_utf8_unchecked,
};
use xxhash_rust::xxh3::Xxh3;

pub fn shuffle_words(string: &str) -> String {
    let re: Regex = Regex::new(r"\S+").unwrap();
    let mut words: Vec<&str> = re
        .find_iter(string)
        .filter_map(|m: Result<Match, Error>| m.ok().map(|m: Match| m.as_str()))
        .collect();

    shuffle(&mut words);

    re.replace_all(string, |_: &Captures| words.pop().unwrap_or(""))
        .into_owned()
}

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn get_parameter_translated<'a>(
    code: Code,
    mut parameter: &'a str,
    hashmap: &'a HashMap<String, String, BuildHasherDefault<Xxh3>>,
    game_type: &Option<GameType>,
) -> Option<String> {
    if let Some(game_type) = game_type {
        match game_type {
            GameType::Termina => match code {
                Code::Dialogue => {}
                Code::Choice => {}
                Code::System => {
                    if !parameter.starts_with("Gab")
                        && (!parameter.starts_with("choice_text") || parameter.ends_with("????"))
                    {
                        return None;
                    }
                }
                Code::Unknown => {}
            },
            // custom processing for other games
        }
    }

    hashmap.get(parameter).map(|s: &String| s.to_owned())
}

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn get_variable_translated(
    mut variable_text: &str,
    variable_name: Variable,
    filename: &str,
    hashmap: &HashMap<String, String, BuildHasherDefault<Xxh3>>,
    game_type: &Option<GameType>,
) -> Option<String> {
    if let Some(game_type) = game_type {
        match game_type {
            GameType::Termina => match variable_name {
                Variable::Name => {}
                Variable::Nickname => {}
                Variable::Description => {}
                Variable::Note => {
                    if filename.starts_with("It") {
                        for string in [
                            "<Menu Category: Items>",
                            "<Menu Category: Food>",
                            "<Menu Category: Healing>",
                            "<Menu Category: Body bag>",
                        ] {
                            if variable_text.contains(string) {
                                return Some(variable_text.replacen(string, &hashmap[string], 1));
                            }
                        }
                    }
                }
            },
            // custom processing for other games
        }
    }

    hashmap.get(variable_text).map(|s: &String| s.to_owned())
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
) {
    let maps_obj_vec: Vec<(String, Object)> = read_dir(original_path)
        .unwrap()
        .par_bridge()
        .fold(
            Vec::new,
            |mut vec: Vec<(String, Object)>, entry: Result<DirEntry, _>| match entry {
                Ok(entry) => {
                    let filename: OsString = entry.file_name();
                    let filename_str: &str =
                        unsafe { from_utf8_unchecked(filename.as_encoded_bytes()) };

                    let slice: char;
                    unsafe {
                        slice = *filename_str.as_bytes().get_unchecked(4) as char;
                    }

                    if filename_str.starts_with("Map")
                        && slice.is_ascii_digit()
                        && filename_str.ends_with("json")
                    {
                        vec.push((
                            filename_str.to_string(),
                            from_str(&read_to_string(entry.path()).unwrap()).unwrap(),
                        ))
                    }
                    vec
                }
                Err(_) => vec![],
            },
        )
        .reduce(Vec::new, |mut a, b| {
            a.extend(b);
            a
        });

    let maps_original_text_vec: Vec<String> = read_to_string(maps_path.join("maps.txt"))
        .unwrap()
        .par_split('\n')
        .map(|line: &str| line.replace(r"\#", "\n").trim().to_string())
        .collect();

    let names_original_text_vec: Vec<String> = read_to_string(maps_path.join("names.txt"))
        .unwrap()
        .par_split('\n')
        .map(|line: &str| line.replace(r"\#", "\n").trim().to_string())
        .collect();

    let mut maps_translated_text_vec: Vec<String> =
        read_to_string(maps_path.join("maps_trans.txt"))
            .unwrap()
            .par_split('\n')
            .map(|line: &str| line.replace(r"\#", "\n").trim().to_string())
            .collect();

    let mut names_translated_text_vec: Vec<String> =
        read_to_string(maps_path.join("names_trans.txt"))
            .unwrap()
            .par_split('\n')
            .map(|line: &str| line.replace(r"\#", "\n").trim().to_string())
            .collect();

    match shuffle_level {
        1 => {
            shuffle(&mut maps_translated_text_vec);
            shuffle(&mut names_translated_text_vec);
        }
        2 => {
            for (text_string, name_string) in maps_translated_text_vec
                .iter_mut()
                .zip(names_translated_text_vec.iter_mut())
            {
                *text_string = shuffle_words(text_string);
                *name_string = shuffle_words(name_string);
            }
        }
        _ => {}
    }

    let maps_translation_map: HashMap<String, String, BuildHasherDefault<Xxh3>> =
        maps_original_text_vec
            .into_par_iter()
            .zip(maps_translated_text_vec.into_par_iter())
            .fold(
                HashMap::default,
                |mut map: HashMap<String, String, BuildHasherDefault<Xxh3>>,
                 (key, value): (String, String)| {
                    map.insert(key, value);
                    map
                },
            )
            .reduce(HashMap::default, |mut a, b| {
                a.extend(b);
                a
            });

    let names_translation_map: HashMap<String, String, BuildHasherDefault<Xxh3>> =
        names_original_text_vec
            .into_par_iter()
            .zip(names_translated_text_vec.into_par_iter())
            .fold(
                HashMap::default,
                |mut map: HashMap<String, String, BuildHasherDefault<Xxh3>>,
                 (key, value): (String, String)| {
                    map.insert(key, value);
                    map
                },
            )
            .reduce(HashMap::default, |mut a, b| {
                a.extend(b);
                a
            });

    // 401 - dialogue lines
    // 102 - dialogue choices array
    // 402 - one of the dialogue choices from the array
    // 356 - system lines (special texts)
    // 324 - i don't know what is it but it's some used in-game lines
    const ALLOWED_CODES: [u64; 5] = [401, 102, 402, 356, 324];

    maps_obj_vec
        .into_par_iter()
        .for_each(|(filename, mut obj)| {
            let mut display_name: String = obj["displayName"].as_str().unwrap().to_string();

            if romanize {
                display_name = romanize_string(display_name)
            }

            if let Some(location_name) = names_translation_map.get(&display_name) {
                obj["displayName"] = to_value(location_name).unwrap();
            }

            drop(display_name);

            obj["events"]
                .as_array_mut()
                .unwrap()
                .par_iter_mut()
                .skip(1) //Skipping first element in array as it is null
                .for_each(|event: &mut Value| {
                    if event.is_null() {
                        return;
                    }

                    event["pages"]
                        .as_array_mut()
                        .unwrap()
                        .par_iter_mut()
                        .for_each(|page: &mut Value| {
                            let mut in_sequence: bool = false;
                            let mut line: Vec<String> = Vec::with_capacity(4);
                            let mut item_indices: Vec<usize> = Vec::with_capacity(4);

                            let list: &mut Array = page["list"].as_array_mut().unwrap();
                            let list_len: usize = list.len();

                            for it in 0..list_len {
                                let code: u64 = list[it]["code"].as_u64().unwrap();

                                if !ALLOWED_CODES.contains(&code) {
                                    if in_sequence {
                                        let mut joined: String = line.join("\n").trim().to_string();

                                        if romanize {
                                            joined = romanize_string(joined);
                                        }

                                        let translated: Option<String> = get_parameter_translated(
                                            Code::Dialogue,
                                            &joined,
                                            &maps_translation_map,
                                            game_type,
                                        );

                                        if let Some(text) = translated {
                                            let split: Vec<&str> = text.split('\n').collect();
                                            let split_length: usize = split.len();
                                            let line_length: usize = line.len();

                                            for (i, &index) in item_indices.iter().enumerate() {
                                                if i < split_length {
                                                    list[index]["parameters"][0] =
                                                        to_value(split[i]).unwrap();
                                                } else {
                                                    list[index]["parameters"][0] =
                                                        to_value("").unwrap();
                                                }
                                            }

                                            if split_length > line_length {
                                                let remaining: String =
                                                    split[line_length..].join("\n");

                                                list[*item_indices.last().unwrap()]["parameters"]
                                                    [0] = to_value(&remaining).unwrap();
                                            }
                                        }

                                        line.clear();
                                        item_indices.clear();
                                        in_sequence = false;
                                    }
                                    continue;
                                }

                                if code == 401 {
                                    if let Some(parameter_str) = list[it]["parameters"][0].as_str()
                                    {
                                        line.push(parameter_str.to_string());
                                        item_indices.push(it);
                                        in_sequence = true;
                                    }
                                } else if list[it]["parameters"][0].is_array() {
                                    for i in 0..list[it]["parameters"][0].as_array().unwrap().len()
                                    {
                                        if let Some(subparameter_str) =
                                            list[it]["parameters"][0][i].as_str()
                                        {
                                            let mut subparameter_string: String =
                                                subparameter_str.to_string();

                                            if romanize {
                                                subparameter_string =
                                                    romanize_string(subparameter_string);
                                            }

                                            let translated: Option<String> =
                                                get_parameter_translated(
                                                    Code::Choice,
                                                    &subparameter_string,
                                                    &maps_translation_map,
                                                    game_type,
                                                );

                                            if let Some(translated) = translated {
                                                list[it]["parameters"][0][i] =
                                                    to_value(&translated).unwrap();
                                            }
                                        }
                                    }
                                } else if let Some(parameter_str) =
                                    list[it]["parameters"][0].as_str()
                                {
                                    let mut parameter_string: String = parameter_str.to_string();

                                    if romanize {
                                        parameter_string = romanize_string(parameter_string);
                                    }

                                    let translated: Option<String> = get_parameter_translated(
                                        Code::System,
                                        &parameter_string,
                                        &maps_translation_map,
                                        game_type,
                                    );

                                    if let Some(translated) = translated {
                                        list[it]["parameters"][0] = to_value(&translated).unwrap();
                                    }
                                } else if let Some(parameter_str) =
                                    list[it]["parameters"][1].as_str()
                                {
                                    let mut parameter_string: String = parameter_str.to_string();

                                    if romanize {
                                        parameter_string = romanize_string(parameter_string);
                                    }

                                    let translated: Option<String> = get_parameter_translated(
                                        Code::Unknown,
                                        &parameter_string,
                                        &maps_translation_map,
                                        game_type,
                                    );

                                    if let Some(translated) = translated {
                                        list[it]["parameters"][1] = to_value(&translated).unwrap();
                                    }
                                }
                            }
                        });
                });

            write(output_path.join(&filename), to_string(&obj).unwrap()).unwrap();

            if logging {
                println!("{file_written_msg} {filename}");
            }
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
) {
    let other_obj_arr_vec: Vec<(String, Array)> = read_dir(original_path)
        .unwrap()
        .par_bridge()
        .fold(
            Vec::new,
            |mut vec: Vec<(String, Array)>, entry: Result<DirEntry, _>| match entry {
                Ok(entry) => {
                    let filename_os_string: OsString = entry.file_name();
                    let filename: &str =
                        unsafe { from_utf8_unchecked(filename_os_string.as_encoded_bytes()) };
                    let (real_name, extension) = filename.split_once('.').unwrap();

                    if !real_name.starts_with("Map")
                        && !matches!(real_name, "Tilesets" | "Animations" | "States" | "System")
                        && extension == "json"
                    {
                        vec.push((
                            filename.to_string(),
                            from_str(&read_to_string(entry.path()).unwrap()).unwrap(),
                        ));
                    }
                    vec
                }
                Err(_) => vec![],
            },
        )
        .reduce(Vec::new, |mut a, b| {
            a.extend(b);
            a
        });

    // 401 - dialogue lines
    // 405 - credits lines
    // 102 - dialogue choices array
    // 402 - one of the dialogue choices from the array
    // 356 - system lines (special texts)
    // 324 - i don't know what is it but it's some used in-game lines
    const ALLOWED_CODES: [u64; 6] = [401, 402, 405, 356, 102, 324];

    other_obj_arr_vec
        .into_par_iter()
        .for_each(|(filename, mut obj_arr)| {
            let other_processed_filename: &str = &filename[..filename.len() - 5];

            let other_original_text: Vec<String> =
                read_to_string(other_path.join(format!("{other_processed_filename}.txt")))
                    .unwrap()
                    .par_split('\n')
                    .map(|line: &str| line.replace(r"\#", "\n").trim().to_string())
                    .collect();

            let mut other_translated_text: Vec<String> =
                read_to_string(other_path.join(format!("{other_processed_filename}_trans.txt")))
                    .unwrap()
                    .par_split('\n')
                    .map(|line: &str| line.replace(r"\#", "\n").trim().to_string())
                    .collect();

            match shuffle_level {
                1 => {
                    shuffle(&mut other_translated_text);
                }
                2 => {
                    for text_string in other_translated_text.iter_mut() {
                        *text_string = shuffle_words(text_string);
                    }
                }
                _ => {}
            }

            let other_translation_map: HashMap<String, String, BuildHasherDefault<Xxh3>> =
                other_original_text
                    .into_par_iter()
                    .zip(other_translated_text.into_par_iter())
                    .fold(
                        HashMap::default,
                        |mut map: HashMap<String, String, BuildHasherDefault<Xxh3>>,
                         (key, value): (String, String)| {
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
                    .par_iter_mut()
                    .skip(1) // Skipping first element in array as it is null
                    .for_each(|obj: &mut Value| {
                        for (variable_name, variable_enum) in [
                            ("name", Variable::Name),
                            ("nickname", Variable::Nickname),
                            ("description", Variable::Description),
                            ("note", Variable::Note),
                        ] {
                            if let Some(variable_value) = obj.get(variable_name) {
                                if let Some(variable_str) = variable_value.as_str() {
                                    let mut variable_string: String =
                                        variable_str.trim().to_string();

                                    if !variable_string.is_empty() {
                                        if romanize {
                                            variable_string = romanize_string(variable_string)
                                        }

                                        let translated: Option<String> = get_variable_translated(
                                            &variable_string,
                                            variable_enum,
                                            &filename,
                                            &other_translation_map,
                                            game_type,
                                        );

                                        if let Some(text) = translated {
                                            obj[variable_name] = to_value(&text).unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    });
            } else {
                //Other files have the structure somewhat similar to Maps.json files
                obj_arr
                    .par_iter_mut()
                    .skip(1) //Skipping first element in array as it is null
                    .for_each(|obj: &mut Value| {
                        //CommonEvents doesn't have pages, so we can just check if it's Troops
                        let pages_length: usize = if filename.starts_with("Troops") {
                            obj["pages"].as_array().unwrap().len()
                        } else {
                            1
                        };

                        for i in 0..pages_length {
                            //If element has pages, then we'll iterate over them
                            //Otherwise we'll just iterate over the list
                            let list: &mut Value = if pages_length != 1 {
                                &mut obj["pages"][i]["list"]
                            } else {
                                &mut obj["list"]
                            };

                            if !list.is_array() {
                                continue;
                            }

                            let list_arr: &mut Array = list.as_array_mut().unwrap();
                            let list_len: usize = list_arr.len();

                            let mut in_sequence: bool = false;
                            let mut line: Vec<String> = Vec::with_capacity(4);
                            let mut item_indices: Vec<usize> = Vec::with_capacity(4);

                            for it in 0..list_len {
                                let code: u64 = list_arr[it]["code"].as_u64().unwrap();

                                if !ALLOWED_CODES.contains(&code) {
                                    if in_sequence {
                                        let mut joined: String = line.join("\n").trim().to_string();

                                        if romanize {
                                            joined = romanize_string(joined)
                                        }

                                        let translated: Option<String> = get_parameter_translated(
                                            Code::Dialogue,
                                            &joined,
                                            &other_translation_map,
                                            game_type,
                                        );

                                        if let Some(text) = translated {
                                            let split: Vec<&str> = text.split('\n').collect();
                                            let split_length: usize = split.len();
                                            let line_length: usize = line.len();

                                            for (i, &index) in item_indices.iter().enumerate() {
                                                if i < split_length {
                                                    list_arr[index]["parameters"][0] =
                                                        to_value(split[i]).unwrap();
                                                } else {
                                                    list_arr[index]["parameters"][0] =
                                                        to_value("").unwrap();
                                                }
                                            }

                                            if split_length > line_length {
                                                let remaining: String =
                                                    split[line_length..].join("\n");

                                                list_arr[*item_indices.last().unwrap()]
                                                    ["parameters"][0] =
                                                    to_value(&remaining).unwrap();
                                            }
                                        }

                                        line.clear();
                                        item_indices.clear();
                                        in_sequence = false
                                    }
                                    continue;
                                }

                                if [401, 405].contains(&code) {
                                    if let Some(parameter_str) =
                                        list_arr[it]["parameters"][0].as_str()
                                    {
                                        line.push(parameter_str.to_string());
                                        item_indices.push(it);
                                        in_sequence = true;
                                    }
                                } else if list_arr[it]["parameters"][0].is_array() {
                                    for i in
                                        0..list_arr[it]["parameters"][0].as_array().unwrap().len()
                                    {
                                        if let Some(subparameter_str) =
                                            list_arr[it]["parameters"][0][i].as_str()
                                        {
                                            let mut subparameter_string =
                                                subparameter_str.to_string();

                                            if romanize {
                                                subparameter_string =
                                                    romanize_string(subparameter_string);
                                            }

                                            let translated: Option<String> =
                                                get_parameter_translated(
                                                    Code::Dialogue,
                                                    &subparameter_string,
                                                    &other_translation_map,
                                                    game_type,
                                                );

                                            if let Some(translated) = translated {
                                                list_arr[it]["parameters"][0][i] =
                                                    to_value(&translated).unwrap();
                                            }
                                        }
                                    }
                                } else if let Some(parameter_str) =
                                    list_arr[it]["parameters"][0].as_str()
                                {
                                    let mut parameter_string: String = parameter_str.to_string();

                                    if romanize {
                                        parameter_string = romanize_string(parameter_string);
                                    }

                                    let translated: Option<String> = get_parameter_translated(
                                        Code::System,
                                        &parameter_string,
                                        &other_translation_map,
                                        game_type,
                                    );

                                    if let Some(translated) = translated {
                                        list_arr[it]["parameters"][0] =
                                            to_value(&translated).unwrap();
                                    }
                                } else if let Some(parameter_str) =
                                    list_arr[it]["parameters"][1].as_str()
                                {
                                    let mut parameter_string: String = parameter_str.to_string();

                                    if romanize {
                                        parameter_string = romanize_string(parameter_string);
                                    }

                                    let translated: Option<String> = get_parameter_translated(
                                        Code::Unknown,
                                        &parameter_string,
                                        &other_translation_map,
                                        game_type,
                                    );

                                    if let Some(translated) = translated {
                                        list_arr[it]["parameters"][1] =
                                            to_value(&translated).unwrap();
                                    }
                                }
                            }
                        }
                    });
            }

            write(output_path.join(&filename), to_string(&obj_arr).unwrap()).unwrap();

            if logging {
                println!("{file_written_msg} {filename}");
            }
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
) {
    let mut system_obj: Object = from_str(&read_to_string(system_file_path).unwrap()).unwrap();

    let system_original_text: Vec<String> = read_to_string(other_path.join("system.txt"))
        .unwrap()
        .par_split('\n')
        .map(|line: &str| line.trim().to_string())
        .collect();

    let system_translation_string: (String, String) =
        read_to_string(other_path.join("system_trans.txt"))
            .unwrap()
            .into_rsplit_once('\n')
            .unwrap();

    let game_title: String = system_translation_string.1;

    let mut system_translated_text: Vec<String> = system_translation_string
        .0
        .par_split('\n')
        .map(|line: &str| line.trim().to_string())
        .collect();

    match shuffle_level {
        1 => {
            shuffle(&mut system_translated_text);
        }
        2 => {
            for text_string in system_translated_text.iter_mut() {
                *text_string = shuffle_words(text_string);
            }
        }
        _ => {}
    }

    let system_translation_map: HashMap<String, String, BuildHasherDefault<Xxh3>> =
        system_original_text
            .into_par_iter()
            .zip(system_translated_text.into_par_iter())
            .fold(
                HashMap::default,
                |mut map: HashMap<String, String, BuildHasherDefault<Xxh3>>,
                 (key, value): (String, String)| {
                    map.insert(key, value);
                    map
                },
            )
            .reduce(HashMap::default, |mut a, b| {
                a.extend(b);
                a
            });

    system_obj["armorTypes"]
        .as_array_mut()
        .unwrap()
        .par_iter_mut()
        .for_each(|value: &mut Value| {
            let mut string: String = value.as_str().unwrap().trim().to_string();

            if romanize {
                string = romanize_string(string);
            }

            if let Some(text) = system_translation_map.get(&string) {
                *value = to_value(text).unwrap();
            }
        });

    system_obj["elements"]
        .as_array_mut()
        .unwrap()
        .par_iter_mut()
        .for_each(|value: &mut Value| {
            let mut string: String = value.as_str().unwrap().trim().to_string();

            if romanize {
                string = romanize_string(string);
            }

            if let Some(text) = system_translation_map.get(&string) {
                *value = to_value(text).unwrap();
            }
        });

    system_obj["equipTypes"]
        .as_array_mut()
        .unwrap()
        .par_iter_mut()
        .for_each(|value: &mut Value| {
            let mut string: String = value.as_str().unwrap().trim().to_string();

            if romanize {
                string = romanize_string(string);
            }

            if let Some(text) = system_translation_map.get(&string) {
                *value = to_value(text).unwrap();
            }
        });

    system_obj["skillTypes"]
        .as_array_mut()
        .unwrap()
        .par_iter_mut()
        .for_each(|value: &mut Value| {
            let mut string: String = value.as_str().unwrap().trim().to_string();

            if romanize {
                string = romanize_string(string);
            }

            if let Some(text) = system_translation_map.get(&string) {
                *value = to_value(text).unwrap();
            }
        });

    system_obj["terms"]
        .as_object_mut()
        .unwrap()
        .iter_mut()
        .par_bridge()
        .for_each(|(key, value): (&str, &mut Value)| {
            if key != "messages" {
                value
                    .as_array_mut()
                    .unwrap()
                    .par_iter_mut()
                    .for_each(|subvalue: &mut Value| {
                        if let Some(str) = subvalue.as_str() {
                            let mut string: String = str.trim().to_string();

                            if romanize {
                                string = romanize_string(string);
                            }

                            if let Some(text) = system_translation_map.get(&string) {
                                *subvalue = to_value(text).unwrap();
                            }
                        }
                    });
            } else {
                if !value.is_object() {
                    return;
                }

                value
                    .as_object_mut()
                    .unwrap()
                    .iter_mut()
                    .par_bridge()
                    .for_each(|(_, value)| {
                        let mut string: String = value.as_str().unwrap().trim().to_string();

                        if romanize {
                            string = romanize_string(string)
                        }

                        if let Some(text) = system_translation_map.get(&string) {
                            *value = to_value(text).unwrap();
                        }
                    });
            }
        });

    system_obj["weaponTypes"]
        .as_array_mut()
        .unwrap()
        .par_iter_mut()
        .for_each(|value: &mut Value| {
            let mut string: String = value.as_str().unwrap().trim().to_string();

            if romanize {
                string = romanize_string(string);
            }

            if let Some(text) = system_translation_map.get(&string) {
                *value = to_value(text).unwrap();
            }
        });

    system_obj["gameTitle"] = to_value(&game_title).unwrap();

    write(
        output_path.join("System.json"),
        to_string(&system_obj).unwrap(),
    )
    .unwrap();

    if logging {
        println!("{file_written_msg} System.json");
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
    pluigns_file_path: &Path,
    plugins_path: &Path,
    output_path: &Path,
    shuffle_level: u64,
    logging: bool,
    file_written_msg: &str,
) {
    let mut obj_arr: Vec<Object> = from_str(&read_to_string(pluigns_file_path).unwrap()).unwrap();

    let plugins_original_text_vec: Vec<String> = read_to_string(plugins_path.join("plugins.txt"))
        .unwrap()
        .par_split('\n')
        .map(|line: &str| line.to_string())
        .collect();

    let mut plugins_translated_text_vec: Vec<String> =
        read_to_string(plugins_path.join("plugins_trans.txt"))
            .unwrap()
            .par_split('\n')
            .map(|line: &str| line.to_string())
            .collect();

    match shuffle_level {
        1 => {
            shuffle(&mut plugins_translated_text_vec);
        }
        2 => {
            for text_string in plugins_translated_text_vec.iter_mut() {
                *text_string = shuffle_words(text_string);
            }
        }
        _ => {}
    }

    let plugins_translation_map: HashMap<String, String, BuildHasherDefault<Xxh3>> =
        plugins_original_text_vec
            .into_par_iter()
            .zip(plugins_translated_text_vec.into_par_iter())
            .fold(
                HashMap::default,
                |mut map: HashMap<String, String, BuildHasherDefault<Xxh3>>,
                 (key, value): (String, String)| {
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
            //YEP_OptionsCore should be processed differently, as its parameters is a mess, that can't even be parsed to json
            if name == "YEP_OptionsCore" {
                obj["parameters"]
                    .as_object_mut()
                    .unwrap()
                    .iter_mut()
                    .par_bridge()
                    .for_each(|(key, value): (&str, &mut Value)| {
                        let mut string: String = value.as_str().unwrap().to_string();

                        if key == "OptionsCategories" {
                            for (text, translated_text) in plugins_translation_map
                                .keys()
                                .zip(plugins_translation_map.values())
                            {
                                string = string.replacen(text, translated_text, 1);
                            }

                            *value = to_value(&string).unwrap();
                        } else if let Some(param) = plugins_translation_map.get(&string) {
                            *value = to_value(param).unwrap();
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
                    .for_each(|(_, string)| {
                        if let Some(str) = string.as_str() {
                            if let Some(param) = plugins_translation_map.get(str) {
                                *string = to_value(param).unwrap();
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
