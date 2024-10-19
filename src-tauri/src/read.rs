#![allow(clippy::too_many_arguments)]
use crate::{
    extract_strings, get_object_data, romanize_string, Code, EngineType, GameType, MapsProcessingMode, ProcessingMode,
    Variable, ENDS_WITH_IF_RE, EXTENSION, INVALID_MULTILINE_VARIABLE_RE, INVALID_VARIABLE_RE, LINES_SEPARATOR,
    LISA_PREFIX_RE, NEW_LINE, STRING_IS_ONLY_SYMBOLS_RE,
};
use encoding_rs::Encoding;
use flate2::read::ZlibDecoder;
use indexmap::{IndexMap, IndexSet};
use marshal_rs::{load, StringMode};
use rayon::prelude::*;
use regex::Regex;
use sonic_rs::{from_str, from_value, prelude::*, Array, Value};
use std::{
    cell::UnsafeCell,
    collections::VecDeque,
    ffi::OsString,
    fs::{read, read_dir, read_to_string, write, DirEntry},
    hash::BuildHasherDefault,
    io::Read,
    path::Path,
    str::{from_utf8_unchecked, Chars},
};
use xxhash_rust::xxh3::Xxh3;

type Xxh3IndexSet = IndexSet<String, BuildHasherDefault<Xxh3>>;
type Xxh3IndexMap<'a, 'b> = IndexMap<&'a str, &'b str, BuildHasherDefault<Xxh3>>;

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn parse_parameter(
    code: Code,
    mut parameter: &str,
    game_type: Option<GameType>,
    engine_type: EngineType,
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

    Some(parameter.to_owned())
}

