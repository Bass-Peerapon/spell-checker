use const_format::formatcp;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead};
use std::iter::FromIterator;

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
    let corpus = get_corpus("../data/tnc_freq.txt", true); // You can use the get_corpus function
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
    let char_vec: Vec<char> = word.chars().collect(); // เปลี่ยนสตริงให้เป็นอาเรย์ตัวอักษร

    // สร้างรายการการแบ่งคำที่ตัวอักษร (ไม่ใช่ไบต์)
    let splits: Vec<(Vec<char>, Vec<char>)> = (0..=char_vec.len())
        .map(|i| (char_vec[..i].to_vec(), char_vec[i..].to_vec()))
        .collect();

    // Deletes (ลบตัวอักษรหนึ่งตัวออกจากคำ)
    for (l, r) in &splits {
        if !r.is_empty() {
            let mut candidate = String::with_capacity(l.len() + r.len() - 1);
            candidate.extend(l);
            candidate.extend(&r[1..]);
            edits.insert(candidate);
        }
    }

    // Transposes (สลับตัวอักษรสองตัว)
    for (l, r) in &splits {
        if r.len() > 1 {
            let mut candidate = String::with_capacity(l.len() + r.len());
            candidate.extend(l);
            candidate.push(r[1]);
            candidate.push(r[0]);
            candidate.extend(&r[2..]);
            edits.insert(candidate);
        }
    }

    // Replaces (แทนที่ตัวอักษรหนึ่งตัว)
    for (l, r) in &splits {
        if !r.is_empty() {
            for c in THAI_LETTERS.chars() {
                let mut candidate = String::with_capacity(l.len() + r.len());
                candidate.extend(l);
                candidate.push(c);
                candidate.extend(&r[1..]);
                edits.insert(candidate);
            }
        }
    }

    // Inserts (แทรกตัวอักษรหนึ่งตัว)
    for (l, r) in &splits {
        for c in THAI_LETTERS.chars() {
            let mut candidate = String::with_capacity(l.len() + r.len() + 1);
            candidate.extend(l);
            candidate.push(c);
            candidate.extend(r);
            edits.insert(candidate);
        }
    }

    edits
}

fn _edits2(word: &str) -> HashSet<String> {
    let mut result = HashSet::new();
    for e1 in _edits1(word) {
        result.extend(_edits1(&e1));
    }
    result
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
        let candidates: HashSet<String> = self
            .known(&HashSet::from_iter(vec![word.to_string()]))
            .into_iter()
            .chain(self.known(&_edits1(word)))
            // .chain(self.known(&_edits2(word)))
            .collect();

        let mut candidates_vec: Vec<String> = candidates.into_iter().collect();
        candidates_vec.sort_by(|a, b| {
            self.freq(b).cmp(&self.freq(a)).then_with(|| {
                self.prob(b)
                    .partial_cmp(&self.prob(a))
                    .unwrap_or(Ordering::Equal)
            })
        });

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
