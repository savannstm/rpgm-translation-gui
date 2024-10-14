#![allow(clippy::too_many_arguments)]
use crate::{
    extract_strings, get_object_data, romanize_string, Code, EngineType, GameType, MapsProcessingMode, Variable,
    ENDS_WITH_IF_RE, EXTENSION, LINES_SEPARATOR, LISA_PREFIX_RE, NEW_LINE,
};
use encoding_rs::Encoding;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use marshal_rs::{dump, load, StringMode};
use rayon::prelude::*;
use sonic_rs::{from_str, from_value, json, prelude::*, to_string, Array, Object, Value};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    ffi::OsString,
    fs::{read, read_dir, read_to_string, write, DirEntry},
    hash::BuildHasherDefault,
    io::{Read, Write},
    mem::take,
    path::Path,
    str::{from_utf8_unchecked, Chars},
    sync::{Arc, Mutex},
};
use xxhash_rust::xxh3::Xxh3;

type StringHashMap = HashMap<String, String, BuildHasherDefault<Xxh3>>;

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn get_translated_parameter<'a>(
    code: Code,
    mut parameter: &'a str,
    hashmap: &'a StringHashMap,
    game_type: Option<GameType>,
    engine_type: EngineType,
) -> Option<String> {
    let mut remaining_strings: Vec<String> = Vec::with_capacity(4);

    // bool indicates insert whether at start or at end
    // true inserts at end
    // false inserts at start
    let mut insert_positions: Vec<bool> = Vec::with_capacity(4);

    #[allow(unreachable_patterns)]
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
                        remaining_strings.push(re_match.as_str().to_owned());
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
            remaining_strings.push(re_match.as_str().to_owned());
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

        for (string, position) in remaining_strings.into_iter().zip(insert_positions) {
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
fn get_translated_parameter_deque(
    code: Code,
    mut parameter: &str,
    deque: &mut VecDeque<String>,
    game_type: Option<GameType>,
    engine_type: EngineType,
) -> Option<String> {
    let mut remaining_strings: Vec<String> = Vec::with_capacity(4);

    // bool indicates insert whether at start or at end
    // true inserts at end
    // false inserts at start
    let mut insert_positions: Vec<bool> = Vec::with_capacity(4);

    #[allow(unreachable_patterns)]
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
                        remaining_strings.push(re_match.as_str().to_owned());
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
            remaining_strings.push(re_match.as_str().to_owned());
            insert_positions.push(true);
        }
    }

    let translated: Option<String> = if code == Code::Choice {
        deque.front().map(String::to_owned)
    } else {
        deque.pop_front()
    };

    if let Some(mut translated) = translated {
        if translated.is_empty() {
            return None;
        }

        for (string, position) in remaining_strings.into_iter().zip(insert_positions) {
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
    hashmap: &StringHashMap,
    game_type: Option<GameType>,
    engine_type: EngineType,
) -> Option<String> {
    let mut remaining_strings: Vec<String> = Vec::with_capacity(4);

    // bool indicates insert whether at start or at end
    // true inserts at end
    // false inserts at start
    let mut insert_positions: Vec<bool> = Vec::with_capacity(4);

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

                        let mut note_chars: Chars = note.chars();
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
                                    note_string = String::from("\n") + left;
                                }
                            } else if note.ends_with(['.', '%', '!', '"']) {
                                note_string = note.to_owned();
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
                        let mut variable_text_chars: Chars = variable_text.chars();
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
                                return Some(right.to_owned());
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

        for (string, position) in remaining_strings.into_iter().zip(insert_positions) {
            match position {
                true => result.push_str(&string),
                false => result = string + &result,
            }
        }

        if matches!(
            variable_type,
            Variable::Message1 | Variable::Message2 | Variable::Message3 | Variable::Message4
        ) {
            result = String::from(" ") + &result;
        }

        #[allow(clippy::collapsible_if, clippy::collapsible_match)]
        if let Some(game_type) = game_type {
            match game_type {
                GameType::Termina => match variable_type {
                    Variable::Note => {
                        if let Some(first_char) = result.chars().next() {
                            if first_char != '\n' {
                                result = String::from("\n") + &result
                            }
                        }
                    }
                    _ => {}
                },
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
    allowed_codes: &[u16],
    romanize: bool,
    game_type: Option<GameType>,
    engine_type: EngineType,
    map: &StringHashMap,
    (code_label, parameters_label): (&str, &str),
) {
    let list_length: usize = list.len();

    let mut in_sequence: bool = false;
    let mut line: Vec<String> = Vec::with_capacity(8);
    let mut item_indices: Vec<usize> = Vec::with_capacity(8);

    for it in 0..list_length {
        let code: u16 = list[it][code_label].as_u64().unwrap() as u16;

        let write_string_literally: bool = if engine_type != EngineType::New {
            !match code {
                320 | 324 | 356 | 401 | 405 => list[it][parameters_label][0].is_object(),
                102 => list[it][parameters_label][0][0].is_object(),
                402 => list[it][parameters_label][1].is_object(),
                _ => false,
            }
        } else {
            true
        };

        if in_sequence && ![401, 405].contains(&code) {
            if !line.is_empty() {
                let mut joined: String = line.join("\n").trim().to_owned();

                if romanize {
                    joined = romanize_string(joined)
                }

                let translated: Option<String> =
                    get_translated_parameter(Code::Dialogue, &joined, map, game_type, engine_type);

                if let Some(translated) = translated {
                    let split_vec: Vec<&str> = translated.split('\n').collect();
                    let split_length: usize = split_vec.len();
                    let line_length: usize = line.len();

                    for (i, &index) in item_indices.iter().enumerate() {
                        if i < split_length {
                            list[index][parameters_label][0] = if !write_string_literally {
                                json!({
                                    "__type": "bytes",
                                    "data": Array::from(split_vec[i].as_bytes())
                                })
                            } else {
                                Value::from(split_vec[i])
                            };
                        } else {
                            list[index][parameters_label][0] = Value::from_static_str(" ");
                        }
                    }

                    if split_length > line_length {
                        let remaining: String = split_vec[line_length - 1..].join("\n");
                        list[*unsafe { item_indices.last().unwrap_unchecked() }][parameters_label][0] =
                            Value::from(&remaining);
                    }
                }

                line.clear();
                item_indices.clear();
            }

            in_sequence = false
        }

        if !allowed_codes.contains(&code) {
            continue;
        }

        match code {
            401 | 405 => {
                let parameter_string: String = list[it][parameters_label][0]
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or(match list[it][parameters_label][0].as_object() {
                        Some(obj) => get_object_data(obj),
                        None => String::new(),
                    })
                    .trim()
                    .to_owned();

                if !parameter_string.is_empty() {
                    line.push(parameter_string);
                    item_indices.push(it);
                    in_sequence = true;
                }
            }
            102 => {
                for i in 0..list[it][parameters_label][0].as_array().unwrap().len() {
                    let mut subparameter_string: String = list[it][parameters_label][0][i]
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or(match list[it][parameters_label][0][i].as_object() {
                            Some(obj) => get_object_data(obj),
                            None => String::new(),
                        })
                        .trim()
                        .to_owned();

                    if romanize {
                        subparameter_string = romanize_string(subparameter_string);
                    }

                    let translated: Option<String> =
                        get_translated_parameter(Code::Choice, &subparameter_string, map, game_type, engine_type);

                    if let Some(translated) = translated {
                        if engine_type == EngineType::New {
                            list[it][parameters_label][0][i] = Value::from(&translated);
                        } else {
                            list[it][parameters_label][0][i] =
                                json!({"__type": "bytes", "data": Array::from(translated.as_bytes())});
                        }
                    }
                }
            }
            356 => {
                let mut parameter_string: String = list[it][parameters_label][0]
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or(match list[it][parameters_label][0].as_object() {
                        Some(obj) => get_object_data(obj),
                        None => String::new(),
                    })
                    .trim()
                    .to_owned();

                if romanize {
                    parameter_string = romanize_string(parameter_string);
                }

                let translated: Option<String> =
                    get_translated_parameter(Code::System, &parameter_string, map, game_type, engine_type);

                if let Some(translated) = translated {
                    if engine_type == EngineType::New {
                        list[it][parameters_label][0] = Value::from(&translated);
                    } else {
                        list[it][parameters_label][0] =
                            json!({"__type": "bytes", "data": Array::from(translated.as_bytes())});
                    }
                }
            }
            320 | 324 | 402 => {
                let mut parameter_string: String = list[it][parameters_label][1]
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or(match list[it][parameters_label][1].as_object() {
                        Some(obj) => get_object_data(obj),
                        None => String::new(),
                    })
                    .trim()
                    .to_owned();

                if romanize {
                    parameter_string = romanize_string(parameter_string);
                }

                let translated: Option<String> =
                    get_translated_parameter(Code::Misc, &parameter_string, map, game_type, engine_type);

                if let Some(translated) = translated {
                    if engine_type == EngineType::New {
                        list[it][parameters_label][1] = Value::from(&translated);
                    } else {
                        list[it][parameters_label][1] =
                            json!({"__type": "bytes", "data": Array::from(translated.as_bytes())});
                    }
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
/// * `maps_processing_mode` - how to deal with lines duplicates in maps
/// * `romanize` - if files were read with romanize, this option will romanize original game text to compare with parsed
/// * `logging` - whether to log or not
/// * `game_type` - game type for custom parsing
/// * `engine_type` - engine type for right files processing
/// * `file_written_msg` - message to log when file is written
pub fn write_maps(
    maps_path: &Path,
    original_path: &Path,
    output_path: &Path,
    maps_processing_mode: MapsProcessingMode,
    romanize: bool,
    logging: bool,
    game_type: Option<GameType>,
    engine_type: EngineType,
    file_written_msg: &str,
) {
    if maps_processing_mode != MapsProcessingMode::Preserve {
        let maps_obj_iter =
            read_dir(original_path)
                .unwrap()
                .par_bridge()
                .filter_map(|entry: Result<DirEntry, std::io::Error>| {
                    if let Ok(entry) = entry {
                        let filename: OsString = entry.file_name();
                        let filename_str: &str = unsafe { from_utf8_unchecked(filename.as_encoded_bytes()) };

                        if filename_str.starts_with("Map")
                            && unsafe { (*filename_str.as_bytes().get_unchecked(3) as char).is_ascii_digit() }
                            && filename_str.ends_with(unsafe { EXTENSION })
                        {
                            let json: Value = if engine_type == EngineType::New {
                                from_str(&read_to_string(entry.path()).unwrap()).unwrap()
                            } else {
                                load(&read(entry.path()).unwrap(), None, Some("")).unwrap()
                            };

                            Some((filename_str.to_owned(), json))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

        let original_content: String = read_to_string(maps_path.join("maps.txt")).unwrap();

        let mut names_lines_map: StringHashMap = HashMap::default();

        let lines_maps_vec: Vec<StringHashMap> = {
            let mut vec: Vec<StringHashMap> = Vec::with_capacity(512);
            let mut hashmap: StringHashMap = HashMap::default();

            for line in original_content.split('\n') {
                if line.starts_with("<!-- Map") {
                    let (original, translated) = line.split_once(LINES_SEPARATOR).unwrap();

                    if original.len() > 20 {
                        let map_name: &str = &original[17..original.len() - 4];
                        names_lines_map.insert(map_name.trim().to_owned(), translated.trim().to_owned());
                    }

                    if maps_processing_mode == MapsProcessingMode::Separate {
                        vec.push(take(&mut hashmap));
                    }
                } else {
                    if line.starts_with("<!--") {
                        continue;
                    }

                    let (original, translated) = line.split_once(LINES_SEPARATOR).unwrap();

                    let processed_original: String = original.replace(NEW_LINE, "\n").trim().to_owned().to_owned();
                    let processed_translated: String = translated.replace(NEW_LINE, "\n").trim().to_owned().to_owned();

                    hashmap.insert(processed_original, processed_translated);
                }
            }

            if vec.is_empty() {
                vec.push(hashmap);
            }

            vec
        };

        // 401 - dialogue lines
        // 102 - dialogue choices array
        // 402 - one of the dialogue choices from the array
        // 356 - system lines (special texts)
        // 324, 320 - i don't know what is it but it's some used in-game lines
        const ALLOWED_CODES: [u16; 6] = [102, 320, 324, 356, 401, 402];

        let (display_name_label, events_label, pages_label, list_label, code_label, parameters_label) =
            if engine_type == EngineType::New {
                ("displayName", "events", "pages", "list", "code", "parameters")
            } else {
                (
                    "__symbol__display_name",
                    "__symbol__events",
                    "__symbol__pages",
                    "__symbol__list",
                    "__symbol__code",
                    "__symbol__parameters",
                )
            };

        let idx: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
        maps_obj_iter.for_each_with(idx, |idx: &mut Arc<Mutex<usize>>, (filename, mut obj)| {
            let hashmap: &StringHashMap = lines_maps_vec
                .get(*idx.lock().unwrap())
                .unwrap_or(unsafe { lines_maps_vec.get_unchecked(0) });

            *idx.lock().unwrap() += 1;

            if let Some(display_name) = obj[display_name_label].as_str() {
                let mut display_name: String = display_name.to_owned();

                if romanize {
                    display_name = romanize_string(display_name)
                }

                if let Some(location_name) = names_lines_map.get(&display_name) {
                    obj[display_name_label] = Value::from(location_name);
                }
            }

            // Skipping first element in array as it is null
            let mut events_arr: Vec<&mut Value> = if engine_type == EngineType::New {
                obj[events_label]
                    .as_array_mut()
                    .unwrap()
                    .par_iter_mut()
                    .skip(1)
                    .collect()
            } else {
                obj[events_label]
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

                event[pages_label]
                    .as_array_mut()
                    .unwrap()
                    .par_iter_mut()
                    .for_each(|page: &mut Value| {
                        write_list(
                            page[list_label].as_array_mut().unwrap(),
                            &ALLOWED_CODES,
                            romanize,
                            game_type,
                            engine_type,
                            hashmap,
                            (code_label, parameters_label),
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
    } else {
        let maps_obj_iter = read_dir(original_path)
            .unwrap()
            .filter_map(|entry: Result<DirEntry, std::io::Error>| {
                if let Ok(entry) = entry {
                    let filename: OsString = entry.file_name();
                    let filename_str: &str = unsafe { from_utf8_unchecked(filename.as_encoded_bytes()) };

                    if filename_str.starts_with("Map")
                        && unsafe { (*filename_str.as_bytes().get_unchecked(3) as char).is_ascii_digit() }
                        && filename_str.ends_with(unsafe { EXTENSION })
                    {
                        let json: Value = if engine_type == EngineType::New {
                            from_str(&read_to_string(entry.path()).unwrap()).unwrap()
                        } else {
                            load(&read(entry.path()).unwrap(), None, Some("")).unwrap()
                        };

                        Some((filename_str.to_owned(), json))
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

        let original_content: String = read_to_string(maps_path.join("maps.txt")).unwrap();

        let mut names_lines_map: StringHashMap = HashMap::default();

        let mut lines_deque: VecDeque<String> = {
            let mut deque: VecDeque<String> = VecDeque::new();

            for line in original_content.split('\n') {
                if line.starts_with("<!-- Map") {
                    let (original, translated) = line.split_once(LINES_SEPARATOR).unwrap();

                    if original.len() > 20 {
                        let map_name: &str = &original[17..original.len() - 4];
                        names_lines_map.insert(map_name.trim().to_owned(), translated.trim().to_owned());
                    }
                } else {
                    if line.starts_with("<!--") {
                        continue;
                    }

                    let (_, translated) = line.split_once(LINES_SEPARATOR).unwrap();

                    let processed_translated: String = translated.replace(NEW_LINE, "\n").trim().to_owned().to_owned();

                    deque.push_back(processed_translated);
                }
            }

            deque
        };

        // 401 - dialogue lines
        // 102 - dialogue choices array
        // 402 - one of the dialogue choices from the array
        // 356 - system lines (special texts)
        // 324, 320 - i don't know what is it but it's some used in-game lines
        const ALLOWED_CODES: [u16; 6] = [102, 320, 324, 356, 401, 402];

        let (display_name_label, events_label, pages_label, list_label, code_label, parameters_label) =
            if engine_type == EngineType::New {
                ("displayName", "events", "pages", "list", "code", "parameters")
            } else {
                (
                    "__symbol__display_name",
                    "__symbol__events",
                    "__symbol__pages",
                    "__symbol__list",
                    "__symbol__code",
                    "__symbol__parameters",
                )
            };

        maps_obj_iter.for_each(|(filename, mut obj)| {
            if let Some(display_name) = obj[display_name_label].as_str() {
                let mut display_name: String = display_name.to_owned();

                if romanize {
                    display_name = romanize_string(display_name)
                }

                if let Some(location_name) = names_lines_map.get(&display_name) {
                    obj[display_name_label] = Value::from(location_name);
                }
            }

            // Skipping first element in array as it is null
            let mut events_arr: Vec<&mut Value> = if engine_type == EngineType::New {
                obj[events_label].as_array_mut().unwrap().iter_mut().skip(1).collect()
            } else {
                obj[events_label]
                    .as_object_mut()
                    .unwrap()
                    .iter_mut()
                    .map(|(_, value)| value)
                    .collect()
            };

            events_arr.iter_mut().for_each(|event: &mut &mut Value| {
                if event.is_null() {
                    return;
                }

                event[pages_label]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .for_each(|page: &mut Value| {

                            let list: &mut Array = page[list_label].as_array_mut().unwrap();
                            let allowed_codes: &[u16] = &ALLOWED_CODES;
                            let (code_label, parameters_label) = (code_label, parameters_label);
                            let list_length: usize = list.len();

                            let mut in_sequence: bool = false;
                            let mut line: Vec<String> = Vec::with_capacity(8);
                            let mut item_indices: Vec<usize> = Vec::with_capacity(8);

                            for it in 0..list_length {
                                let code: u16 = list[it][code_label].as_u64().unwrap() as u16;

                                let write_string_literally: bool = if engine_type != EngineType::New {
                                    !match code {
                                        320 | 324 | 356 | 401 | 405 => list[it][parameters_label][0].is_object(),
                                        102 => list[it][parameters_label][0][0].is_object(),
                                        402 => list[it][parameters_label][1].is_object(),
                                        _ => false,
                                    }
                                } else {
                                    true
                                };

                                if in_sequence && ![401, 405].contains(&code) {
                                    if !line.is_empty() {
                                        let mut joined: String = line.join("\n").trim().to_owned();

                                        if romanize {
                                            joined = romanize_string(joined)
                                        }

                                        let translated: Option<String> =
                                            get_translated_parameter_deque(Code::Dialogue, &joined, &mut lines_deque, game_type, engine_type);

                                        if let Some(translated) = translated {
                                            let split_vec: Vec<&str> = translated.split('\n').collect();
                                            let split_length: usize = split_vec.len();
                                            let line_length: usize = line.len();

                                            for (i, &index) in item_indices.iter().enumerate() {
                                                if i < split_length {
                                                    list[index][parameters_label][0] = if !write_string_literally {
                                                        json!({
                                                            "__type": "bytes",
                                                            "data": Array::from(split_vec[i].as_bytes())
                                                        })
                                                    } else {
                                                        Value::from(split_vec[i])
                                                    };
                                                } else {
                                                    list[index][parameters_label][0] = Value::from_static_str(" ");
                                                }
                                            }

                                            if split_length > line_length {
                                                let remaining: String = split_vec[line_length - 1..].join("\n");
                                                list[*unsafe { item_indices.last().unwrap_unchecked() }][parameters_label][0] =
                                                    Value::from(&remaining);
                                            }
                                        }

                                        line.clear();
                                        item_indices.clear();
                                    }

                                    in_sequence = false
                                }

                                if !allowed_codes.contains(&code) {
                                    continue;
                                }

                                match code {
                                    401 | 405 => {
                                        let parameter_string: String = list[it][parameters_label][0]
                                            .as_str()
                                            .map(str::to_owned)
                                            .unwrap_or(match list[it][parameters_label][0].as_object() {
                                                Some(obj) => get_object_data(obj),
                                                None => String::new(),
                                            })
                                            .trim()
                                            .to_owned();

                                        if !parameter_string.is_empty() {
                                            line.push(parameter_string);
                                            item_indices.push(it);
                                            in_sequence = true;
                                        }
                                    }
                                    102 => {
                                        for i in 0..list[it][parameters_label][0].as_array().unwrap().len() {
                                            let mut subparameter_string: String = list[it][parameters_label][0][i]
                                                .as_str()
                                                .map(str::to_owned)
                                                .unwrap_or(match list[it][parameters_label][0][i].as_object() {
                                                    Some(obj) => get_object_data(obj),
                                                    None => String::new(),
                                                })
                                                .trim()
                                                .to_owned();

                                            if romanize {
                                                subparameter_string = romanize_string(subparameter_string);
                                            }

                                            let translated: Option<String> =
                                                get_translated_parameter_deque(Code::Choice, &subparameter_string, &mut lines_deque, game_type, engine_type);

                                            if let Some(translated) = translated {
                                                if engine_type == EngineType::New {
                                                    list[it][parameters_label][0][i] = Value::from(&translated);
                                                } else {
                                                    list[it][parameters_label][0][i] =
                                                        json!({"__type": "bytes", "data": Array::from(translated.as_bytes())});
                                                }
                                            }
                                        }
                                    }
                                    356 => {
                                        let mut parameter_string: String = list[it][parameters_label][0]
                                            .as_str()
                                            .map(str::to_owned)
                                            .unwrap_or(match list[it][parameters_label][0].as_object() {
                                                Some(obj) => get_object_data(obj),
                                                None => String::new(),
                                            })
                                            .trim()
                                            .to_owned();

                                        if romanize {
                                            parameter_string = romanize_string(parameter_string);
                                        }

                                        let translated: Option<String> =
                                            get_translated_parameter_deque(Code::System, &parameter_string, &mut lines_deque, game_type, engine_type);

                                        if let Some(translated) = translated {
                                            if engine_type == EngineType::New {
                                                list[it][parameters_label][0] = Value::from(&translated);
                                            } else {
                                                list[it][parameters_label][0] =
                                                    json!({"__type": "bytes", "data": Array::from(translated.as_bytes())});
                                            }
                                        }
                                    }
                                    320 | 324 | 402 => {
                                        let mut parameter_string: String = list[it][parameters_label][1]
                                            .as_str()
                                            .map(str::to_owned)
                                            .unwrap_or(match list[it][parameters_label][1].as_object() {
                                                Some(obj) => get_object_data(obj),
                                                None => String::new(),
                                            })
                                            .trim()
                                            .to_owned();

                                        if romanize {
                                            parameter_string = romanize_string(parameter_string);
                                        }

                                        let translated: Option<String> =
                                            get_translated_parameter_deque(Code::Misc, &parameter_string, &mut lines_deque, game_type, engine_type);

                                        if let Some(translated) = translated {
                                            if engine_type == EngineType::New {
                                                list[it][parameters_label][1] = Value::from(&translated);
                                            } else {
                                                list[it][parameters_label][1] =
                                                    json!({"__type": "bytes", "data": Array::from(translated.as_bytes())});
                                            }
                                        }
                                    }
                                    _ => unreachable!(),
                                }
                            }
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
}

/// Writes .txt files from other folder back to their initial form.
/// # Parameters
/// * `other_path` - path to the other directory
/// * `original_path` - path to the original directory
/// * `output_path` - path to the output directory
/// * `romanize` - if files were read with romanize, this option will romanize original game text to compare with parsed
/// * `logging` - whether to log or not
/// * `game_type` - game type for custom parsing
/// * `engine_type` - engine type for right files processing
/// * `file_written_msg` - message to log when file is written
pub fn write_other(
    other_path: &Path,
    original_path: &Path,
    output_path: &Path,
    romanize: bool,
    logging: bool,
    game_type: Option<GameType>,
    engine_type: EngineType,
    file_written_msg: &str,
) {
    let other_obj_arr_vec =
        read_dir(original_path)
            .unwrap()
            .par_bridge()
            .filter_map(|entry: Result<DirEntry, std::io::Error>| {
                if let Ok(entry) = entry {
                    let filename: OsString = entry.file_name();
                    let filename_str: &str = unsafe { from_utf8_unchecked(filename.as_encoded_bytes()) };
                    let (real_name, _) = filename_str.split_once('.').unwrap();

                    if !real_name.starts_with("Map")
                        && !matches!(real_name, "Tilesets" | "Animations" | "System" | "Scripts")
                        && filename_str.ends_with(unsafe { EXTENSION })
                    {
                        if game_type.is_some_and(|game_type: GameType| game_type == GameType::Termina)
                            && real_name == "States"
                        {
                            return None;
                        }

                        let json: Value = if engine_type == EngineType::New {
                            from_str(&read_to_string(entry.path()).unwrap()).unwrap()
                        } else {
                            load(&read(entry.path()).unwrap(), None, Some("")).unwrap()
                        };

                        Some((filename_str.to_owned(), json))
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
    // 324, 320 - i don't know what is it but it's some used in-game lines
    const ALLOWED_CODES: [u16; 7] = [102, 320, 324, 356, 401, 402, 405];

    let variable_tuples: Arc<[(&str, Variable); 8]> = Arc::new(if engine_type == EngineType::New {
        [
            ("name", Variable::Name),
            ("nickname", Variable::Nickname),
            ("description", Variable::Description),
            ("message1", Variable::Message1),
            ("message2", Variable::Message2),
            ("message3", Variable::Message3),
            ("message4", Variable::Message4),
            ("note", Variable::Note),
        ]
    } else {
        [
            ("__symbol__name", Variable::Name),
            ("__symbol__nickname", Variable::Nickname),
            ("__symbol__description", Variable::Description),
            ("__symbol__message1", Variable::Message1),
            ("__symbol__message2", Variable::Message2),
            ("__symbol__message3", Variable::Message3),
            ("__symbol__message4", Variable::Message4),
            ("__symbol__note", Variable::Note),
        ]
    });

    let (pages_label, list_label, code_label, parameters_label) = if engine_type == EngineType::New {
        ("pages", "list", "code", "parameters")
    } else {
        (
            "__symbol__pages",
            "__symbol__list",
            "__symbol__code",
            "__symbol__parameters",
        )
    };

    other_obj_arr_vec.into_par_iter().for_each(|(filename, mut obj_arr)| {
        let content_path: &Path =
            &other_path.join(filename[..filename.len() - unsafe { EXTENSION.len() }].to_owned() + ".txt");

        let original_content: String = read_to_string(content_path).unwrap();

        let lines_map: StringHashMap =
            HashMap::from_par_iter(original_content.par_split('\n').filter_map(|line: &str| {
                if line.starts_with("<!--") {
                    None
                } else {
                    let (original, translated) = line.split_once(LINES_SEPARATOR).unwrap();
                    Some((
                        original.replace(r"\#", "\n").trim().to_owned(),
                        translated.replace(r"\#", "\n").trim().to_owned(),
                    ))
                }
            }));

        // Other files except CommonEvents and Troops have the structure that consists
        // of name, nickname, description and note
        if !filename.starts_with("Co") && !filename.starts_with("Tr") {
            obj_arr
                .as_array_mut()
                .unwrap()
                .par_iter_mut()
                .skip(1) // Skipping first element in array as it is null
                .for_each(|obj: &mut Value| {
                    for (variable_label, variable_type) in variable_tuples.into_iter() {
                        if let Some(variable_str) = obj[variable_label].as_str() {
                            let mut variable_string: String = if variable_type != Variable::Note {
                                variable_str.trim().to_owned()
                            } else {
                                variable_str.to_owned()
                            };

                            if !variable_string.is_empty() {
                                if romanize {
                                    variable_string = romanize_string(variable_string)
                                }

                                variable_string = variable_string
                                    .split('\n')
                                    .map(str::trim)
                                    .collect::<Vec<_>>()
                                    .join("\n");

                                let note_text: Option<&str> = if game_type
                                    .is_some_and(|game_type: GameType| game_type != GameType::Termina)
                                    && variable_type != Variable::Description
                                {
                                    None
                                } else {
                                    match obj.get(unsafe { variable_tuples.last().unwrap_unchecked() }.0) {
                                        Some(value) => value.as_str(),
                                        None => None,
                                    }
                                };

                                let translated: Option<String> = get_translated_variable(
                                    variable_string,
                                    note_text,
                                    variable_type,
                                    &filename,
                                    &lines_map,
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
            // Other files have the structure somewhat similar to Maps files
            obj_arr
                .as_array_mut()
                .unwrap()
                .par_iter_mut()
                .skip(1) // Skipping first element in array as it is null
                .for_each(|obj: &mut Value| {
                    // CommonEvents doesn't have pages, so we can just check if it's Troops
                    let pages_length: usize = if filename.starts_with("Troops") {
                        obj[pages_label].as_array().unwrap().len()
                    } else {
                        1
                    };

                    for i in 0..pages_length {
                        // If element has pages, then we'll iterate over them
                        // Otherwise we'll just iterate over the list
                        let list_value: &mut Value = if pages_length != 1 {
                            &mut obj[pages_label][i][list_label]
                        } else {
                            &mut obj[list_label]
                        };

                        if let Some(list) = list_value.as_array_mut() {
                            write_list(
                                list,
                                &ALLOWED_CODES,
                                romanize,
                                game_type,
                                engine_type,
                                &lines_map,
                                (code_label, parameters_label),
                            );
                        }
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
/// * `logging` - whether to log or not
/// * `engine_type` - engine type for right files processing
/// * `file_written_msg` - message to log when file is written
pub fn write_system(
    system_file_path: &Path,
    other_path: &Path,
    output_path: &Path,
    romanize: bool,
    logging: bool,
    engine_type: EngineType,
    file_written_msg: &str,
) {
    let mut obj: Value = if engine_type == EngineType::New {
        from_str(&read_to_string(system_file_path).unwrap()).unwrap()
    } else {
        load(&read(system_file_path).unwrap(), None, Some("")).unwrap()
    };

    let original_content: String = read_to_string(other_path.join("system.txt")).unwrap();

    let lines_map: StringHashMap = HashMap::from_par_iter(original_content.par_split('\n').filter_map(|line: &str| {
        if line.starts_with("<!--") {
            None
        } else {
            let (original, translated) = line.split_once(LINES_SEPARATOR).unwrap();
            Some((original.trim().to_owned(), translated.trim().to_owned()))
        }
    }));
    let game_title: String = original_content.rsplit_once(LINES_SEPARATOR).unwrap().1.to_owned();

    let (armor_types_label, elements_label, skill_types_label, terms_label, weapon_types_label, game_title_label) =
        if engine_type == EngineType::New {
            (
                "armorTypes",
                "elements",
                "skillTypes",
                "terms",
                "weaponTypes",
                "gameTitle",
            )
        } else {
            (
                "__symbol__armor_types",
                "__symbol__elements",
                "__symbol__skill_types",
                if engine_type == EngineType::XP {
                    "__symbol__words"
                } else {
                    "__symbol__terms"
                },
                "__symbol__weapon_types",
                "__symbol__game_title",
            )
        };

    if engine_type != EngineType::New {
        let mut string: String = obj["__symbol__currency_unit"].as_str().unwrap().trim().to_owned();

        if romanize {
            string = romanize_string(string);
        }

        if let Some(translated) = lines_map.get(&string) {
            if !translated.is_empty() {
                obj["__symbol__currency_unit"] = Value::from(translated);
            }
        }
    }

    obj[armor_types_label]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .for_each(|value: &mut Value| {
            let mut string: String = value.as_str().unwrap().trim().to_owned();

            if romanize {
                string = romanize_string(string);
            }

            if let Some(translated) = lines_map.get(&string) {
                if translated.is_empty() {
                    return;
                }

                *value = Value::from(translated);
            }
        });

    obj[elements_label]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .for_each(|value: &mut Value| {
            let mut string: String = value.as_str().unwrap().trim().to_owned();

            if romanize {
                string = romanize_string(string);
            }

            if let Some(translated) = lines_map.get(&string) {
                if translated.is_empty() {
                    return;
                }

                *value = Value::from(translated);
            }
        });

    if engine_type == EngineType::New {
        obj["equipTypes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .for_each(|value: &mut Value| {
                let mut string: String = value.as_str().unwrap().trim().to_owned();

                if romanize {
                    string = romanize_string(string);
                }

                if let Some(translated) = lines_map.get(&string) {
                    if translated.is_empty() {
                        return;
                    }

                    *value = Value::from(translated);
                }
            });
    }

    obj[skill_types_label]
        .as_array_mut()
        .unwrap()
        .par_iter_mut()
        .for_each(|value: &mut Value| {
            let mut string: String = value.as_str().unwrap().trim().to_owned();

            if romanize {
                string = romanize_string(string);
            }

            if let Some(translated) = lines_map.get(&string) {
                if translated.is_empty() {
                    return;
                }

                *value = Value::from(translated);
            }
        });

    obj[terms_label]
        .as_object_mut()
        .unwrap()
        .iter_mut()
        .for_each(|(key, value): (&str, &mut Value)| {
            if engine_type != EngineType::New && !key.starts_with("__symbol__") {
                return;
            }

            if key != "messages" {
                value
                    .as_array_mut()
                    .unwrap()
                    .par_iter_mut()
                    .for_each(|subvalue: &mut Value| {
                        if let Some(str) = subvalue.as_str() {
                            let mut string: String = str.trim().to_owned();

                            if romanize {
                                string = romanize_string(string);
                            }

                            if let Some(translated) = lines_map.get(&string) {
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
                    let mut string: String = value.as_str().unwrap().trim().to_owned();

                    if romanize {
                        string = romanize_string(string)
                    }

                    if let Some(translated) = lines_map.get(&string) {
                        if translated.is_empty() {
                            return;
                        }

                        *value = Value::from(translated);
                    }
                });
            }
        });

    obj[weapon_types_label]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .for_each(|value: &mut Value| {
            let mut string: String = value.as_str().unwrap().trim().to_owned();

            if romanize {
                string = romanize_string(string);
            }

            if let Some(translated) = lines_map.get(&string) {
                if translated.is_empty() {
                    return;
                }

                *value = Value::from(translated);
            }
        });

    obj[game_title_label] = Value::from(&game_title);

    let output_data: Vec<u8> = if engine_type == EngineType::New {
        to_string(&obj).unwrap().into_bytes()
    } else {
        dump(obj, Some(""))
    };

    if logging {
        println!("{file_written_msg} {}", system_file_path.display());
    }

    write(output_path.join(system_file_path.file_name().unwrap()), output_data).unwrap();
}

/// Writes plugins.txt file back to its initial form. Currently works only if game_type is GameType::Termina.
/// # Parameters
/// * `plugins_file_path` - path to the original plugins file
/// * `plugins_path` - path to the plugins directory
/// * `output_path` - path to the output directory
/// * `logging` - whether to log or not
/// * `file_written_msg` - message to log when file is written
pub fn write_plugins(
    pluigns_file_path: &Path,
    plugins_path: &Path,
    output_path: &Path,
    logging: bool,
    file_written_msg: &str,
) {
    let mut obj_arr: Vec<Object> = from_str(&read_to_string(pluigns_file_path).unwrap()).unwrap();

    let original_content: String = read_to_string(plugins_path.join("plugins.txt")).unwrap();

    let lines_map: StringHashMap = HashMap::from_iter(original_content.split('\n').filter_map(|line: &str| {
        if line.starts_with("<!--") {
            None
        } else {
            let (original, translated) = line.split_once(LINES_SEPARATOR).unwrap();
            Some((original.trim().to_owned(), translated.trim().to_owned()))
        }
    }));

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
                        let mut string: String = value.as_str().unwrap().to_owned();

                        if key == "OptionsCategories" {
                            for (text, translated) in lines_map.keys().zip(lines_map.values()) {
                                string = string.replacen(text, translated, 1);
                            }

                            *value = Value::from(string.as_str());
                        } else if let Some(translated) = lines_map.get(&string) {
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
                            if let Some(translated) = lines_map.get(str) {
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

pub fn write_scripts(
    scripts_file_path: &Path,
    other_path: &Path,
    output_path: &Path,
    romanize: bool,
    logging: bool,
    file_written_msg: &str,
) {
    let mut script_entries: Value = load(&read(scripts_file_path).unwrap(), Some(StringMode::Binary), None).unwrap();

    let original_content: String = read_to_string(other_path.join("scripts.txt")).unwrap();

    let lines_map: StringHashMap = {
        let mut hashmap: StringHashMap = HashMap::default();

        for line in original_content.split('\n') {
            if line.starts_with("<!--") {
                continue;
            }

            let (original, translated) = line.split_once(LINES_SEPARATOR).unwrap();
            hashmap.insert(original.trim().to_owned(), translated.trim().to_owned());
        }

        hashmap
    };

    let encodings: [&Encoding; 5] = [
        encoding_rs::UTF_8,
        encoding_rs::WINDOWS_1252,
        encoding_rs::WINDOWS_1251,
        encoding_rs::SHIFT_JIS,
        encoding_rs::GB18030,
    ];

    script_entries
        .as_array_mut()
        .unwrap()
        .par_iter_mut()
        .for_each(|script: &mut Value| {
            let data: Vec<u8> = from_value(&script.as_array().unwrap()[2]["data"]).unwrap();

            let mut inflated: Vec<u8> = Vec::new();
            ZlibDecoder::new(&*data).read_to_end(&mut inflated).unwrap();

            let mut code: String = String::with_capacity(4);

            for encoding in encodings {
                let (cow, _, had_errors) = encoding.decode(&inflated);

                if !had_errors {
                    code = cow.into_owned();
                    break;
                }
            }

            let (strings_array, indices_array) = extract_strings(&code, true);

            for (mut string, index) in strings_array.into_iter().zip(indices_array).rev() {
                if string.is_empty() || !lines_map.contains_key(&string) {
                    continue;
                }

                if romanize {
                    string = romanize_string(string);
                }

                let translated: Option<&String> = lines_map.get(&string);

                if let Some(translated) = translated {
                    code.replace_range(index..index + string.len(), translated);
                }
            }

            let mut buf: Vec<u8> = Vec::new();

            ZlibEncoder::new(&mut buf, Compression::new(6))
                .write_all(code.as_bytes())
                .unwrap();

            let data: Array = Array::from(buf);

            if let Some(arr) = script[2].as_array_mut() {
                arr[2]["data"] = data.into()
            };
        });

    if logging {
        println!("{file_written_msg} {}", scripts_file_path.display());
    }

    write(
        output_path.join(String::from("Scripts") + unsafe { EXTENSION }),
        dump(script_entries, None),
    )
    .unwrap();
}
