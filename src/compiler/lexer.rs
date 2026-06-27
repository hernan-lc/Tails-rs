use crate::errors::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    String(String),
    BigInt(String),
    Regex(String),
    Identifier(String),
    TemplateLiteral(Vec<TemplatePart>),
    Const,
    Let,
    Var,
    Function,
    Return,
    If,
    Else,
    While,
    For,
    Do,
    Switch,
    Case,
    Break,
    Continue,
    New,
    Void,
    Delete,
    Typeof,
    Instanceof,
    Constructor,
    In,
    Of,
    Class,
    Extends,
    Super,
    This,
    Import,
    Export,
    Default,
    From,
    As,
    Type,
    Interface,
    Enum,
    Async,
    Await,
    Promise,
    Try,
    Catch,
    Finally,
    Throw,
    Static,
    Get,
    Set,
    Yield,
    Public,
    Private,
    Protected,
    Readonly,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Power,
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,
    PowerAssign,
    AndAssign,
    OrAssign,
    Equal,
    StrictEqual,
    NotEqual,
    StrictNotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    Not,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
    Increment,
    Decrement,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Colon,
    Comma,
    Dot,
    Question,
    QuestionDot,
    NullishCoalescing,
    Arrow,
    Ellipsis,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    Text(String),
    Expression(Vec<Token>),
}

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
                    let filtered: Vec<Token> = inner_tokens
                        .into_iter()
                        .filter(|t| *t != Token::Eof)
                        .collect();
                    parts.push(TemplatePart::Expression(filtered));
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

pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();

    while let Some(&(_pos, ch)) = chars.peek() {
        match ch {
            ' ' | '\t' | '\r' => {
                chars.next();
            }
            '\n' => {
                chars.next();
            }
            '/' => {
                chars.next();
                if let Some(&(_, '/')) = chars.peek() {
                    while let Some(&(_, c)) = chars.peek() {
                        if c == '\n' {
                            break;
                        }
                        chars.next();
                    }
                } else if let Some(&(_, '*')) = chars.peek() {
                    chars.next();
                    loop {
                        match chars.next() {
                            Some((_, '*')) => {
                                if let Some(&(_, '/')) = chars.peek() {
                                    chars.next();
                                    break;
                                }
                            }
                            None => return Err(Error::ParseError("Unterminated comment".into())),
                            _ => {}
                        }
                    }
                } else if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::SlashAssign);
                } else {
                    tokens.push(Token::Slash);
                }
            }
            '0'..='9' => {
                let mut num = String::new();
                while let Some(&(_, c)) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(num.parse().unwrap_or(0.0)));
            }
            'a'..='z' | 'A'..='Z' | '_' | '$' => {
                let mut ident = String::new();
                while let Some(&(_, c)) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '$' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "const" => tokens.push(Token::Const),
                    "let" => tokens.push(Token::Let),
                    "var" => tokens.push(Token::Var),
                    "function" => tokens.push(Token::Function),
                    "return" => tokens.push(Token::Return),
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "while" => tokens.push(Token::While),
                    "for" => tokens.push(Token::For),
                    "do" => tokens.push(Token::Do),
                    "switch" => tokens.push(Token::Switch),
                    "case" => tokens.push(Token::Case),
                    "break" => tokens.push(Token::Break),
                    "continue" => tokens.push(Token::Continue),
                    "new" => tokens.push(Token::New),
                    "void" => tokens.push(Token::Void),
                    "delete" => tokens.push(Token::Delete),
                    "typeof" => tokens.push(Token::Typeof),
                    "instanceof" => tokens.push(Token::Instanceof),
                    "in" => tokens.push(Token::In),
                    "of" => tokens.push(Token::Of),
                    "class" => tokens.push(Token::Class),
                    "extends" => tokens.push(Token::Extends),
                    "super" => tokens.push(Token::Super),
                    "this" => tokens.push(Token::This),
                    "import" => tokens.push(Token::Import),
                    "export" => tokens.push(Token::Export),
                    "default" => tokens.push(Token::Default),
                    "from" => tokens.push(Token::From),
                    "as" => tokens.push(Token::As),
                    "type" => tokens.push(Token::Type),
                    "interface" => tokens.push(Token::Interface),
                    "enum" => tokens.push(Token::Enum),
                    "async" => tokens.push(Token::Async),
                    "await" => tokens.push(Token::Await),
                    "yield" => tokens.push(Token::Yield),
                    "try" => tokens.push(Token::Try),
                    "catch" => tokens.push(Token::Catch),
                    "finally" => tokens.push(Token::Finally),
                    "throw" => tokens.push(Token::Throw),
                    "static" => tokens.push(Token::Static),
                    "public" => tokens.push(Token::Public),
                    "private" => tokens.push(Token::Private),
                    "protected" => tokens.push(Token::Protected),
                    "readonly" => tokens.push(Token::Readonly),
                    "constructor" => tokens.push(Token::Constructor),
                    "get" => tokens.push(Token::Get),
                    "set" => tokens.push(Token::Set),
                    "true" => tokens.push(Token::Identifier("true".into())),
                    "false" => tokens.push(Token::Identifier("false".into())),
                    "null" => tokens.push(Token::Identifier("null".into())),
                    "undefined" => tokens.push(Token::Identifier("undefined".into())),
                    _ => tokens.push(Token::Identifier(ident)),
                }
            }
            '`' => {
                chars.next();
                let parts = tokenize_template_literal(&mut chars)?;
                tokens.push(Token::TemplateLiteral(parts));
            }
            '"' | '\'' => {
                let quote = ch;
                chars.next();
                let mut str = String::new();
                loop {
                    match chars.next() {
                        Some((_, c)) if c == quote => break,
                        Some((_, '\\')) => {
                            if let Some((_, c)) = chars.next() {
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
                        Some((_, c)) => str.push(c),
                        None => return Err(Error::ParseError("Unterminated string".into())),
                    }
                }
                tokens.push(Token::String(str));
            }
            '+' => {
                chars.next();
                if let Some(&(_, '+')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::Increment);
                } else if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::PlusAssign);
                } else {
                    tokens.push(Token::Plus);
                }
            }
            '-' => {
                chars.next();
                if let Some(&(_, '-')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::Decrement);
                } else if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::MinusAssign);
                } else {
                    tokens.push(Token::Minus);
                }
            }
            '*' => {
                chars.next();
                if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::StarAssign);
                } else if let Some(&(_, '*')) = chars.peek() {
                    chars.next();
                    if let Some(&(_, '=')) = chars.peek() {
                        chars.next();
                        tokens.push(Token::PowerAssign);
                    } else {
                        tokens.push(Token::Power);
                    }
                } else {
                    tokens.push(Token::Star);
                }
            }
            '%' => {
                chars.next();
                if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::PercentAssign);
                } else {
                    tokens.push(Token::Percent);
                }
            }
            '(' => {
                chars.next();
                tokens.push(Token::LeftParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RightParen);
            }
            '{' => {
                chars.next();
                tokens.push(Token::LeftBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RightBrace);
            }
            '[' => {
                chars.next();
                tokens.push(Token::LeftBracket);
            }
            ']' => {
                chars.next();
                tokens.push(Token::RightBracket);
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            ':' => {
                chars.next();
                tokens.push(Token::Colon);
            }
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            }
            '.' => {
                chars.next();
                if let Some(&(_, '.')) = chars.peek() {
                    chars.next();
                    if let Some(&(_, '.')) = chars.peek() {
                        chars.next();
                        tokens.push(Token::Ellipsis);
                    } else {
                        tokens.push(Token::Dot);
                        tokens.push(Token::Dot);
                    }
                } else {
                    tokens.push(Token::Dot);
                }
            }
            '?' => {
                chars.next();
                if let Some(&(_, '.')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::QuestionDot);
                } else if let Some(&(_, '?')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::NullishCoalescing);
                } else {
                    tokens.push(Token::Question);
                }
            }
            '=' => {
                chars.next();
                if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    if let Some(&(_, '=')) = chars.peek() {
                        chars.next();
                        tokens.push(Token::StrictEqual);
                    } else {
                        tokens.push(Token::Equal);
                    }
                } else if let Some(&(_, '>')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::Arrow);
                } else {
                    tokens.push(Token::Assign);
                }
            }
            '!' => {
                chars.next();
                if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    if let Some(&(_, '=')) = chars.peek() {
                        chars.next();
                        tokens.push(Token::StrictNotEqual);
                    } else {
                        tokens.push(Token::NotEqual);
                    }
                } else {
                    tokens.push(Token::Not);
                }
            }
            '<' => {
                chars.next();
                if let Some(&(_, '<')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::ShiftLeft);
                } else if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::LessEqual);
                } else {
                    tokens.push(Token::Less);
                }
            }
            '>' => {
                chars.next();
                if let Some(&(_, '>')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::ShiftRight);
                } else if let Some(&(_, '=')) = chars.peek() {
                    chars.next();
                    tokens.push(Token::GreaterEqual);
                } else {
                    tokens.push(Token::Greater);
                }
            }
            '&' => {
                chars.next();
                if let Some(&(_, '&')) = chars.peek() {
                    chars.next();
                    if let Some(&(_, '=')) = chars.peek() {
                        chars.next();
                        tokens.push(Token::AndAssign);
                    } else {
                        tokens.push(Token::And);
                    }
                } else {
                    tokens.push(Token::BitAnd);
                }
            }
            '|' => {
                chars.next();
                if let Some(&(_, '|')) = chars.peek() {
                    chars.next();
                    if let Some(&(_, '=')) = chars.peek() {
                        chars.next();
                        tokens.push(Token::OrAssign);
                    } else {
                        tokens.push(Token::Or);
                    }
                } else {
                    tokens.push(Token::BitOr);
                }
            }
            '^' => {
                chars.next();
                tokens.push(Token::BitXor);
            }
            '~' => {
                chars.next();
                tokens.push(Token::BitNot);
            }
            _ => {
                chars.next();
            }
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}
