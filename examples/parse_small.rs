use medley::ebnf::{grammar, parse_str};
use std::time::Instant;

fn main() {
    let grammar = grammar! {
        expr ::= word op word;
        word ::= letter { letter };
        letter ::= 'a'..'z' | 'A'..'Z';
        op ::= "+" | "-";
    };

    let input = "abc+def-ghi";
    let iterations = 100_000;

    let start = Instant::now();
    for _ in 0..iterations {
        let events: Vec<_> = parse_str(&grammar, input).collect();
        std::hint::black_box(events);
    }
    let elapsed = start.elapsed();

    println!(
        "parse_str_small: {} iterations in {:?}",
        iterations, elapsed
    );
    println!("  Average: {:?} per iteration", elapsed / iterations);
}
