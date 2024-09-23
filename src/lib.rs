use const_format::formatcp;
use core::str;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead};
use std::time::Instant;

const THAI_CONSONANTS: &str = "กขฃคฅฆงจฉชซฌญฎฏฐฑฒณดตถทธนบปผฝพฟภมยรลวศษสหฬอฮ"; // 44 chars
const THAI_VOWELS: &str = "\u{0e24}\u{0e26}\u{0e30}\u{0e31}\u{0e32}\u{0e33}\u{0e34}\u{0e35}\u{0e36}\u{0e37}\u{0e38}\u{0e39}\u{0e40}\u{0e41}\u{0e42}\u{0e43}\u{0e44}\u{0e45}\u{0e4d}\u{0e47}";
const THAI_TONEMARKS: &str = "\u{0e48}\u{0e49}\u{0e4a}\u{0e4b}";
const THAI_SIGNS: &str = "\u{0e2f}\u{0e3a}\u{0e46}\u{0e4c}\u{0e4d}\u{0e4e}";
const THAI_LETTERS: &str = formatcp!(
    "{}{}{}{}",
    THAI_CONSONANTS,
    THAI_VOWELS,
    THAI_TONEMARKS,
    THAI_SIGNS
); // 74 chars
const THAI_DIGITS: &str = "๐๑๒๓๔๕๖๗๘๙";
const DIGITS: &str = "0123456789";

macro_rules! insert_prefix_str {
    ($filename:expr) => {
        if cfg!(feature = "onedir") {
            concat!("./", $filename)
        } else {
            concat!(env!("CARGO_MANIFEST_DIR"), "/data/", $filename)
        }
    };
}

pub fn tnc_freq_path() -> Option<String> {
    Some(insert_prefix_str!("tnc_freq.txt").to_owned())
}

fn is_thai_char(c: char) -> bool {
    THAI_LETTERS.contains(c)
}

fn is_thai_digit(c: char) -> bool {
    THAI_DIGITS.contains(c)
}

fn is_digit(c: char) -> bool {
    DIGITS.contains(c)
}

pub fn is_thai_and_not_num(word: &str) -> bool {
    for c in word.chars() {
        if c != '.' && !is_thai_char(c) {
            return false;
        };

        if is_thai_digit(c) || is_digit(c) {
            return false;
        };
    }

    true
}
fn get_corpus(filename: &str, comments: bool) -> HashSet<String> {
    let mut lines_set = HashSet::new();

    if let Ok(file) = File::open(filename) {
        let reader = io::BufReader::new(file);

        for mut line in reader.lines().map_while(Result::ok) {
            if !comments {
                if let Some(pos) = line.find('#') {
                    line = line[..pos].trim().to_string();
                }
            }
            if !line.is_empty() {
                lines_set.insert(line.trim().to_string());
            }
        }
    }

    lines_set
}
fn word_freqs() -> Vec<(String, usize)> {
    let corpus = get_corpus(tnc_freq_path().unwrap().as_str(), true); // You can use the get_corpus function
    let mut word_freqs = Vec::new();

    for line in corpus {
        let word_freq: Vec<&str> = line.split('\t').collect();
        if word_freq.len() >= 2 {
            if let Ok(freq) = word_freq[1].parse::<usize>() {
                word_freqs.push((word_freq[0].to_string(), freq));
            }
        }
    }

    word_freqs
}

fn _edits1(word: &str) -> HashSet<String> {
    let mut edits = HashSet::new();
    let mut splits: Vec<(&str, &str)> = Vec::new();
    for (i, _) in word.char_indices() {
        splits.push(word.split_at(i));
    }
    splits.push((word, ""));

    // Deletes (ลบตัวอักษรหนึ่งตัวออกจากคำ)
    for (l, r) in &splits {
        if !r.is_empty() {
            let mut chars = r.chars();
            chars.next(); // Skip the first char (deletion)
            edits.insert(format!("{}{}", l, chars.collect::<String>()));
        }
    }

    // Transposes (สลับตัวอักษรสองตัว)
    for (l, r) in &splits {
        let c: Vec<char> = r.chars().collect();
        if c.len() > 1 {
            let mut r_chars = r.chars().collect::<Vec<char>>();
            r_chars.swap(0, 1);
            edits.insert(format!("{}{}", l, r_chars.iter().collect::<String>()));
        }
    }

    // Replaces (แทนที่ตัวอักษรหนึ่งตัว)
    for (l, r) in &splits {
        if r.chars().next().is_some() {
            for c in THAI_LETTERS.chars() {
                let mut r_chars = r.chars();
                r_chars.next(); // Skip the first char for replacement
                edits.insert(format!("{}{}{}", l, c, r_chars.collect::<String>()));
            }
        }
    }

    // Inserts (แทรกตัวอักษรหนึ่งตัว)
    for (l, r) in &splits {
        for c in THAI_LETTERS.chars() {
            edits.insert(format!("{}{}{}", l, c, r));
        }
    }

    edits
}

