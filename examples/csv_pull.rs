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

                match ch {
                    ',' => {
                        record.push(field_buf.clone());
                        field_buf.clear();
                    }
                    _ => field_buf.push(ch),
                }
            }
            ParseEvent::Error(_) => continue,
            _ => {}
        }
    }

    if !field_buf.is_empty() {
        record.push(field_buf);
    }
    if !record.is_empty() {
        println!("record: {:?}", record);
    }
}

fn main() {
    let grammar = grammar! {
        record = [abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789,]+;
    };

    let data = "alpha,beta,gamma\n1,2,3\nx,y,z\n";
    for line in data.lines() {
        stream_csv_line(&grammar, line);
    }
}
