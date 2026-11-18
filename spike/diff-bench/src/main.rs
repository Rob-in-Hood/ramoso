//! Spike: is `similar` fast enough to diff a 20,000-line file?

use similar::{Algorithm, InlineChangeMode, InlineChangeOptions, TextDiff};
use std::time::{Duration, Instant};

/// Tiny deterministic random numbers, so every run diffs the same files.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        // Knuth's MMIX multiplier and increment.
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

/// One plausible line of code built from random words.
fn code_line(rng: &mut Lcg, index: usize) -> String {
    const WORDS: [&str; 10] = ["let", "value", "user", "session", "token", "await", "return", "config", "store", "ttl"];

    let count = 3 + (rng.next() % 6) as usize;
    let words: Vec<&str> = (0..count).map(|_| WORDS[(rng.next() % 10) as usize]).collect();

    format!("    {} = fn_{}({});\n", words.join("_"), index, words.join(", "))
}

/// Runs `f` once and prints how long it took.
fn time<F: FnOnce() -> usize>(label: &str, f: F) {
    let start = Instant::now();
    let items = f();
    println!("{label:<44} {:>8.1} ms  ({items} items)", start.elapsed().as_secs_f64() * 1000.0);
}

fn main() {
    // Old file: 20,000 lines.
    let mut rng = Lcg(42);
    let old: Vec<String> = (0..20_000).map(|i| code_line(&mut rng, i)).collect();

    // New file: ~1,000 edited lines and a 200-line block moved from line 2,000 to 15,000.
    let mut new = old.clone();
    for _ in 0..1_000 {
        let i = (rng.next() % 20_000) as usize;
        new[i] = new[i].replacen("value", "result", 1);
    }
    let block: Vec<String> = new.drain(2_000..2_200).collect();
    new.splice(15_000..15_000, block);

    // Worst case: a completely different file of the same size.
    let (old_text, new_text) = (old.concat(), new.concat());
    let unrelated: String = (0..20_000).map(|i| code_line(&mut rng, i + 50_000)).collect();

    // Diff settings: give up after 2 s; highlight changed words.
    let timeout = Duration::from_secs(2);
    let mut word_options = InlineChangeOptions::new();
    word_options.mode(InlineChangeMode::Words);

    // Time each case with both algorithms.
    println!("old {} lines / {} KB, new {} lines", old.len(), old_text.len() / 1024, new.len());
    for (name, algorithm) in [("Myers", Algorithm::Myers), ("Patience", Algorithm::Patience)] {
        let config = || {
            let mut config = TextDiff::configure();
            config.algorithm(algorithm).timeout(timeout);
            config
        };

        time(&format!("{name}: line diff"), || config().diff_lines(&old_text, &new_text).ops().len());

        time(&format!("{name}: line diff + word highlights"), || {
            let diff = config().diff_lines(&old_text, &new_text);
            diff.iter_all_inline_changes_with_options(word_options).count()
        });

        time(&format!("{name}: worst case, unrelated files"), || {
            config().diff_lines(&old_text, &unrelated).ops().len()
        });
    }
}
