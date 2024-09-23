use spell_checker::NorvigSpellChecker;

fn main() {
    let spell_checker = NorvigSpellChecker::default();

    // Example usage
    let word1 = "เหตการณ";

    let candidates = spell_checker.spell(word1);

    println!("Candidates for '{}': {:?}", word1, candidates);
}
