use super::super::*;
use crate::errors::{Error, Result};

impl<'a> Parser<'a> {
    pub(crate) fn parse_interface_declaration(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::Interface)?;
        let name = match self.advance().token {
            Token::Identifier(name) => name,
            t => {
                return Err(Error::ParseError(format!(
                    "Expected interface name, got {:?}",
                    t
                )))
            }
        };
        self.skip_type_parameters();
        let mut extends = Vec::new();
        if self.peek().token == Token::Extends {
            self.advance();
            loop {
                match self.advance().token {
                    Token::Identifier(n) => extends.push(n),
                    t => {
                        return Err(Error::ParseError(format!(
                            "Expected identifier, got {:?}",
                            t
                        )))
                    }
                }
                self.skip_type_parameters();
                if self.peek().token == Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::LeftBrace)?;
        let mut members = Vec::new();
        while self.peek().token != Token::RightBrace && self.peek().token != Token::Eof {
            if self.peek().token == Token::Comma || self.peek().token == Token::Semicolon {
                self.advance();
                continue;
            }
            loop {
                match &self.peek().token {
                    Token::Public | Token::Private | Token::Protected | Token::Static => {
                        self.advance();
                    }
                    Token::Identifier(s) if s == "readonly" => {
                        self.advance();
                    }
                    _ => break,
                }
            }
            let name = self.token_to_key_string()?;
            if self.peek().token == Token::Question {
                self.advance();
            }
            if self.peek().token == Token::LeftParen {
                self.advance();
                let mut params = Vec::new();
                if self.peek().token != Token::RightParen {
                    loop {
                        let pname = match self.advance().token {
                            Token::Identifier(n) => n,
                            t => {
                                return Err(Error::ParseError(format!(
                                    "Expected param name, got {:?}",
                                    t
                                )))
                            }
                        };
                        let ptype = if self.peek().token == Token::Colon {
                            self.advance();
                            self.parse_type_annotation()?
                        } else {
                            TypeAnnotation::Any
                        };
                        params.push((pname, ptype));
                        if self.peek().token == Token::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(&Token::RightParen)?;
                let return_type = if self.peek().token == Token::Colon {
                    self.advance();
                    self.parse_type_annotation()?
                } else {
                    TypeAnnotation::Any
                };
                if self.peek().token == Token::Semicolon {
                    self.advance();
                }
                members.push(InterfaceMember::Method {
                    name,
                    params,
                    return_type,
                });
            } else {
                let optional = if self.peek().token == Token::Question {
                    self.advance();
                    true
                } else {
                    false
                };
                self.expect(&Token::Colon)?;
                let type_annotation = self.parse_type_annotation()?;
                if self.peek().token == Token::Semicolon {
                    self.advance();
                }
                members.push(InterfaceMember::Property {
                    name,
                    type_annotation,
                    optional,
                });
            }
        }
        self.expect(&Token::RightBrace)?;
        if self.peek().token == Token::Semicolon {
            self.advance();
        }
        Ok(self.spanned(Statement::InterfaceDeclaration {
            name,
            extends,
            members,
        }))
    }

    pub(crate) fn parse_type_alias_declaration(&mut self) -> Result<SpannedNode<Statement>> {
        match &self.peek().token {
            Token::Identifier(s) if s == "type" => {
                self.advance();
            }
            _ => return Err(Error::ParseError("Expected 'type' keyword".into())),
        }
        let name = match self.advance().token {
            Token::Identifier(name) => name,
            t => {
                return Err(Error::ParseError(format!(
                    "Expected type name, got {:?}",
                    t
                )))
            }
        };
        self.skip_type_parameters();
        self.expect(&Token::Assign)?;
        let type_annotation = self.parse_type_annotation()?;
        if self.peek().token == Token::Semicolon {
            self.advance();
        }
        Ok(self.spanned(Statement::TypeAliasDeclaration {
            name,
            type_annotation,
        }))
    }

    pub(crate) fn parse_enum_declaration(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::Enum)?;
        let name = match self.advance().token {
            Token::Identifier(name) => name,
            t => {
                return Err(Error::ParseError(format!(
                    "Expected enum name, got {:?}",
                    t
                )))
            }
        };
        self.expect(&Token::LeftBrace)?;
        let mut members = Vec::new();
        while self.peek().token != Token::RightBrace && self.peek().token != Token::Eof {
            let member_name = match self.advance().token {
                Token::Identifier(n) => n,
                t => {
                    return Err(Error::ParseError(format!(
                        "Expected enum member name, got {:?}",
                        t
                    )))
                }
            };
            let value = if self.peek().token == Token::Assign {
                self.advance();
                match self.peek().token.clone() {
                    Token::Number(n) => {
                        self.advance();
                        Some(TypeLiteral::Number(n))
                    }
                    Token::String(s) => {
                        self.advance();
                        Some(TypeLiteral::String(s))
                    }
                    _ => None,
                }
            } else {
                None
            };
            members.push(EnumMember {
                name: member_name,
                value,
            });
            if self.peek().token == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::RightBrace)?;
        if self.peek().token == Token::Semicolon {
            self.advance();
        }
        Ok(self.spanned(Statement::EnumDeclaration { name, members }))
    }
}
