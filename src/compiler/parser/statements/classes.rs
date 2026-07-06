use super::super::*;
use crate::errors::{Error, Result};

impl<'a> Parser<'a> {
    pub(crate) fn parse_class_declaration(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::Class)?;
        let name = match self.advance().token {
            Token::Identifier(name) => name,
            t => {
                return Err(Error::ParseError(format!(
                    "Expected class name, got {:?}",
                    t
                )))
            }
        };
        self.skip_type_parameters();
        let superclass = if self.peek().token == Token::Extends {
            self.advance();
            Some(self.parse_call()?.inner)
        } else {
            None
        };
        if let Token::Identifier(s) = &self.peek().token {
            if s == "implements" {
                self.advance();
                while self.peek().token != Token::LeftBrace && self.peek().token != Token::Eof {
                    self.advance();
                    if self.peek().token == Token::Comma {
                        self.advance();
                    }
                }
            }
        }
        self.expect(&Token::LeftBrace)?;
        let body = self.parse_class_body()?;
        self.expect(&Token::RightBrace)?;
        Ok(self.spanned(Statement::ClassDeclaration {
            name,
            superclass: superclass.map(Box::new),
            body,
        }))
    }

    pub(crate) fn parse_class_body(&mut self) -> Result<Vec<ClassMember>> {
        let mut members = Vec::new();
        while self.peek().token != Token::RightBrace && self.peek().token != Token::Eof {
            let is_static = if self.peek().token == Token::Static {
                self.advance();
                true
            } else {
                false
            };
            let is_async = if self.peek().token == Token::Async {
                self.advance();
                true
            } else {
                false
            };
            loop {
                match &self.peek().token {
                    Token::Public | Token::Private | Token::Protected => {
                        self.advance();
                    }
                    Token::Identifier(s) if s == "readonly" => {
                        self.advance();
                    }
                    _ => break,
                }
            }

            if self.peek().token == Token::Constructor {
                self.advance();
                self.expect(&Token::LeftParen)?;
                let params = self.parse_constructor_params()?;
                self.expect(&Token::RightParen)?;
                if self.peek().token == Token::Colon {
                    self.advance();
                    self.parse_type_annotation()?;
                }
                self.expect(&Token::LeftBrace)?;
                let body = self.parse_block_body()?;
                self.expect(&Token::RightBrace)?;
                members.push(ClassMember::Constructor { params, body });
            } else if self.peek().token == Token::Get && !is_async {
                let is_method = self.pos + 1 < self.tokens.len()
                    && self.tokens[self.pos + 1].token == Token::LeftParen;
                if is_method {
                    let name = "get".to_string();
                    self.advance();
                    self.advance();
                    let (params, param_types, _defaults, _rest_param) =
                        self.parse_typed_params()?;
                    self.expect(&Token::RightParen)?;
                    let return_type = if self.peek().token == Token::Colon {
                        self.advance();
                        Some(self.parse_type_annotation()?)
                    } else {
                        None
                    };
                    if self.peek().token == Token::Semicolon {
                        self.advance();
                    } else {
                        self.expect(&Token::LeftBrace)?;
                        let body = self.parse_block_body()?;
                        self.expect(&Token::RightBrace)?;
                        members.push(ClassMember::Method {
                            name,
                            params,
                            param_types: Some(param_types),
                            return_type,
                            body,
                            is_static,
                            is_async,
                        });
                    }
                } else {
                    self.advance();
                    let name = match self.advance().token {
                        Token::Identifier(name) => name,
                        t => {
                            return Err(Error::ParseError(format!(
                                "Expected property name after 'get', got {:?}",
                                t
                            )))
                        }
                    };
                    self.expect(&Token::LeftParen)?;
                    self.expect(&Token::RightParen)?;
                    let return_type = if self.peek().token == Token::Colon {
                        self.advance();
                        Some(self.parse_type_annotation()?)
                    } else {
                        None
                    };
                    self.expect(&Token::LeftBrace)?;
                    let body = self.parse_block_body()?;
                    self.expect(&Token::RightBrace)?;
                    members.push(ClassMember::Getter {
                        name,
                        return_type,
                        body,
                        is_static,
                    });
                }
            } else if self.peek().token == Token::Set && !is_async {
                let is_method = self.pos + 1 < self.tokens.len()
                    && self.tokens[self.pos + 1].token == Token::LeftParen;
                if is_method {
                    let name = "set".to_string();
                    self.advance();
                    self.advance();
                    let (params, param_types, _defaults, _rest_param) =
                        self.parse_typed_params()?;
                    self.expect(&Token::RightParen)?;
                    let return_type = if self.peek().token == Token::Colon {
                        self.advance();
                        Some(self.parse_type_annotation()?)
                    } else {
                        None
                    };
                    if self.peek().token == Token::Semicolon {
                        self.advance();
                    } else {
                        self.expect(&Token::LeftBrace)?;
                        let body = self.parse_block_body()?;
                        self.expect(&Token::RightBrace)?;
                        members.push(ClassMember::Method {
                            name,
                            params,
                            param_types: Some(param_types),
                            return_type,
                            body,
                            is_static,
                            is_async,
                        });
                    }
                } else {
                    self.advance();
                    let name = match self.advance().token {
                        Token::Identifier(name) => name,
                        t => {
                            return Err(Error::ParseError(format!(
                                "Expected property name after 'set', got {:?}",
                                t
                            )))
                        }
                    };
                    let (param, param_type) = {
                        self.expect(&Token::LeftParen)?;
                        let pname = match self.advance().token {
                            Token::Identifier(n) => n,
                            t => {
                                return Err(Error::ParseError(format!(
                                    "Expected parameter name, got {:?}",
                                    t
                                )))
                            }
                        };
                        let ptype = if self.peek().token == Token::Colon {
                            self.advance();
                            Some(self.parse_type_annotation()?)
                        } else {
                            None
                        };
                        (pname, ptype)
                    };
                    self.expect(&Token::RightParen)?;
                    if self.peek().token == Token::Colon {
                        self.advance();
                        self.parse_type_annotation()?;
                    }
                    self.expect(&Token::LeftBrace)?;
                    let body = self.parse_block_body()?;
                    self.expect(&Token::RightBrace)?;
                    members.push(ClassMember::Setter {
                        name,
                        param,
                        param_type,
                        body,
                        is_static,
                    });
                }
            } else {
                let name = match self.advance().token {
                    Token::Identifier(name) => name,
                    Token::String(name) => name,
                    Token::Number(n) => n.to_string(),
                    t => {
                        return Err(Error::ParseError(format!(
                            "Expected method name, got {:?}",
                            t
                        )))
                    }
                };
                if self.peek().token == Token::Less {
                    self.advance();
                    let mut depth = 1;
                    while depth > 0 && self.peek().token != Token::Eof {
                        match self.peek().token {
                            Token::Less => depth += 1,
                            Token::Greater => depth -= 1,
                            _ => {}
                        }
                        self.advance();
                    }
                }
                if self.peek().token == Token::LeftParen {
                    self.advance();
                    let (params, param_types, _defaults, _rest_param) =
                        self.parse_typed_params()?;
                    self.expect(&Token::RightParen)?;
                    let return_type = if self.peek().token == Token::Colon {
                        self.advance();
                        Some(self.parse_type_annotation()?)
                    } else {
                        None
                    };
                    if self.peek().token == Token::Semicolon {
                        self.advance();
                    } else {
                        self.expect(&Token::LeftBrace)?;
                        let body = self.parse_block_body()?;
                        self.expect(&Token::RightBrace)?;
                        members.push(ClassMember::Method {
                            name,
                            params,
                            param_types: Some(param_types),
                            return_type,
                            body,
                            is_static,
                            is_async,
                        });
                    }
                } else {
                    if self.peek().token == Token::Colon {
                        self.advance();
                        self.parse_type_annotation()?;
                    }
                    let init = if self.peek().token == Token::Assign {
                        self.advance();
                        Some(self.parse_expression()?.inner)
                    } else {
                        None
                    };
                    members.push(ClassMember::Property {
                        name,
                        is_static,
                        init,
                    });
                    if self.peek().token == Token::Semicolon {
                        self.advance();
                    }
                }
            }
        }
        Ok(members)
    }
}
