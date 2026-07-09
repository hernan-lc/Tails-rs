mod token;
pub use token::*;

use crate::errors::{Error, Result};
use crate::well_known as wk;

fn tokenize_template_literal(
    chars: &mut std::iter::Peekable<std::str::CharIndices>,
) -> Result<Vec<TemplatePart>> {
    let mut parts = Vec::new();
    let mut text_buf = String::new();

    loop {
        match chars.next() {
            Some((_, '`')) => {
                if !text_buf.is_empty() {
                    parts.push(TemplatePart::Text(text_buf.clone()));
                    text_buf.clear();
                }
                return Ok(parts);
            }
            Some((_, '$')) => {
                if let Some(&(_, '{')) = chars.peek() {
                    chars.next();
                    if !text_buf.is_empty() {
                        parts.push(TemplatePart::Text(text_buf.clone()));
                        text_buf.clear();
                    }
                    let mut depth = 1u32;
                    let mut expr_src = String::new();
                    loop {
                        match chars.next() {
                            Some((_, '{')) => {
                                depth += 1;
                                expr_src.push('{');
                            }
                            Some((_, '}')) => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                                expr_src.push('}');
                            }
                            Some((_, c)) => expr_src.push(c),
                            None => {
                                return Err(Error::ParseError(
                                    "Unterminated template expression".into(),
                                ))
                            }
                        }
                    }
                    let inner_tokens = tokenize(&expr_src)?;
                    // Keep the trailing Eof: the parser's `advance` relies on a
                    // final non-repeatable token to terminate loops. Removing
                    // it (as we previously did) causes infinite loops on
                    // template expressions whose last token is a loop-matched
                    // one like `++` (e.g. `` `key_${counter++}` ``).
                    parts.push(TemplatePart::Expression(inner_tokens));
                } else {
                    text_buf.push('$');
                }
            }
            Some((_, '\\')) => {
                if let Some((_, c)) = chars.next() {
                    match c {
                        'n' => text_buf.push('\n'),
                        't' => text_buf.push('\t'),
                        'r' => text_buf.push('\r'),
                        '\\' => text_buf.push('\\'),
                        '\'' => text_buf.push('\''),
                        '"' => text_buf.push('"'),
                        '`' => text_buf.push('`'),
                        '$' => text_buf.push('$'),
                        _ => {
                            text_buf.push('\\');
                            text_buf.push(c);
                        }
                    }
                }
            }
            Some((_, c)) => text_buf.push(c),
            None => return Err(Error::ParseError("Unterminated template literal".into())),
        }
    }
}

