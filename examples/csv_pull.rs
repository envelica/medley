use medley::ebnf::{grammar, parse, Grammar, ParseEvent, TokenKind};
use std::io::Cursor;

fn stream_csv_line(grammar: &Grammar, line: &str) {
    let mut field_buf = String::new();
    let mut record: Vec<String> = Vec::new();

    for event in parse(grammar, Cursor::new(line.as_bytes())) {
        match event {
            ParseEvent::Token { kind, .. } => {
                let ch = match kind {
                    TokenKind::Char(c) | TokenKind::Class(c) => c,
                    TokenKind::Str(s) => s.chars().next().unwrap_or(','),
                };
                
                // Skip comma delimiters
                if ch != ',' {
                    field_buf.push(ch);
                }
            }
            ParseEvent::End { rule } if rule == "field" => {
                // Field ended, save it
                if !field_buf.is_empty() {
                    record.push(field_buf.clone());
                    field_buf.clear();
                }
            }
            _ => {}
        }
    }

    if !record.is_empty() {
        println!("record: {:?}", record);
    }
}

fn main() {
    let grammar = grammar! {
        record = field (',' field)*;
        field = word+;
        word = [a-z] | [A-Z] | [0-9];
    };

    let data = "alpha,beta,gamma\n1,2,3\nx,y,z\n";
    for line in data.lines() {
        stream_csv_line(&grammar, line);
    }
}
