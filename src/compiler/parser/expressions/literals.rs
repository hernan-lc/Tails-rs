use super::super::*;
use crate::errors::{Error, Result};
use crate::compiler::lexer::TemplatePart;

impl<'a> Parser<'a> {
    pub(crate) fn parse_literal(&mut self) -> Result<SpannedNode<Expression>> {
        match self.peek().token.clone() {
            Token::Number(n) => {
                self.advance();
                Ok(self.spanned(Expression::NumberLiteral(n)))
            }
            Token::BigInt(ref s) => {
                let s = s.clone();
                self.advance();
                Ok(self.spanned(Expression::BigIntLiteral(s)))
            }
            Token::String(s) => {
                self.advance();
                Ok(self.spanned(Expression::StringLiteral(s)))
            }
            Token::Regex(s) => {
                self.advance();
                let (pattern, flags) = match s.rfind('/') {
                    Some(pos) => (s[..pos].to_string(), s[pos + 1..].to_string()),
                    None => (s.clone(), String::new()),
                };
                Ok(self.spanned(Expression::RegexLiteral { pattern, flags }))
            }
            Token::TemplateLiteral(parts) => {
                self.advance();
                self.parse_template_literal(parts)
            }
            _ => Err(Error::ParseError(format!(
                "Expected literal, got {:?}",
                self.peek().token
            ))),
        }
    }

    pub(crate) fn parse_identifier_or_keyword(&mut self) -> Result<SpannedNode<Expression>> {
        match self.peek().token.clone() {
            Token::Identifier(name) => {
                self.advance();
                match name.as_str() {
                    "true" => Ok(self.spanned(Expression::BooleanLiteral(true))),
                    "false" => Ok(self.spanned(Expression::BooleanLiteral(false))),
                    "null" => Ok(self.spanned(Expression::NullLiteral)),
                    "undefined" => Ok(self.spanned(Expression::UndefinedLiteral)),
                    "NaN" => Ok(self.spanned(Expression::NaNLiteral)),
                    "Infinity" => Ok(self.spanned(Expression::InfinityLiteral)),
                    _ => {
                        if self.peek().token == Token::Arrow {
                            self.advance();
                            self.parse_arrow_body(vec![name], None, vec![], None, None, false)
                        } else {
                            Ok(self.spanned(Expression::Identifier(name)))
                        }
                    }
                }
            }
            _ => self.parse_keyword_as_identifier(),
        }
    }

    pub(crate) fn parse_keyword_as_identifier(
        &mut self,
    ) -> Result<SpannedNode<Expression>> {
        let token = self.peek().token.clone();
        let name = match &token {
            Token::Set => Some("set"),
            Token::Get => Some("get"),
            Token::Delete => Some("delete"),
            Token::Typeof => Some("typeof"),
            Token::Void => Some("void"),
            Token::New => Some("new"),
            Token::Return => Some("return"),
            Token::If => Some("if"),
            Token::Else => Some("else"),
            Token::While => Some("while"),
            Token::For => Some("for"),
            Token::Do => Some("do"),
            Token::Switch => Some("switch"),
            Token::Case => Some("case"),
            Token::Break => Some("break"),
            Token::Continue => Some("continue"),
            Token::Try => Some("try"),
            Token::Catch => Some("catch"),
            Token::Finally => Some("finally"),
            Token::Throw => Some("throw"),
            Token::Const => Some("const"),
            Token::Let => Some("let"),
            Token::Var => Some("var"),
            Token::In => Some("in"),
            Token::Of => Some("of"),
            Token::Instanceof => Some("instanceof"),
            Token::Extends => Some("extends"),
            Token::Static => Some("static"),
            Token::Public => Some("public"),
            Token::Private => Some("private"),
            Token::Protected => Some("protected"),
            Token::Enum => Some("enum"),
            Token::Interface => Some("interface"),
            Token::Yield => Some("yield"),
            Token::Await => Some("await"),
            Token::Constructor => Some("constructor"),
            Token::From => Some("from"),
            Token::As => Some("as"),
            Token::Default => Some("default"),
            Token::Import => Some("import"),
            Token::Export => Some("export"),
            Token::Function => Some("function"),
            Token::Class => Some("class"),
            Token::Super => Some("super"),
            _ => None,
        };
        if let Some(name) = name {
            self.advance();
            Ok(self.spanned(Expression::Identifier(name.to_string())))
        } else {
            Err(Error::ParseError(format!(
                "Unexpected token {:?} at {}:{}",
                token, self.current_span.line, self.current_span.col
            )))
        }
    }

    pub(crate) fn parse_template_literal(
        &mut self,
        parts: Vec<TemplatePart>,
    ) -> Result<SpannedNode<Expression>> {
        let mut quasis = Vec::new();
        let mut expressions = Vec::new();
        let mut text_buf = String::new();
        for part in parts {
            match part {
                TemplatePart::Text(t) => text_buf.push_str(&t),
                TemplatePart::Expression(expr_tokens) => {
                    quasis.push(text_buf.clone());
                    text_buf.clear();
                    let mut owned_tokens = expr_tokens.clone();
                    let mut sub_parser = Parser::new(&mut owned_tokens);
                    let expr = sub_parser.parse_expression()?;
                    expressions.push(expr.inner);
                }
            }
        }
        quasis.push(text_buf);
        Ok(self.spanned(Expression::TemplateLiteral {
            quasis,
            expressions,
        }))
    }
}