pub fn tokenize(source: &str) -> Result<Vec<SpannedToken>> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();
    let mut expects_regex = true;
    let mut line: usize = 1;
    let mut col: usize = 1;

    let push =
        |tokens: &mut Vec<SpannedToken>, token: Token, line: usize, col: usize, offset: usize| {
            tokens.push(SpannedToken {
                token,
                span: crate::errors::Span::new(line, col, offset),
            });
        };

    while let Some(&(pos, ch)) = chars.peek() {
        match ch {
            ' ' | '\t' | '\r' => {
                chars.next();
                col += 1;
            }
            '\n' => {
                chars.next();
                line += 1;
                col = 1;
            }
            '/' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '/')) = chars.peek() {
                    while let Some(&(_, c)) = chars.peek() {
                        if c == '\n' {
                            break;
                        }
                        chars.next();
                        col += 1;
                    }
                } else if let Some(&(_, '*')) = chars.peek() {
                    chars.next();
                    col += 1;
                    loop {
                        match chars.next() {
                            Some((_, '*')) => {
                                col += 1;
                                if let Some(&(_, '/')) = chars.peek() {
                                    chars.next();
                                    col += 1;
                                    break;
                                }
                            }
                            Some((_, '\n')) => {
                                line += 1;
                                col = 1;
                            }
                            None => return Err(Error::ParseError("Unterminated comment".into())),
                            Some(_) => {
                                col += 1;
                            }
                        }
                    }
                } else if expects_regex {
                    let mut pattern = String::new();
                    let mut escaped = false;
                    let mut bracket_depth = 0;
                    loop {
                        match chars.next() {
                            Some((_, '\\')) if !escaped => {
                                // Start of an escape sequence; the following
                                // character is processed in the `escaped` arm.
                                escaped = true;
                                col += 1;
                            }
                            Some((_, c)) if escaped => {
                                match c {
                                    '\\' => pattern.push('\\'),
                                    'n' => pattern.push('\n'),
                                    't' => pattern.push('\t'),
                                    'r' => pattern.push('\r'),
                                    'f' => pattern.push('\x0c'),
                                    'v' => pattern.push('\x0b'),
                                    '0' => pattern.push('\0'),
                                    'x' => {
                                        let mut hex = String::new();
                                        for _ in 0..2 {
                                            if let Some(&(_, hc)) = chars.peek() {
                                                if hc.is_ascii_hexdigit() {
                                                    hex.push(hc);
                                                    chars.next();
                                                    col += 1;
                                                } else {
                                                    break;
                                                }
                                            } else {
                                                break;
                                            }
                                        }
                                        match u8::from_str_radix(&hex, 16) {
                                            Ok(b) => pattern.push(b as char),
                                            Err(_) => pattern.push('x'),
                                        }
                                    }
                                    'u' => {
                                        let mut hex = String::new();
                                        for _ in 0..4 {
                                            if let Some(&(_, hc)) = chars.peek() {
                                                if hc.is_ascii_hexdigit() {
                                                    hex.push(hc);
                                                    chars.next();
                                                    col += 1;
                                                } else {
                                                    break;
                                                }
                                            } else {
                                                break;
                                            }
                                        }
                                        match u32::from_str_radix(&hex, 16) {
                                            Ok(cp) => {
                                                if let Some(ch) = char::from_u32(cp) {
                                                    pattern.push(ch);
                                                } else {
                                                    pattern.push('u');
                                                }
                                            }
                                            Err(_) => pattern.push('u'),
                                        }
                                    }
                                    // Everything else (e.g. \d \w \. \( \* ) is a
                                    // regex metacharacter: keep the backslash so
                                    // the engine interprets the escape.
                                    other => {
                                        pattern.push('\\');
                                        pattern.push(other);
                                    }
                                }
                                escaped = false;
                                col += 1;
                            }
                            Some((_, '[')) if !escaped && bracket_depth == 0 => {
                                pattern.push('[');
                                bracket_depth = 1;
                                col += 1;
                                escaped = false;
                            }
                            Some((_, ']')) if !escaped && bracket_depth > 0 => {
                                pattern.push(']');
                                bracket_depth -= 1;
                                col += 1;
                                escaped = false;
                            }
                            Some((_, '/')) if !escaped && bracket_depth == 0 => {
                                col += 1;
                                break;
                            }
                            Some((_, '\n')) => {
                                pattern.push('\n');
                                line += 1;
                                col = 1;
                                escaped = false;
                            }
                            Some((_, c)) => {
                                pattern.push(c);
                                col += 1;
                                escaped = false;
                            }
                            None => return Err(Error::ParseError("Unterminated regex".into())),
                        }
                    }
                    let mut flags = String::new();
                    while let Some(&(_, c)) = chars.peek() {
                        if c.is_ascii_alphabetic() {
                            flags.push(c);
                            chars.next();
                            col += 1;
                        } else {
                            break;
                        }
                    }
                    push(
                        &mut tokens,
                        Token::Regex(format!("{}/{}", pattern, flags)),
                        tok_line,
                        tok_col,
                        tok_offset,
                    );
                    expects_regex = false;
                } else if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(
                        &mut tokens,
                        Token::SlashAssign,
                        tok_line,
                        tok_col,
                        tok_offset,
                    );
                    expects_regex = false;
                } else {
                    push(&mut tokens, Token::Slash, tok_line, tok_col, tok_offset);
                    expects_regex = true;
                }
            }
            '0'..='9' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                let mut num = String::new();
                while let Some(&(_, c)) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num.push(c);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }
                if let Some(&(_, 'e' | 'E')) = chars.peek() {
                    num.push(chars.next().unwrap().1);
                    col += 1;
                    if let Some(&(_, '+' | '-')) = chars.peek() {
                        num.push(chars.next().unwrap().1);
                        col += 1;
                    }
                    while let Some(&(_, c)) = chars.peek() {
                        if c.is_ascii_digit() {
                            num.push(c);
                            chars.next();
                            col += 1;
                        } else {
                            break;
                        }
                    }
                }
                if let Some(&(_, 'n')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(
                        &mut tokens,
                        Token::BigInt(num),
                        tok_line,
                        tok_col,
                        tok_offset,
                    );
                } else {
                    push(
                        &mut tokens,
                        Token::Number(num.parse().unwrap_or(0.0)),
                        tok_line,
                        tok_col,
                        tok_offset,
                    );
                }
            }
            'a'..='z' | 'A'..='Z' | '_' | '$' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                let mut ident = String::new();
                while let Some(&(_, c)) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '$' {
                        ident.push(c);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }
                let token = match ident.as_str() {
                    "const" => Token::Const,
                    "let" => Token::Let,
                    "var" => Token::Var,
                    "function" => Token::Function,
                    "return" => Token::Return,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "while" => Token::While,
                    "for" => Token::For,
                    "do" => Token::Do,
                    "switch" => Token::Switch,
                    "case" => Token::Case,
                    "break" => Token::Break,
                    "continue" => Token::Continue,
                    "new" => Token::New,
                    "void" => Token::Void,
                    "delete" => Token::Delete,
                    "typeof" => Token::Typeof,
                    "instanceof" => Token::Instanceof,
                    "in" => Token::In,
                    "of" => Token::Of,
                    "class" => Token::Class,
                    "extends" => Token::Extends,
                    "super" => Token::Super,
                    "this" => Token::This,
                    "import" => Token::Import,
                    "export" => Token::Export,
                    "default" => Token::Default,
                    "from" => Token::From,
                    "as" => Token::As,
                    "type" => Token::Identifier("type".to_string()),
                    "interface" => Token::Interface,
                    "enum" => Token::Enum,
                    "async" => Token::Async,
                    "await" => Token::Await,
                    "yield" => Token::Yield,
                    "try" => Token::Try,
                    wk::CATCH => Token::Catch,
                    wk::FINALLY => Token::Finally,
                    "throw" => Token::Throw,
                    "static" => Token::Static,
                    "public" => Token::Public,
                    "private" => Token::Private,
                    "protected" => Token::Protected,
                    "readonly" => Token::Identifier("readonly".to_string()),
                    wk::CONSTRUCTOR => Token::Constructor,
                    "get" => Token::Get,
                    "set" => Token::Set,
                    wk::TRUE => Token::Identifier(wk::TRUE.into()),
                    wk::FALSE => Token::Identifier(wk::FALSE.into()),
                    wk::NULL => Token::Identifier(wk::NULL.into()),
                    wk::UNDEFINED => Token::Identifier(wk::UNDEFINED.into()),
                    _ => Token::Identifier(ident),
                };
                push(&mut tokens, token, tok_line, tok_col, tok_offset);
            }
            '`' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                let parts = tokenize_template_literal(&mut chars)?;
                push(
                    &mut tokens,
                    Token::TemplateLiteral(parts),
                    tok_line,
                    tok_col,
                    tok_offset,
                );
            }
            '"' | '\'' => {
                let quote = ch;
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                let mut str = String::new();
                loop {
                    match chars.next() {
                        Some((_, c)) if c == quote => {
                            col += 1;
                            break;
                        }
                        Some((_, '\\')) => {
                            col += 1;
                            if let Some((_, c)) = chars.next() {
                                col += 1;
                                match c {
                                    'n' => str.push('\n'),
                                    't' => str.push('\t'),
                                    'r' => str.push('\r'),
                                    '\\' => str.push('\\'),
                                    '\'' => str.push('\''),
                                    '"' => str.push('"'),
                                    '`' => str.push('`'),
                                    _ => {
                                        str.push('\\');
                                        str.push(c);
                                    }
                                }
                            }
                        }
                        Some((_, '\n')) => {
                            str.push('\n');
                            line += 1;
                            col = 1;
                        }
                        Some((_, c)) => {
                            str.push(c);
                            col += 1;
                        }
                        None => return Err(Error::ParseError("Unterminated string".into())),
                    }
                }
                push(
                    &mut tokens,
                    Token::String(str),
                    tok_line,
                    tok_col,
                    tok_offset,
                );
            }
            '+' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '+')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(&mut tokens, Token::Increment, tok_line, tok_col, tok_offset);
                } else if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(
                        &mut tokens,
                        Token::PlusAssign,
                        tok_line,
                        tok_col,
                        tok_offset,
                    );
                } else {
                    push(&mut tokens, Token::Plus, tok_line, tok_col, tok_offset);
                }
            }
            '-' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '-')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(&mut tokens, Token::Decrement, tok_line, tok_col, tok_offset);
                } else if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(
                        &mut tokens,
                        Token::MinusAssign,
                        tok_line,
                        tok_col,
                        tok_offset,
                    );
                } else {
                    push(&mut tokens, Token::Minus, tok_line, tok_col, tok_offset);
                }
            }
            '*' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(
                        &mut tokens,
                        Token::StarAssign,
                        tok_line,
                        tok_col,
                        tok_offset,
                    );
                } else if let Some(&(_, '*')) = chars.peek() {
                    chars.next();
                    col += 1;
                    if let Some(&(_, '=')) = chars.peek() {
                        chars.next();
                        col += 1;
                        push(
                            &mut tokens,
                            Token::PowerAssign,
                            tok_line,
                            tok_col,
                            tok_offset,
                        );
                    } else {
                        push(&mut tokens, Token::Power, tok_line, tok_col, tok_offset);
                    }
                } else {
                    push(&mut tokens, Token::Star, tok_line, tok_col, tok_offset);
                }
            }
            '%' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(
                        &mut tokens,
                        Token::PercentAssign,
                        tok_line,
                        tok_col,
                        tok_offset,
                    );
                } else {
                    push(&mut tokens, Token::Percent, tok_line, tok_col, tok_offset);
                }
            }
            '(' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                push(&mut tokens, Token::LeftParen, tok_line, tok_col, tok_offset);
            }
            ')' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                push(
                    &mut tokens,
                    Token::RightParen,
                    tok_line,
                    tok_col,
                    tok_offset,
                );
            }
            '{' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                push(&mut tokens, Token::LeftBrace, tok_line, tok_col, tok_offset);
            }
            '}' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                push(
                    &mut tokens,
                    Token::RightBrace,
                    tok_line,
                    tok_col,
                    tok_offset,
                );
            }
            '[' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                push(
                    &mut tokens,
                    Token::LeftBracket,
                    tok_line,
                    tok_col,
                    tok_offset,
                );
            }
            ']' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                push(
                    &mut tokens,
                    Token::RightBracket,
                    tok_line,
                    tok_col,
                    tok_offset,
                );
            }
            ';' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                push(&mut tokens, Token::Semicolon, tok_line, tok_col, tok_offset);
            }
            ':' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                push(&mut tokens, Token::Colon, tok_line, tok_col, tok_offset);
            }
            ',' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                push(&mut tokens, Token::Comma, tok_line, tok_col, tok_offset);
            }
            '.' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '.')) = chars.peek() {
                    chars.next();
                    col += 1;
                    if let Some(&(_, '.')) = chars.peek() {
                        chars.next();
                        col += 1;
                        push(&mut tokens, Token::Ellipsis, tok_line, tok_col, tok_offset);
                    } else {
                        push(&mut tokens, Token::Dot, tok_line, tok_col, tok_offset);
                        push(&mut tokens, Token::Dot, line, col - 1, pos + 1);
                    }
                } else {
                    push(&mut tokens, Token::Dot, tok_line, tok_col, tok_offset);
                }
            }
            '?' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '.')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(
                        &mut tokens,
                        Token::QuestionDot,
                        tok_line,
                        tok_col,
                        tok_offset,
                    );
                } else if let Some(&(_, '?')) = chars.peek() {
                    chars.next();
                    col += 1;
                    if let Some(&(_, '=')) = chars.peek() {
                        chars.next();
                        col += 1;
                        push(
                            &mut tokens,
                            Token::NullishCoalescingAssign,
                            tok_line,
                            tok_col,
                            tok_offset,
                        );
                    } else {
                        push(
                            &mut tokens,
                            Token::NullishCoalescing,
                            tok_line,
                            tok_col,
                            tok_offset,
                        );
                    }
                } else {
                    push(&mut tokens, Token::Question, tok_line, tok_col, tok_offset);
                }
            }
            '=' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    col += 1;
                    if let Some(&(_, '=')) = chars.peek() {
                        chars.next();
                        col += 1;
                        push(
                            &mut tokens,
                            Token::StrictEqual,
                            tok_line,
                            tok_col,
                            tok_offset,
                        );
                    } else {
                        push(&mut tokens, Token::Equal, tok_line, tok_col, tok_offset);
                    }
                } else if let Some(&(_, '>')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(&mut tokens, Token::Arrow, tok_line, tok_col, tok_offset);
                } else {
                    push(&mut tokens, Token::Assign, tok_line, tok_col, tok_offset);
                }
            }
            '!' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    col += 1;
                    if let Some(&(_, '=')) = chars.peek() {
                        chars.next();
                        col += 1;
                        push(
                            &mut tokens,
                            Token::StrictNotEqual,
                            tok_line,
                            tok_col,
                            tok_offset,
                        );
                    } else {
                        push(&mut tokens, Token::NotEqual, tok_line, tok_col, tok_offset);
                    }
                } else {
                    push(&mut tokens, Token::Not, tok_line, tok_col, tok_offset);
                }
            }
            '<' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '<')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(&mut tokens, Token::ShiftLeft, tok_line, tok_col, tok_offset);
                } else if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(&mut tokens, Token::LessEqual, tok_line, tok_col, tok_offset);
                } else {
                    push(&mut tokens, Token::Less, tok_line, tok_col, tok_offset);
                }
            }
            '>' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '>')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(
                        &mut tokens,
                        Token::ShiftRight,
                        tok_line,
                        tok_col,
                        tok_offset,
                    );
                } else if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(
                        &mut tokens,
                        Token::GreaterEqual,
                        tok_line,
                        tok_col,
                        tok_offset,
                    );
                } else {
                    push(&mut tokens, Token::Greater, tok_line, tok_col, tok_offset);
                }
            }
            '&' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '&')) = chars.peek() {
                    chars.next();
                    col += 1;
                    if let Some(&(_, '=')) = chars.peek() {
                        chars.next();
                        col += 1;
                        push(&mut tokens, Token::AndAssign, tok_line, tok_col, tok_offset);
                    } else {
                        push(&mut tokens, Token::And, tok_line, tok_col, tok_offset);
                    }
                } else if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(
                        &mut tokens,
                        Token::BitAndAssign,
                        tok_line,
                        tok_col,
                        tok_offset,
                    );
                } else {
                    push(&mut tokens, Token::BitAnd, tok_line, tok_col, tok_offset);
                }
            }
            '|' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '|')) = chars.peek() {
                    chars.next();
                    col += 1;
                    if let Some(&(_, '=')) = chars.peek() {
                        chars.next();
                        col += 1;
                        push(&mut tokens, Token::OrAssign, tok_line, tok_col, tok_offset);
                    } else {
                        push(&mut tokens, Token::Or, tok_line, tok_col, tok_offset);
                    }
                } else if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(
                        &mut tokens,
                        Token::BitOrAssign,
                        tok_line,
                        tok_col,
                        tok_offset,
                    );
                } else {
                    push(&mut tokens, Token::BitOr, tok_line, tok_col, tok_offset);
                }
            }
            '^' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    col += 1;
                    push(&mut tokens, Token::XorAssign, tok_line, tok_col, tok_offset);
                } else {
                    push(&mut tokens, Token::BitXor, tok_line, tok_col, tok_offset);
                }
            }
            '~' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                push(&mut tokens, Token::BitNot, tok_line, tok_col, tok_offset);
            }
            '#' => {
                let tok_line = line;
                let tok_col = col;
                let tok_offset = pos;
                chars.next();
                col += 1;
                // Private field: #name → Identifier("#name")
                let mut ident = String::from("#");
                while let Some(&(_, c)) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '$' {
                        ident.push(c);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }
                push(
                    &mut tokens,
                    Token::Identifier(ident),
                    tok_line,
                    tok_col,
                    tok_offset,
                );
            }
            _ => {
                chars.next();
                col += 1;
            }
        }

        if let Some(last) = tokens.last() {
            expects_regex = !matches!(
                last.token,
                Token::Number(_)
                    | Token::BigInt(_)
                    | Token::String(_)
                    | Token::Regex(_)
                    | Token::Identifier(_)
                    | Token::RightParen
                    | Token::RightBracket
                    | Token::RightBrace
                    | Token::Increment
                    | Token::Decrement
                    | Token::This
            );
        }
    }

    push(&mut tokens, Token::Eof, line, col, source.len());
    Ok(tokens)
}
