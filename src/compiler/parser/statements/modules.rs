use super::super::*;
use crate::errors::{Error, Result};

impl<'a> Parser<'a> {
    pub(crate) fn parse_import_declaration(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::Import)?;
        if self.peek().token == Token::Identifier("type".to_string()) {
            self.advance();
        }
        let mut specifiers = Vec::new();

        if matches!(self.peek().token, Token::String(_)) {
            let source = match self.advance().token {
                Token::String(s) => s,
                _ => unreachable!(),
            };
            if self.peek().token == Token::Semicolon {
                self.advance();
            }
            return Ok(self.spanned(Statement::ImportDeclaration {
                specifiers: vec![],
                source,
            }));
        }

        let has_default_import = matches!(self.peek().token, Token::Identifier(_));
        let default_local_name = if has_default_import {
            match self.advance().token {
                Token::Identifier(name) => Some(name),
                _ => None,
            }
        } else {
            None
        };
        let has_default = default_local_name.is_some();

        if let Some(default_name) = default_local_name {
            specifiers.push(ImportSpecifier {
                local: default_name.clone(),
                imported: Some("default".to_string()),
            });
        }

        if self.peek().token == Token::Comma {
            self.advance();
            if self.peek().token == Token::LeftBrace {
                self.advance();
                while self.peek().token != Token::RightBrace {
                    let imported = match self.advance().token {
                        Token::Identifier(name) => name,
                        t => {
                            return Err(Error::ParseError(format!(
                                "Expected identifier, got {:?}",
                                t
                            )))
                        }
                    };
                    let local = if self.peek().token == Token::As {
                        self.advance();
                        match self.advance().token {
                            Token::Identifier(name) => name,
                            t => {
                                return Err(Error::ParseError(format!(
                                    "Expected identifier, got {:?}",
                                    t
                                )))
                            }
                        }
                    } else {
                        imported.clone()
                    };
                    specifiers.push(ImportSpecifier {
                        local,
                        imported: Some(imported),
                    });
                    if self.peek().token == Token::Comma {
                        self.advance();
                        if self.peek().token == Token::RightBrace {
                            break;
                        }
                    }
                }
                self.expect(&Token::RightBrace)?;
            }
        } else if self.peek().token == Token::LeftBrace {
            self.advance();
            while self.peek().token != Token::RightBrace {
                let imported = match self.advance().token {
                    Token::Identifier(name) => name,
                    t => {
                        return Err(Error::ParseError(format!(
                            "Expected identifier, got {:?}",
                            t
                        )))
                    }
                };
                let local = if self.peek().token == Token::As {
                    self.advance();
                    match self.advance().token {
                        Token::Identifier(name) => name,
                        t => {
                            return Err(Error::ParseError(format!(
                                "Expected identifier, got {:?}",
                                t
                            )))
                        }
                    }
                } else {
                    imported.clone()
                };
                specifiers.push(ImportSpecifier {
                    local,
                    imported: Some(imported),
                });
                if self.peek().token == Token::Comma {
                    self.advance();
                    if self.peek().token == Token::RightBrace {
                        break;
                    }
                }
            }
            self.expect(&Token::RightBrace)?;
        } else if self.peek().token == Token::Star {
            self.advance();
            self.expect(&Token::As)?;
            let local = match self.advance().token {
                Token::Identifier(name) => name,
                t => {
                    return Err(Error::ParseError(format!(
                        "Expected identifier, got {:?}",
                        t
                    )))
                }
            };
            specifiers.push(ImportSpecifier {
                local,
                imported: Some("*".to_string()),
            });
        } else if !has_default {
            return Err(Error::ParseError("Expected import specifier".into()));
        }