fn _edits2(word: &str) -> HashSet<String> {
    _edits1(word)
        .into_iter()
        .flat_map(|e1| _edits1(&e1))
        .collect()
}

fn _keep(
    word_freq: &(String, usize), // Tuple equivalent in Rust: (word, frequency)
    min_freq: usize,
    min_len: usize,
    max_len: usize,
    dict_filter: Option<fn(&str) -> bool>, // Optional filter function
) -> bool {
    // Check if the word or its frequency does not meet the minimum frequency
    if word_freq.1 < min_freq {
        return false;
    }

    let word = &word_freq.0;

    // Check if word length is between min_len and max_len, and does not start with '.'
    if word.is_empty() || word.len() < min_len || word.len() > max_len || word.starts_with('.') {
        return false;
    }

    // Apply the filter function, if one is provided
    if let Some(filter_fn) = dict_filter {
        return filter_fn(word);
    }

    true
}

// Function to convert custom dictionary types into a Vec<(String, usize)> and apply filtering
fn convert_custom_dict(
    custom_dict: CustomDict, // Custom type to represent different input types
    min_freq: usize,
    min_len: usize,
    max_len: usize,
    dict_filter: Option<fn(&str) -> bool>, // Optional filter function
) -> HashMap<String, usize> {
    let mut result = HashMap::new();

    match custom_dict {
        CustomDict::HashMap(dict) => {
            // If it's a HashMap<String, usize>, filter and insert into result
            for (word, freq) in dict {
                if _keep(
                    &(word.clone(), freq),
                    min_freq,
                    min_len,
                    max_len,
                    dict_filter,
                ) {
                    result.insert(word, freq);
                }
            }
        }
    }

    result
}

// Enum to handle different input types (HashMap, Vec<String>, Vec<(String, usize)>)
enum CustomDict {
    HashMap(HashMap<String, usize>),
}

pub struct NorvigSpellChecker {
    word_freqs: HashMap<String, usize>,
    total_words: usize,
}

impl NorvigSpellChecker {
    pub fn new(
        custom_dict: Option<HashMap<String, usize>>,
        min_freq: usize,
        min_len: usize,
        max_len: usize,
        dict_filter: Option<fn(&str) -> bool>,
    ) -> Self {
        let custom_dict = custom_dict.unwrap_or_else(|| {
            // Default: using Thai National Corpus (TNC), this should be loaded from a dataset
            let mut default_dict = HashMap::new();

            word_freqs().into_iter().for_each(|(word, freq)| {
                default_dict.insert(word, freq);
            });
            default_dict
        });

        let word_freqs = convert_custom_dict(
            CustomDict::HashMap(custom_dict),
            min_freq,
            min_len,
            max_len,
            dict_filter,
        );
        let total_words = word_freqs.values().sum();
        NorvigSpellChecker {
            word_freqs,
            total_words,
        }
    }

    pub fn known(&self, words: &HashSet<String>) -> HashSet<String> {
        words
            .iter()
            .filter(|&word| self.word_freqs.contains_key(word))
            .cloned()
            .collect()
    }

    pub fn prob(&self, word: &str) -> f64 {
        *self.word_freqs.get(word).unwrap_or(&0) as f64 / self.total_words as f64
    }

    pub fn freq(&self, word: &str) -> usize {
        *self.word_freqs.get(word).unwrap_or(&0)
    }

    pub fn spell(&self, word: &str) -> Vec<String> {
        let start = Instant::now();
        let mut candidates = self.known(&HashSet::from([word.to_string()]));
        let duration = start.elapsed();

        println!("Time elapsed in known() is: {:?}", duration);

        let start = Instant::now();
        if candidates.is_empty() {
            candidates = self.known(&_edits1(word));
        }
        let duration = start.elapsed();
        println!("Time elapsed in _edits1() is: {:?}", duration);

        let start = Instant::now();
        if candidates.is_empty() {
            candidates = self.known(&_edits2(word));
        }
        let duration = start.elapsed();
        println!("Time elapsed in _edits2() is: {:?}", duration);

        if candidates.is_empty() {
            candidates = HashSet::from([word.to_string()]);
        }

        let mut candidates_vec: Vec<String> = candidates.into_iter().collect();
        // Sort candidates by frequency, highest first
        candidates_vec.sort_by_key(|w| std::cmp::Reverse(self.freq(w)));
        candidates_vec
    }

    pub fn correct(&self, word: &str) -> String {
        if word.parse::<f64>().is_ok() {
            return word.to_string();
        }

        if let Some(candidates) = self.spell(word).first() {
            return candidates.clone();
        }

        word.to_string()
    }

    pub fn dictionary(&self) -> Vec<(String, usize)> {
        self.word_freqs
            .iter()
            .map(|(word, &freq)| (word.clone(), freq))
            .collect()
    }
}

impl Default for NorvigSpellChecker {
    fn default() -> Self {
        NorvigSpellChecker::new(None, 2, 2, 40, Some(is_thai_and_not_num))
    }
}
