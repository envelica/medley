use medley::ebnf::{grammar, Grammar, ParseEvent, Parser, TokenKind};

fn eval_expr(grammar: &Grammar, input: &str) -> Result<i64, String> {
    let mut num_buf = String::new();
    let mut nums: Vec<i64> = Vec::new();
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
                    op = Some(ch);
                }
            }
            ParseEvent::End { rule } if rule == "num" => {
                // Number ended - convert accumulated digits
                if !num_buf.is_empty() {
                    let num: i64 = num_buf.parse().map_err(|e| format!("invalid number '{num_buf}': {e}"))?;
                    nums.push(num);
                    num_buf.clear();
                }
            }
            ParseEvent::Error(_) => {
                // Flush any remaining number
                if !num_buf.is_empty() {
                    let num: i64 = num_buf.parse().map_err(|e| format!("invalid number '{num_buf}': {e}"))?;
                    nums.push(num);
                    num_buf.clear();
                }
                break;
            }
            _ => {}
        }
    }

    if nums.len() == 1 {
        return Ok(nums[0]);
    } else if nums.len() == 2 {
        let left = nums[0];
        let right = nums[1];
        let op = op.ok_or_else(|| "operator missing".to_string())?;

        let result = match op {
            '+' => left + right,
            '-' => left - right,
            _ => return Err(format!("unsupported operator: {op}")),
        };

        Ok(result)
    } else {
        Err(format!("expected 1 or 2 numbers, got {}", nums.len()))
    }
}

fn main() {
    let grammar = grammar! {
        expr = num op num;
        num = digit digit?;
        digit = [0-9];
        op = [+-];
    };

    let input = "12+30";
    let value = eval_expr(&grammar, input).expect("evaluation failed");
    println!("{} = {}", input, value);
}
