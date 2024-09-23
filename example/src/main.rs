use spell_checker::{load_custom_dict, tnc_freq_path, NorvigSpellChecker};

fn main() {
    let custom_dict = load_custom_dict(tnc_freq_path());
    let spell_checker = NorvigSpellChecker::new_with_custom_dict(custom_dict);

    // Example usage
    let word1 = "เหตการณ";

    let candidates = spell_checker.spell(word1);

    println!("Candidates for '{}': {:?}", word1, candidates);
    assert_eq!(candidates, vec!["เหตุการณ์"]);
}
