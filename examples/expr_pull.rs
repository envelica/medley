use medley::ebnf::{grammar, Grammar, ParseEvent, Parser, TokenKind};

fn eval_expr(grammar: &Grammar, input: &str) -> Result<i64, String> {
    let mut num_buf = String::new();
    let mut left: Option<i64> = None;
    let mut op: Option<char> = None;

    let mut parser = Parser::from_str(grammar, input);
    while let Some(event) = parser.next_event() {
        match event {
            ParseEvent::Token { kind, .. } => {
                let ch = match kind {
                    TokenKind::Char(c) | TokenKind::Class(c) => c,
                    TokenKind::Str(s) => s.chars().next().unwrap_or('+'),
                };

                if ch.is_ascii_digit() {
                    num_buf.push(ch);
                } else if ch == '+' || ch == '-' {
                    if left.is_none() {
                        left = Some(num_buf.parse().map_err(|e| format!("invalid number '{num_buf}': {e}"))?);
                    }
                    num_buf.clear();
                    op = Some(ch);
                }
            }
            ParseEvent::Error(_) => break,
            _ => {}
        }
    }

    let right: i64 = num_buf
        .parse()
        .map_err(|e| format!("invalid number '{num_buf}': {e}"))?;

    let left = left.ok_or_else(|| "left operand missing".to_string())?;
    let op = op.ok_or_else(|| "operator missing".to_string())?;

    let result = match op {
        '+' => left + right,
        '-' => left - right,
        _ => return Err(format!("unsupported operator: {op}")),
    };

    Ok(result)
}

fn main() {
    let grammar = grammar! {
        expr = [0123456789+-]+;
    };

    let input = "12+30";
    let value = eval_expr(&grammar, input).expect("evaluation failed");
    println!("{} = {}", input, value);
}