#[allow(clippy::single_match, clippy::match_single_binding, unused_mut)]
fn parse_variable(
    mut variable_text: String,
    variable_type: &Variable,
    filename: &str,
    game_type: Option<GameType>,
    engine_type: EngineType,
) -> Option<(String, bool)> {
    if STRING_IS_ONLY_SYMBOLS_RE.is_match(&variable_text) {
        return None;
    }

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

    let mut is_continuation_of_description: bool = false;

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
                            let mut variable_text_chars: Chars = variable_text.chars();

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

                                    variable_text = NEW_LINE.to_owned() + left;
                                } else {
                                    if !variable_text.ends_with(['.', '%', '!', '"']) {
                                        return None;
                                    }

                                    variable_text = NEW_LINE.to_owned() + &variable_text
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

fn parse_list<'a>(
    list: &Array,
    allowed_codes: &[u16],
    romanize: bool,
    game_type: Option<GameType>,
    engine_type: EngineType,
    processing_mode: ProcessingMode,
    (code_label, parameters_label): (&str, &str),
    set: &'a UnsafeCell<Xxh3IndexSet>,
    map: &'a mut Xxh3IndexMap,
) {
    let mut in_sequence: bool = false;
    let mut line: Vec<String> = Vec::with_capacity(8);

    let set_mut_ref: &mut Xxh3IndexSet = unsafe { &mut *set.get() };
    let set_ref: &Xxh3IndexSet = unsafe { &*set.get() };

    for item in list {
        let code: u16 = item[code_label].as_u64().unwrap() as u16;

        if in_sequence && ![401, 405].contains(&code) {
            if !line.is_empty() {
                let mut joined: String = line.join("\n").trim().replace('\n', NEW_LINE);

                if romanize {
                    joined = romanize_string(joined);
                }

                let parsed: Option<String> = parse_parameter(Code::Dialogue, &joined, game_type, engine_type);

                if let Some(parsed) = parsed {
                    set_mut_ref.insert(parsed);
                    let string_ref: &str = unsafe { set_ref.last().unwrap_unchecked() }.as_str();

                    if processing_mode == ProcessingMode::Append && !map.contains_key(string_ref) {
                        map.shift_insert(set_ref.len() - 1, string_ref, "");
                    }
                }

                line.clear();
            }

            in_sequence = false;
        }

        if !allowed_codes.contains(&code) {
            continue;
        }

        let parameters: &Array = item[parameters_label].as_array().unwrap();

        match code {
            401 | 405 => {
                let parameter_string: String = parameters[0]
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or(match parameters[0].as_object() {
                        Some(obj) => get_object_data(obj),
                        None => String::new(),
                    })
                    .trim()
                    .to_owned();

                if !parameter_string.is_empty() {
                    in_sequence = true;
                    line.push(parameter_string);
                }
            }
            102 => {
                for i in 0..parameters[0].as_array().unwrap().len() {
                    let subparameter_string: String = parameters[0][i]
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or(match parameters[0][i].as_object() {
                            Some(obj) => get_object_data(obj),
                            None => String::new(),
                        })
                        .trim()
                        .to_owned();

                    if !subparameter_string.is_empty() {
                        let parsed: Option<String> =
                            parse_parameter(Code::Choice, &subparameter_string, game_type, engine_type);

                        if let Some(mut parsed) = parsed {
                            if romanize {
                                parsed = romanize_string(parsed);
                            }

                            set_mut_ref.insert(parsed);
                            let string_ref: &str = unsafe { set_ref.last().unwrap_unchecked() }.as_str();

                            if processing_mode == ProcessingMode::Append && !map.contains_key(string_ref) {
                                map.shift_insert(set_ref.len() - 1, string_ref, "");
                            }
                        }
                    }
                }
            }
            356 => {
                let parameter_string: String = parameters[0]
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or(match parameters[0].as_object() {
                        Some(obj) => get_object_data(obj),
                        None => String::new(),
                    })
                    .trim()
                    .to_owned();

                if !parameter_string.is_empty() {
                    let parsed: Option<String> =
                        parse_parameter(Code::System, &parameter_string, game_type, engine_type);

                    if let Some(mut parsed) = parsed {
                        if romanize {
                            parsed = romanize_string(parsed);
                        }

                        set_mut_ref.insert(parsed);
                        let string_ref: &str = unsafe { set_ref.last().unwrap_unchecked() }.as_str();

                        if processing_mode == ProcessingMode::Append && !map.contains_key(string_ref) {
                            map.shift_insert(set_ref.len() - 1, string_ref, "");
                        }
                    }
                }
            }
            320 | 324 => {
                let parameter_string: String = parameters[1]
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or(match parameters[1].as_object() {
                        Some(obj) => get_object_data(obj),
                        None => String::new(),
                    })
                    .trim()
                    .to_owned();

                if !parameter_string.is_empty() {
                    let parsed: Option<String> = parse_parameter(Code::Misc, &parameter_string, game_type, engine_type);

                    if let Some(mut parsed) = parsed {
                        if romanize {
                            parsed = romanize_string(parsed);
                        }

                        set_mut_ref.insert(parsed);
                        let string_ref: &str = unsafe { set_ref.last().unwrap_unchecked() }.as_str();

                        if processing_mode == ProcessingMode::Append && !map.contains_key(string_ref) {
                            map.shift_insert(set_ref.len() - 1, string_ref, "");
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

/// Reads all Map files of maps_path and parses them into .txt files in output_path.
/// # Parameters
/// * `maps_path` - path to directory than contains game files
/// * `output_path` - path to output directory
/// * `maps_processing_mode` - how to deal with lines duplicates in maps
/// * `romanize` - whether to romanize text
/// * `logging` - whether to log
/// * `game_type` - game type for custom parsing
/// * `engine_type` - which engine's files are we processing, essential for the right processing
/// * `processing_mode` - whether to read in default mode, force rewrite or append new text to existing files
/// * `file_parsed_msg` - message to log when file is parsed
/// * `file_already_parsed_msg` - message to log when file that's about to be parsed already exists (default processing mode)
/// * `file_is_not_parsed_msg` - message to log when file that's about to be parsed not exist (append processing mode)
pub fn read_map(
    original_path: &Path,
    output_path: &Path,
    maps_processing_mode: MapsProcessingMode,
    romanize: bool,
    logging: bool,
    game_type: Option<GameType>,
    engine_type: EngineType,
    mut processing_mode: ProcessingMode,
    (file_parsed_msg, file_already_parsed_msg, file_is_not_parsed_msg): (&str, &str, &str),
) {
    let output_path: &Path = &output_path.join("maps.txt");

    if processing_mode == ProcessingMode::Default && output_path.exists() {
        println!("maps_trans.txt {file_already_parsed_msg}");
        return;
    }

    let obj_vec_iter =
        read_dir(original_path)
            .unwrap()
            .filter_map(|entry: Result<DirEntry, std::io::Error>| match entry {
                Ok(entry) => {
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
                }
                Err(_) => None,
            });

    if maps_processing_mode != MapsProcessingMode::Preserve {
        let mut lines_vec: Vec<String> = Vec::new();

        let lines: UnsafeCell<Xxh3IndexSet> = UnsafeCell::new(IndexSet::default());
        let lines_mut_ref: &mut Xxh3IndexSet = unsafe { &mut *lines.get() };
        let lines_ref: &Xxh3IndexSet = unsafe { &*lines.get() };

        let mut lines_map: Xxh3IndexMap = IndexMap::default();

        let original_content: String = if processing_mode == ProcessingMode::Append {
            read_to_string(output_path).unwrap()
        } else {
            String::new()
        };

        if processing_mode == ProcessingMode::Append {
            if output_path.exists() {
                for line in original_content.split('\n') {
                    let (original, translated) = line.split_once(LINES_SEPARATOR).unwrap();
                    lines_map.insert(original, translated);
                }
            } else {
                println!("{file_is_not_parsed_msg}");
                processing_mode = ProcessingMode::Default;
            }
        };

        // 401 - dialogue lines
        // 102 - dialogue choices array
        // 356 - system lines (special texts)
        // 324, 320 - i don't know what is it but it's some used in-game lines
        const ALLOWED_CODES: [u16; 5] = [102, 320, 324, 356, 401];

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

        for (filename, obj) in obj_vec_iter {
            let mut filename_comment: String = format!("<!-- {filename} -->");

            if let Some(display_name) = obj[display_name_label].as_str() {
                if !display_name.is_empty() {
                    let mut display_name_string: String = display_name.to_owned();

                    if romanize {
                        display_name_string = romanize_string(display_name_string);
                    }

                    filename_comment.insert(filename_comment.len() - 3, ' ');
                    filename_comment.insert_str(filename_comment.len() - 4, &display_name_string);
                }
            }

            match maps_processing_mode {
                MapsProcessingMode::Default => {
                    lines_mut_ref.insert(filename_comment);

                    if processing_mode == ProcessingMode::Append {
                        lines_map.shift_insert(lines_ref.len() - 1, unsafe { lines_ref.last().unwrap_unchecked() }, "");
                    }
                }
                MapsProcessingMode::Separate => {
                    lines_vec.extend(lines_mut_ref.drain(..));
                    lines_vec.push(filename_comment);

                    if processing_mode == ProcessingMode::Append {
                        lines_map.shift_insert(lines_ref.len() - 1, unsafe { lines_ref.last().unwrap_unchecked() }, "");
                    }
                }
                _ => unreachable!(),
            }

            let events_arr: Vec<&Value> = if engine_type == EngineType::New {
                obj[events_label].as_array().unwrap().iter().skip(1).collect()
            } else {
                obj[events_label]
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|(_, value)| value)
                    .collect()
            };

            for event in events_arr {
                if !event[pages_label].is_array() {
                    continue;
                }

                for page in event[pages_label].as_array().unwrap() {
                    parse_list(
                        page[list_label].as_array().unwrap(),
                        &ALLOWED_CODES,
                        romanize,
                        game_type,
                        engine_type,
                        processing_mode,
                        (code_label, parameters_label),
                        &lines,
                        &mut lines_map,
                    );
                }
            }

            if logging {
                println!("{file_parsed_msg} {filename}");
            }
        }

        let mut output_content: String = if processing_mode == ProcessingMode::Append {
            String::from_iter(
                lines_map
                    .into_iter()
                    .map(|(original, translated)| format!("{original}{LINES_SEPARATOR}{translated}\n")),
            )
        } else {
            match maps_processing_mode {
                MapsProcessingMode::Default => String::from_iter(
                    lines
                        .into_inner()
                        .into_iter()
                        .map(|line: String| line + LINES_SEPARATOR + "\n"),
                ),
                MapsProcessingMode::Separate => {
                    String::from_iter(lines_vec.into_iter().map(|line: String| line + LINES_SEPARATOR + "\n"))
                }
                _ => unreachable!(),
            }
        };

        output_content.pop();

        write(output_path, output_content).unwrap();
    } else {
        let mut names_lines_vec: VecDeque<String> = VecDeque::new();
        let mut lines_vec: Vec<(String, String)> = Vec::new();
        let mut lines_pos: usize = 0;

        let original_content: String = if processing_mode == ProcessingMode::Append {
            read_to_string(output_path).unwrap()
        } else {
            String::new()
        };

        if processing_mode == ProcessingMode::Append {
            if output_path.exists() {
                for line in original_content.split('\n') {
                    let (original, translated) = line.split_once(LINES_SEPARATOR).unwrap();

                    if original.starts_with("<!-- Map") {
                        if original.len() > 20 {
                            names_lines_vec.push_back(translated.to_owned());
                        }

                        continue;
                    }

                    lines_vec.push((original.to_owned(), translated.to_owned()));
                }
            } else {
                println!("{file_is_not_parsed_msg}");
                processing_mode = ProcessingMode::Default;
            }
        };

        // 401 - dialogue lines
        // 102 - dialogue choices array
        // 356 - system lines (special texts)
        // 324, 320 - i don't know what is it but it's some used in-game lines
        const ALLOWED_CODES: [u16; 5] = [102, 320, 324, 356, 401];

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

        for (filename, obj) in obj_vec_iter {
            let mut filename_comment: String = format!("<!-- {filename} -->");

            if let Some(display_name) = obj[display_name_label].as_str() {
                if !display_name.is_empty() {
                    let mut display_name_string: String = display_name.to_owned();

                    if romanize {
                        display_name_string = romanize_string(display_name_string);
                    }

                    filename_comment.insert(filename_comment.len() - 3, ' ');
                    filename_comment.insert_str(filename_comment.len() - 4, &display_name_string);
                }
            }

            let filename_comment_len: usize = filename_comment.len();
            lines_vec.insert(
                lines_pos,
                (
                    filename_comment,
                    if filename_comment_len > 20 && names_lines_vec.front().is_some() {
                        names_lines_vec.pop_front().unwrap()
                    } else {
                        String::new()
                    },
                ),
            );
            lines_pos += 1;

            let events_arr: Vec<&Value> = if engine_type == EngineType::New {
                obj[events_label].as_array().unwrap().iter().skip(1).collect()
            } else {
                obj[events_label]
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|(_, value)| value)
                    .collect()
            };

            for event in events_arr {
                if !event[pages_label].is_array() {
                    continue;
                }

                for page in event[pages_label].as_array().unwrap() {
                    let list: &Array = page[list_label].as_array().unwrap();
                    let allowed_codes: &[u16] = &ALLOWED_CODES;
                    let (code_label, parameters_label) = (code_label, parameters_label);
                    let mut in_sequence: bool = false;
                    let mut line: Vec<String> = Vec::with_capacity(8);

                    for item in list {
                        let code: u16 = item[code_label].as_u64().unwrap() as u16;

                        if in_sequence && ![401, 405].contains(&code) {
                            if !line.is_empty() {
                                let mut joined: String = line.join("\n").trim().replace('\n', NEW_LINE);

                                if romanize {
                                    joined = romanize_string(joined);
                                }

                                let parsed: Option<String> =
                                    parse_parameter(Code::Dialogue, &joined, game_type, engine_type);

                                if let Some(parsed) = parsed {
                                    if processing_mode == ProcessingMode::Append {
                                        if let Some((o, _)) = lines_vec.get(lines_pos) {
                                            if *o != parsed {
                                                lines_vec.insert(lines_pos, (parsed, String::new()));
                                            }
                                        }
                                    } else {
                                        lines_vec.push((parsed, String::new()));
                                    }

                                    lines_pos += 1;
                                }

                                line.clear();
                            }

                            in_sequence = false;
                        }

                        if !allowed_codes.contains(&code) {
                            continue;
                        }

                        let parameters: &Array = item[parameters_label].as_array().unwrap();

                        match code {
                            401 | 405 => {
                                let parameter_string: String = parameters[0]
                                    .as_str()
                                    .map(str::to_owned)
                                    .unwrap_or(match parameters[0].as_object() {
                                        Some(obj) => get_object_data(obj),
                                        None => String::new(),
                                    })
                                    .trim()
                                    .to_owned();

                                if !parameter_string.is_empty() {
                                    in_sequence = true;
                                    line.push(parameter_string);
                                }
                            }
                            102 => {
                                for i in 0..parameters[0].as_array().unwrap().len() {
                                    let subparameter_string: String = parameters[0][i]
                                        .as_str()
                                        .map(str::to_owned)
                                        .unwrap_or(match parameters[0][i].as_object() {
                                            Some(obj) => get_object_data(obj),
                                            None => String::new(),
                                        })
                                        .trim()
                                        .to_owned();

                                    if !subparameter_string.is_empty() {
                                        let parsed: Option<String> =
                                            parse_parameter(Code::Choice, &subparameter_string, game_type, engine_type);

                                        if let Some(mut parsed) = parsed {
                                            if romanize {
                                                parsed = romanize_string(parsed);
                                            }

                                            if processing_mode == ProcessingMode::Append {
                                                if let Some((o, _)) = lines_vec.get(lines_pos) {
                                                    if *o != parsed {
                                                        lines_vec.insert(lines_pos, (parsed, String::new()));
                                                    }
                                                }
                                            } else {
                                                lines_vec.push((parsed, String::new()));
                                            }

                                            lines_pos += 1;
                                        }
                                    }
                                }
                            }
                            356 => {
                                let parameter_string: String = parameters[0]
                                    .as_str()
                                    .map(str::to_owned)
                                    .unwrap_or(match parameters[0].as_object() {
                                        Some(obj) => get_object_data(obj),
                                        None => String::new(),
                                    })
                                    .trim()
                                    .to_owned();

                                if !parameter_string.is_empty() {
                                    let parsed: Option<String> =
                                        parse_parameter(Code::System, &parameter_string, game_type, engine_type);

                                    if let Some(mut parsed) = parsed {
                                        if romanize {
                                            parsed = romanize_string(parsed);
                                        }

                                        if processing_mode == ProcessingMode::Append {
                                            if let Some((o, _)) = lines_vec.get(lines_pos) {
                                                if *o != parsed {
                                                    lines_vec.insert(lines_pos, (parsed, String::new()));
                                                }
                                            }
                                        } else {
                                            lines_vec.push((parsed, String::new()));
                                        }

                                        lines_pos += 1;
                                    }
                                }
                            }
                            320 | 324 => {
                                let parameter_string: String = parameters[1]
                                    .as_str()
                                    .map(str::to_owned)
                                    .unwrap_or(match parameters[1].as_object() {
                                        Some(obj) => get_object_data(obj),
                                        None => String::new(),
                                    })
                                    .trim()
                                    .to_owned();

                                if !parameter_string.is_empty() {
                                    let parsed: Option<String> =
                                        parse_parameter(Code::Misc, &parameter_string, game_type, engine_type);

                                    if let Some(mut parsed) = parsed {
                                        if romanize {
                                            parsed = romanize_string(parsed);
                                        }

                                        if processing_mode == ProcessingMode::Append {
                                            if let Some((o, _)) = lines_vec.get(lines_pos) {
                                                if *o != parsed {
                                                    lines_vec.insert(lines_pos, (parsed, String::new()));
                                                }
                                            }
                                        } else {
                                            lines_vec.push((parsed, String::new()));
                                        }

                                        lines_pos += 1;
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }

            if logging {
                println!("{file_parsed_msg} {filename}");
            }
        }

        let mut output_content: String =
            String::from_iter(lines_vec.into_iter().map(|(o, t)| o + LINES_SEPARATOR + &t + "\n"));

        output_content.pop();

        write(output_path, output_content).unwrap();
    }
}

/// Reads all other files of original_path and parses them into .txt files in output_path.
/// # Parameters
/// * `original_path` - path to directory than contains game files
/// * `output_path` - path to output directory
/// * `romanize` - whether to romanize text
/// * `logging` - whether to log
/// * `game_type` - game type for custom parsing
/// * `engine_type` - which engine's files are we processing, essential for the right processing
/// * `processing_mode` - whether to read in default mode, force rewrite or append new text to existing files
/// * `file_parsed_msg` - message to log when file is parsed
/// * `file_already_parsed_msg` - message to log when file that's about to be parsed already exists (default processing mode)
/// * `file_is_not_parsed_msg` - message to log when file that's about to be parsed not exist (append processing mode)
pub fn read_other(
    original_path: &Path,
    output_path: &Path,
    romanize: bool,
    logging: bool,
    game_type: Option<GameType>,
    processing_mode: ProcessingMode,
    engine_type: EngineType,
    (file_parsed_msg, file_already_parsed_msg, file_is_not_parsed_msg): (&str, &str, &str),
) {
    let obj_arr_iter =
        read_dir(original_path)
            .unwrap()
            .filter_map(|entry: Result<DirEntry, std::io::Error>| match entry {
                Ok(entry) => {
                    let filename_os_string: OsString = entry.file_name();
                    let filename: &str = unsafe { from_utf8_unchecked(filename_os_string.as_encoded_bytes()) };
                    let (name, _) = filename.split_once('.').unwrap();

                    if !name.starts_with("Map")
                        && !matches!(name, "Tilesets" | "Animations" | "System" | "Scripts")
                        && filename.ends_with(unsafe { EXTENSION })
                    {
                        if game_type.is_some_and(|game_type: GameType| game_type == GameType::Termina)
                            && name == "States"
                        {
                            return None;
                        }

                        let json: Value = if engine_type == EngineType::New {
                            from_str(&read_to_string(entry.path()).unwrap()).unwrap()
                        } else {
                            load(&read(entry.path()).unwrap(), None, Some("")).unwrap()
                        };

                        Some((filename.to_owned(), json))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            });

    let mut inner_processing_mode: ProcessingMode = processing_mode;

    // 401 - dialogue lines
    // 405 - credits lines
    // 102 - dialogue choices array
    // 356 - system lines (special texts)
    // 324, 320 - i don't know what is it but it's some used in-game lines
    const ALLOWED_CODES: [u16; 6] = [102, 320, 324, 356, 401, 405];

    let (
        name_label,
        nickname_label,
        description_label,
        message1_label,
        message2_label,
        message3_label,
        message4_label,
        note_label,
        pages_label,
        list_label,
        code_label,
        parameters_label,
    ) = if engine_type == EngineType::New {
        (
            "name",
            "nickname",
            "description",
            "message1",
            "message2",
            "message3",
            "message4",
            "note",
            "pages",
            "list",
            "code",
            "parameters",
        )
    } else {
        (
            "__symbol__name",
            "__symbol__nickname",
            "__symbol__description",
            "__symbol__message1",
            "__symbol__message2",
            "__symbol__message3",
            "__symbol__message4",
            "__symbol__note",
            "__symbol__pages",
            "__symbol__list",
            "__symbol__code",
            "__symbol__parameters",
        )
    };

    for (filename, obj_arr) in obj_arr_iter {
        let output_path: &Path = &output_path.join(filename[0..filename.rfind('.').unwrap()].to_lowercase() + ".txt");

        if processing_mode == ProcessingMode::Default && output_path.exists() {
            println!("{} {file_already_parsed_msg}", output_path.display());
            continue;
        }

        let lines: UnsafeCell<Xxh3IndexSet> = UnsafeCell::new(IndexSet::default());
        let lines_mut_ref: &mut Xxh3IndexSet = unsafe { &mut *lines.get() };
        let lines_ref: &Xxh3IndexSet = unsafe { &*lines.get() };

        let mut lines_map: Xxh3IndexMap = IndexMap::default();

        let original_content: String = if processing_mode == ProcessingMode::Append {
            read_to_string(output_path).unwrap()
        } else {
            String::new()
        };

        if processing_mode == ProcessingMode::Append {
            if output_path.exists() {
                for line in original_content.par_split('\n').collect::<Vec<_>>() {
                    let (original, translated) = line.split_once(LINES_SEPARATOR).unwrap();
                    lines_map.insert(original, translated);
                }
            } else {
                println!("{file_is_not_parsed_msg}");
                inner_processing_mode = ProcessingMode::Default;
            }
        }

        // Other files except CommonEvents and Troops have the structure that consists
        // of name, nickname, description and note
        if !filename.starts_with("Co") && !filename.starts_with("Tr") {
            if game_type.is_some_and(|game_type: GameType| game_type == GameType::Termina) && filename.starts_with("It")
            {
                for string in [
                    "<Menu Category: Items>",
                    "<Menu Category: Food>",
                    "<Menu Category: Healing>",
                    "<Menu Category: Body bag>",
                ] {
                    lines_mut_ref.insert(string.to_owned());
                }
            }

            'obj: for obj in obj_arr.as_array().unwrap() {
                let mut prev_variable_type: Option<Variable> = None;

                for (variable_text, variable_type) in [
                    (obj[name_label].as_str(), Variable::Name),
                    (obj[nickname_label].as_str(), Variable::Nickname),
                    (obj[description_label].as_str(), Variable::Description),
                    (obj[message1_label].as_str(), Variable::Message1),
                    (obj[message2_label].as_str(), Variable::Message2),
                    (obj[message3_label].as_str(), Variable::Message3),
                    (obj[message4_label].as_str(), Variable::Message4),
                    (obj[note_label].as_str(), Variable::Note),
                ] {
                    if let Some(mut variable_str) = variable_text {
                        variable_str = variable_str.trim();

                        if !variable_str.is_empty() {
                            let parsed: Option<(String, bool)> = parse_variable(
                                variable_str.to_owned(),
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

                                    if let Some(last) = lines_mut_ref.pop() {
                                        lines_mut_ref.insert(last.trim().to_owned() + &parsed);
                                        let string_ref: &str = unsafe { lines_ref.last().unwrap_unchecked() }.as_str();

                                        // TODO: this shit rewrites the translation line but inserts RIGHT original line
                                        if inner_processing_mode == ProcessingMode::Append {
                                            let (idx, _, value) = lines_map.shift_remove_full(last.as_str()).unwrap();
                                            lines_map.shift_insert(idx, string_ref, value);
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
                                    .map(str::trim)
                                    .collect::<Vec<_>>()
                                    .join(NEW_LINE)
                                    .trim()
                                    .to_owned();

                                lines_mut_ref.insert(replaced);
                                let string_ref: &str = unsafe { lines_ref.last().unwrap_unchecked() }.as_str();

                                if inner_processing_mode == ProcessingMode::Append
                                    && !lines_map.contains_key(string_ref)
                                {
                                    lines_map.shift_insert(lines_ref.len() - 1, string_ref, "");
                                }
                            } else if variable_type == Variable::Name {
                                continue 'obj;
                            }
                        }
                    }
                }
            }
        }
        // Other files have the structure somewhat similar to Maps files
        else {
            // Skipping first element in array as it is null
            for obj in obj_arr.as_array().unwrap().iter().skip(1) {
                // CommonEvents doesn't have pages, so we can just check if it's Troops
                let pages_length: usize = if filename.starts_with("Tr") {
                    obj[pages_label].as_array().unwrap().len()
                } else {
                    1
                };

                for i in 0..pages_length {
                    let list: &Value = if pages_length != 1 {
                        &obj[pages_label][i][list_label]
                    } else {
                        &obj[list_label]
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
                        (code_label, parameters_label),
                        &lines,
                        &mut lines_map,
                    );
                }
            }
        }

        let mut output_content: String = if processing_mode == ProcessingMode::Append {
            String::from_iter(
                lines_map
                    .into_iter()
                    .map(|(original, translated)| format!("{original}{LINES_SEPARATOR}{translated}\n")),
            )
        } else {
            String::from_iter(
                lines
                    .into_inner()
                    .into_iter()
                    .map(|line: String| line + LINES_SEPARATOR + "\n"),
            )
        };

        output_content.pop();

        write(output_path, output_content).unwrap();

        if logging {
            println!("{file_parsed_msg} {filename}");
        }
    }
}

/// Reads System file of system_file_path and parses it into .txt file of output_path.
/// # Parameters
/// * `system_file_path` - path to directory than contains game files
/// * `output_path` - path to output directory
/// * `romanize` - whether to romanize text
/// * `logging` - whether to log
/// * `processing_mode` - whether to read in default mode, force rewrite or append new text to existing files
/// * `engine_type` - which engine's files are we processing, essential for the right processing
/// * `file_parsed_msg` - message to log when file is parsed
/// * `file_already_parsed_msg` - message to log when file that's about to be parsed already exists (default processing mode)
/// * `file_is_not_parsed_msg` - message to log when file that's about to be parsed not exist (append processing mode)
pub fn read_system(
    system_file_path: &Path,
    output_path: &Path,
    romanize: bool,
    logging: bool,
    mut processing_mode: ProcessingMode,
    engine_type: EngineType,
    (file_parsed_msg, file_already_parsed_msg, file_is_not_parsed_msg): (&str, &str, &str),
) {
    let output_path: &Path = &output_path.join("system.txt");

    if processing_mode == ProcessingMode::Default && output_path.exists() {
        println!("system.txt {file_already_parsed_msg}");
        return;
    }

    let obj: Value = if engine_type == EngineType::New {
        from_str(&read_to_string(system_file_path).unwrap()).unwrap()
    } else {
        load(&read(system_file_path).unwrap(), None, Some("")).unwrap()
    };

    let lines: UnsafeCell<Xxh3IndexSet> = UnsafeCell::new(IndexSet::default());
    let lines_mut_ref: &mut Xxh3IndexSet = unsafe { &mut *lines.get() };
    let lines_ref: &Xxh3IndexSet = unsafe { &*lines.get() };

    let mut lines_map: Xxh3IndexMap = IndexMap::default();

    let original_content: String = if processing_mode == ProcessingMode::Append {
        read_to_string(output_path).unwrap()
    } else {
        String::new()
    };

    if processing_mode == ProcessingMode::Append {
        if output_path.exists() {
            for line in original_content.par_split('\n').collect::<Vec<_>>() {
                let (original, translated) = line.split_once(LINES_SEPARATOR).unwrap();
                lines_map.insert(original, translated);
            }
        } else {
            println!("{file_is_not_parsed_msg}");
            processing_mode = ProcessingMode::Default;
        }
    }

    if engine_type != EngineType::New {
        let str: &str = obj["__symbol__currency_unit"].as_str().unwrap().trim();

        if !str.is_empty() {
            let mut string: String = str.to_owned();

            if romanize {
                string = romanize_string(string)
            }

            lines_mut_ref.insert(string);
            let string_ref: &str = unsafe { lines_ref.last().unwrap_unchecked() }.as_str();

            if processing_mode == ProcessingMode::Append && !lines_map.contains_key(string_ref) {
                lines_map.shift_insert(lines_ref.len() - 1, string_ref, "");
            }
        }
    }

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

    // Armor types names
    // Normally it's system strings, but might be needed for some purposes
    for string in obj[armor_types_label].as_array().unwrap() {
        let str: &str = string.as_str().unwrap().trim();

        if !str.is_empty() {
            let mut string: String = str.to_owned();

            if romanize {
                string = romanize_string(string)
            }

            lines_mut_ref.insert(string);
            let string_ref: &str = unsafe { lines_ref.last().unwrap_unchecked() }.as_str();

            if processing_mode == ProcessingMode::Append && !lines_map.contains_key(string_ref) {
                lines_map.shift_insert(lines_ref.len() - 1, string_ref, "");
            }
        }
    }

    // Element types names
    // Normally it's system strings, but might be needed for some purposes
    for string in obj[elements_label].as_array().unwrap() {
        let str: &str = string.as_str().unwrap().trim();

        if !str.is_empty() {
            let mut string: String = str.to_owned();

            if romanize {
                string = romanize_string(string)
            }

            lines_mut_ref.insert(string);
            let string_ref: &str = unsafe { lines_ref.last().unwrap_unchecked() }.as_str();

            if processing_mode == ProcessingMode::Append && !lines_map.contains_key(string_ref) {
                lines_map.shift_insert(lines_ref.len() - 1, string_ref, "");
            }
        }
    }

    // Names of equipment slots
    if engine_type == EngineType::New {
        for string in obj["equipTypes"].as_array().unwrap() {
            let str: &str = string.as_str().unwrap().trim();

            if !str.is_empty() {
                let mut string: String = str.to_owned();

                if romanize {
                    string = romanize_string(string)
                }

                lines_mut_ref.insert(string);
                let string_ref: &str = unsafe { lines_ref.last().unwrap_unchecked() }.as_str();

                if processing_mode == ProcessingMode::Append && !lines_map.contains_key(string_ref) {
                    lines_map.shift_insert(lines_ref.len() - 1, string_ref, "");
                }
            }
        }
    }

    // Names of battle options
    for string in obj[skill_types_label].as_array().unwrap() {
        let str: &str = string.as_str().unwrap().trim();

        if !str.is_empty() {
            let mut string: String = str.to_owned();

            if romanize {
                string = romanize_string(string)
            }

            lines_mut_ref.insert(string);
            let string_ref: &str = unsafe { lines_ref.last().unwrap_unchecked() }.as_str();

            if processing_mode == ProcessingMode::Append && !lines_map.contains_key(string_ref) {
                lines_map.shift_insert(lines_ref.len() - 1, string_ref, "");
            }
        }
    }

    // Game terms vocabulary
    for (key, value) in obj[terms_label].as_object().unwrap() {
        if !key.starts_with("__symbol__") {
            continue;
        }

        if key != "messages" {
            for string in value.as_array().unwrap() {
                if let Some(mut str) = string.as_str() {
                    str = str.trim();

                    if !str.is_empty() {
                        let mut string: String = str.to_owned();

                        if romanize {
                            string = romanize_string(string)
                        }

                        lines_mut_ref.insert(string);
                        let string_ref: &str = unsafe { lines_ref.last().unwrap_unchecked() }.as_str();

                        if processing_mode == ProcessingMode::Append && !lines_map.contains_key(string_ref) {
                            lines_map.shift_insert(lines_ref.len() - 1, string_ref, "");
                        }
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
                    let mut string: String = str.to_owned();

                    if romanize {
                        string = romanize_string(string)
                    }

                    lines_mut_ref.insert(string);
                    let string_ref: &str = unsafe { lines_ref.last().unwrap_unchecked() }.as_str();

                    if processing_mode == ProcessingMode::Append && !lines_map.contains_key(string_ref) {
                        lines_map.shift_insert(lines_ref.len() - 1, string_ref, "");
                    }
                }
            }
        }
    }

    // Weapon types names
    // Normally it's system strings, but might be needed for some purposes
    for string in obj[weapon_types_label].as_array().unwrap() {
        let str: &str = string.as_str().unwrap().trim();

        if !str.is_empty() {
            let mut string: String = str.to_owned();

            if romanize {
                string = romanize_string(string)
            }

            lines_mut_ref.insert(string);
            let string_ref: &str = unsafe { lines_ref.last().unwrap_unchecked() }.as_str();

            if processing_mode == ProcessingMode::Append && !lines_map.contains_key(string_ref) {
                lines_map.shift_insert(lines_ref.len() - 1, string_ref, "");
            }
        }
    }

    // Game title, parsed just for fun
    // Translators may add something like "ELFISH TRANSLATION v1.0.0" to the title
    {
        let mut game_title_string: String = obj[game_title_label].as_str().unwrap().trim().to_owned();

        if romanize {
            game_title_string = romanize_string(game_title_string)
        }

        lines_mut_ref.insert(game_title_string);
        let string_ref: &str = unsafe { lines_ref.last().unwrap_unchecked() }.as_str();

        if processing_mode == ProcessingMode::Append && !lines_map.contains_key(string_ref) {
            lines_map.shift_insert(lines_ref.len() - 1, string_ref, "");
        }
    }

    let mut output_content: String = if processing_mode == ProcessingMode::Append {
        String::from_iter(
            lines_map
                .into_iter()
                .map(|(original, translated)| format!("{original}{LINES_SEPARATOR}{translated}\n")),
        )
    } else {
        String::from_iter(
            lines
                .into_inner()
                .into_iter()
                .map(|line: String| line + LINES_SEPARATOR + "\n"),
        )
    };

    output_content.pop();

    write(output_path, output_content).unwrap();

    if logging {
        println!("{file_parsed_msg} {}", system_file_path.display());
    }
}

pub fn read_scripts(scripts_file_path: &Path, other_path: &Path, romanize: bool, logging: bool, file_parsed_msg: &str) {
    let mut strings: Vec<String> = Vec::new();

    let scripts_entries: Value = load(&read(scripts_file_path).unwrap(), Some(StringMode::Binary), None).unwrap();

    let encodings: [&Encoding; 5] = [
        encoding_rs::UTF_8,
        encoding_rs::WINDOWS_1252,
        encoding_rs::WINDOWS_1251,
        encoding_rs::SHIFT_JIS,
        encoding_rs::GB18030,
    ];

    let mut codes_content: Vec<String> = Vec::with_capacity(256);

    for code in scripts_entries.as_array().unwrap() {
        let bytes_stream: Vec<u8> = from_value(&code[2]["data"]).unwrap();

        let mut inflated: Vec<u8> = Vec::new();
        ZlibDecoder::new(&*bytes_stream).read_to_end(&mut inflated).unwrap();

        let mut code: String = String::new();

        for encoding in encodings {
            let (cow, _, had_errors) = encoding.decode(&inflated);

            if !had_errors {
                code = cow.into_owned();
                break;
            }
        }

        codes_content.push(code);
    }

    let extracted_strings: Xxh3IndexSet = extract_strings(&codes_content.join(""), false).0;

    let regexes: [Regex; 11] = [
        unsafe { Regex::new(r"(Graphics|Data|Audio|Movies|System)\/.*\/?").unwrap_unchecked() },
        unsafe { Regex::new(r"r[xv]data2?$").unwrap_unchecked() },
        STRING_IS_ONLY_SYMBOLS_RE.to_owned(),
        unsafe { Regex::new(r"@window").unwrap_unchecked() },
        unsafe { Regex::new(r"\$game").unwrap_unchecked() },
        unsafe { Regex::new(r"_").unwrap_unchecked() },
        unsafe { Regex::new(r"^\\e").unwrap_unchecked() },
        unsafe { Regex::new(r".*\(").unwrap_unchecked() },
        unsafe { Regex::new(r"^([d\d\p{P}+-]*|[d\p{P}+-]&*)$").unwrap_unchecked() },
        unsafe { Regex::new(r"ALPHAC").unwrap_unchecked() },
        unsafe {
            Regex::new(r"^(Actor<id>|ExtraDropItem|EquipLearnSkill|GameOver|Iconset|Window|true|false|MActor%d|[wr]b|\\f|\\n|\[[A-Z]*\])$").unwrap_unchecked()
        },
    ];

    'extracted: for mut extracted in extracted_strings {
        if extracted.is_empty() {
            continue;
        }

        for re in regexes.iter() {
            if re.is_match(&extracted) {
                continue 'extracted;
            }
        }

        if romanize {
            extracted = romanize_string(extracted);
        }

        strings.push(extracted);
    }

    if logging {
        println!("{file_parsed_msg} {}", scripts_file_path.display());
    }

    let mut output_content: String =
        String::from_iter(strings.into_iter().map(|line: String| line + LINES_SEPARATOR + "\n"));

    output_content.pop();

    write(other_path.join("scripts.txt"), output_content).unwrap();
}

// read_plugins is not implemented and will NEVER be, as plugins can differ from each other incredibly.
// Change plugins.js with your own hands.