        if self.peek().token == Token::From {
            self.advance();
        } else {
            return Err(Error::ParseError("Expected 'from' keyword".into()));
        }
        let source = match self.advance().token {
            Token::String(s) => s,
            t => return Err(Error::ParseError(format!("Expected string, got {:?}", t))),
        };
        if self.peek().token == Token::Semicolon {
            self.advance();
        }
        Ok(self.spanned(Statement::ImportDeclaration { specifiers, source }))
    }

    pub(crate) fn parse_export_declaration(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::Export)?;

        if self.peek().token == Token::Identifier("type".to_string()) {
            let next_is_ident = self
                .tokens
                .get(self.pos + 1)
                .map(|t| matches!(t.token, Token::Identifier(_)))
                .unwrap_or(false);
            if next_is_ident {
                let type_alias = self.parse_type_alias_declaration()?;
                return Ok(self.spanned(Statement::ExportDeclaration {
                    kind: ExportDeclarationKind::Local(Box::new(type_alias)),
                }));
            }
        }

        if self.peek().token == Token::Default {
            self.advance();
            let decl = match self.peek().token {
                Token::Function => {
                    let mut look_ahead = self.pos + 1;
                    if self
                        .tokens
                        .get(look_ahead)
                        .map(|t| t.token == Token::Star)
                        .unwrap_or(false)
                    {
                        look_ahead += 1;
                    }
                    if self
                        .tokens
                        .get(look_ahead)
                        .map(|t| t.token == Token::LeftParen)
                        .unwrap_or(false)
                        || self
                            .tokens
                            .get(look_ahead)
                            .map(|t| t.token == Token::LeftBrace)
                            .unwrap_or(false)
                    {
                        let expr = self.parse_expression()?;
                        if self.peek().token == Token::Semicolon {
                            self.advance();
                        }
                        self.spanned(Statement::Expression(expr.inner))
                    } else {
                        self.parse_statement()?
                    }
                }
                Token::Class | Token::Const | Token::Let | Token::Var => self.parse_statement()?,
                _ => {
                    let expr = self.parse_expression()?;
                    if self.peek().token == Token::Semicolon {
                        self.advance();
                    }
                    self.spanned(Statement::Expression(expr.inner))
                }
            };
            return Ok(self.spanned(Statement::ExportDefaultDeclaration {
                declaration: Box::new(decl),
            }));
        }

        if self.peek().token == Token::Star {
            self.advance();
            if self.peek().token == Token::As {
                self.advance();
                let alias = self.advance_as_ident();
                self.expect(&Token::From)?;
                let source = match self.advance().token {
                    Token::String(s) => s,
                    t => {
                        return Err(Error::ParseError(format!(
                            "Expected string literal after 'from', got {:?}",
                            t
                        )))
                    }
                };
                if self.peek().token == Token::Semicolon {
                    self.advance();
                }
                return Ok(self.spanned(Statement::ExportDeclaration {
                    kind: ExportDeclarationKind::ReExport {
                        specifiers: vec![ExportSpecifier {
                            local: "*".to_string(),
                            exported: Some(alias),
                        }],
                        source,
                    },
                }));
            } else {
                self.expect(&Token::From)?;
                let source = match self.advance().token {
                    Token::String(s) => s,
                    t => {
                        return Err(Error::ParseError(format!(
                            "Expected string literal after 'from', got {:?}",
                            t
                        )))
                    }
                };
                if self.peek().token == Token::Semicolon {
                    self.advance();
                }
                return Ok(self.spanned(Statement::ExportDeclaration {
                    kind: ExportDeclarationKind::ReExport {
                        specifiers: vec![ExportSpecifier {
                            local: "*".to_string(),
                            exported: Some("*".to_string()),
                        }],
                        source,
                    },
                }));
            }
        }

        if self.peek().token == Token::LeftBrace {
            self.advance();
            let mut specifiers = Vec::new();
            while self.peek().token != Token::RightBrace {
                if self.peek().token == Token::Comma {
                    self.advance();
                    continue;
                }
                let local = self.advance_as_ident();
                let exported = if self.peek().token == Token::As {
                    self.advance();
                    Some(self.advance_as_ident())
                } else {
                    None
                };
                specifiers.push(ExportSpecifier { local, exported });
                if self.peek().token == Token::Comma {
                    self.advance();
                }
            }
            self.expect(&Token::RightBrace)?;

            if self.peek().token == Token::From {
                self.advance();
                let source = match self.advance().token {
                    Token::String(s) => s,
                    t => {
                        return Err(Error::ParseError(format!(
                            "Expected string literal after 'from', got {:?}",
                            t
                        )))
                    }
                };
                if self.peek().token == Token::Semicolon {
                    self.advance();
                }
                return Ok(self.spanned(Statement::ExportDeclaration {
                    kind: ExportDeclarationKind::ReExport { specifiers, source },
                }));
            }

            if self.peek().token == Token::Semicolon {
                self.advance();
            }
            return Ok(self.spanned(Statement::ExportDeclaration {
                kind: ExportDeclarationKind::ReExport {
                    specifiers,
                    source: String::new(),
                },
            }));
        }

        let decl = self.parse_statement()?;
        Ok(self.spanned(Statement::ExportDeclaration {
            kind: ExportDeclarationKind::Local(Box::new(decl)),
        }))
    }
}
