use std::collections::{HashMap, HashSet};
use std::iter::Iterator;
use std::str::from_utf8;

// http://www.romjist.ro/content/pdf/08-radescu.pdf

const MIN_LENGH: usize = 2;
const MAX_LENGTH: usize = 22;
const ENCODING: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVXXYZ";
const VALID: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVXXYZ";
const DELIMS: [char; 32] = [
    ' ', '\t', ',', ';', '.', ':', '?', '!', '(', ')', '[', ']', '{', '}', '<', '>', '/', '\\',
    '#', '`', '\'', '"', '|', '=', '-', '&', '_', '‘', '’', '+', '~', '@',
];

struct StarTransformData {
    pub translation_table: HashMap<String, String>,
    pub content: Vec<u8>,
}

fn is_word(word: &str) -> bool {
    let len = word.len();
    if len < MIN_LENGH || len > MAX_LENGTH {
        return false;
    }
    for ch in word.chars() {
        if !VALID.contains(ch) {
            return false;
        }
    }
    true
}

pub fn apply(data: &[u8]) -> Vec<u8> {
    // Assume UTF-8 encoding
    let file_content = match from_utf8(&data) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    let mut dictionaries = vec![HashMap::new(); MAX_LENGTH + 1];
    for line in file_content.lines() {
        for word in line.split(&DELIMS[..]) {
            if is_word(word) {
                // Populate word occurrences in sub dictionary
                *dictionaries[word.len()]
                    .entry(String::from(word))
                    .or_insert(0) += 1;
            }
        }
    }

    // Create translation table
    let mut translation_table = HashMap::new();

    for (i, item) in dictionaries.into_iter().enumerate().skip(1) {
        let mut dict: Vec<(&String, &usize)> = item.iter().collect();
        dict.sort_by(|a, b| b.1.cmp(a.1));

        // Ignore empty dictionaries
        if !dict.is_empty() {
            translation_table.extend(encode_dictionary(dict, i));
        }
    }

    // Verify translation table only holds unique values
    debug_assert!({
        let mut value_set = HashSet::new();
        for value in translation_table.values() {
            value_set.insert(value);
        }
        true
    });

    // Translate content with table
    let final_content = translate_content(file_content, &translation_table);
    println!("{}", &final_content);

    // Prepare serialization data
    let data = StarTransformData {
        translation_table,
        content: Vec::from(final_content.as_bytes()),
    };

    data.content
}

/// Takes a dictionary vector as input (sorted by word occurrence)
fn encode_dictionary(dict: Vec<(&String, &usize)>, len: usize) -> HashMap<String, String> {
    let mut translation_table: HashMap<String, String> = HashMap::with_capacity(dict.len());

    // Create word iterator
    let mut words = dict.into_iter().map(|(k, _)| k.clone());

    // translate first word with '*' * len
    let mut code = vec!['*'; len];
    if let Some(word) = words.next() {
        translation_table.insert(word, code.iter().collect());
    }

    // Iterate over each index in reverse to place single mutation
    for i in (0..len).rev() {
        populate_table(&mut translation_table, &mut words, &code, i);
        code[i] = '*'
    }

    for offset in 1..len {
        for ch in ENCODING.chars() {
            code[len - offset] = ch;
            for i in (0..len - offset).rev() {
                populate_table(&mut translation_table, &mut words, &code, i);
                code[i] = '*'
            }
        }
    }

    for (key, value) in &translation_table {
        println!("key: {}, value: {}", key, value);
    }

    debug_assert!(&words.next().is_none());

    translation_table
}

fn populate_table<I: Iterator>(
    translation_table: &mut HashMap<String, String>,
    words: &mut I,
    code: &[char],
    index: usize,
) where
    I: Iterator<Item = String>,
{
    let mut code = code.to_owned();
    for character in ENCODING.chars() {
        code[index] = character;
        if let Some(word) = words.next() {
            translation_table.insert(word.clone(), code.iter().collect());
        } else {
            break;
        }
    }
}

// TODO: replace with ropey
fn translate_content(content: &str, translation_table: &HashMap<String, String>) -> String {
    let mut final_lines: Vec<String> = Vec::new();
    for line in content.lines().collect::<Vec<&str>>() {
        let mut final_line: String = String::from(line);
        for word in line.split(&DELIMS[..]) {
            if is_word(word) {
                // Replace word in final line
                println!("replacing: {}", String::from(word));
                final_line =
                    final_line.replace(word, &translation_table.get(&String::from(word)).unwrap());
            }
        }
        final_lines.push(final_line);
    }
    //TODO: restore original line seperator
    final_lines.join("\n")
}
