use super::*;
use crate::errors::{Error, Result};

impl<'a> Parser<'a> {
    pub(crate) fn parse_variable_declaration(&mut self) -> Result<SpannedNode<Statement>> {
        let kind = match self.advance().token {
            Token::Var => VarKind::Var,
            Token::Let => VarKind::Let,
            Token::Const => VarKind::Const,
            _ => unreachable!(),
        };
        let mut declarations = Vec::new();
        loop {
            let id = self.parse_binding_pattern()?;
            let type_annotation = if self.peek().token == Token::Colon {
                self.advance();
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
            let init = if self.peek().token == Token::Assign {
                self.advance();
                Some(self.parse_expression()?.inner)
            } else {
                None
            };
            declarations.push(VariableDeclarator {
                id,
                type_annotation,
                init,
            });
            if self.peek().token == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::Semicolon)?;
        Ok(self.spanned(Statement::VariableDeclaration { kind, declarations }))
    }

    pub(crate) fn parse_binding_pattern(&mut self) -> Result<BindingPattern> {
        match self.peek().token.clone() {
            Token::LeftBracket => self.parse_array_binding_pattern(),
            Token::LeftBrace => self.parse_object_binding_pattern(),
            _ => {
                let id = match self.advance().token {
                    Token::Identifier(name) => name,
                    token => {
                        return Err(Error::ParseError(format!(
                            "Expected identifier or pattern, got {:?}",
                            token
                        )))
                    }
                };
                Ok(BindingPattern::Identifier(id))
            }
        }
    }

    pub(crate) fn parse_array_binding_pattern(&mut self) -> Result<BindingPattern> {
        self.expect(&Token::LeftBracket)?;
        let mut elements = Vec::new();
        if self.peek().token != Token::RightBracket {
            loop {
                if self.peek().token == Token::Comma {
                    elements.push(ArrayBindingElement::Skip);
                    self.advance();
                    continue;
                }
                if self.peek().token == Token::Ellipsis {
                    self.advance();
                    let rest = self.parse_binding_pattern()?;
                    elements.push(ArrayBindingElement::Rest(Box::new(rest)));
                    break;
                }
                let pattern = self.parse_binding_pattern()?;
                let default = if self.peek().token == Token::Assign {
                    self.advance();
                    Some(self.parse_expression()?.inner)
                } else {
                    None
                };
                elements.push(ArrayBindingElement::Pattern(pattern, Box::new(default)));
                if self.peek().token == Token::Comma {
                    self.advance();
                    if self.peek().token == Token::RightBracket {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::RightBracket)?;
        Ok(BindingPattern::Array(elements))
    }

    pub(crate) fn parse_object_binding_pattern(&mut self) -> Result<BindingPattern> {
        self.expect(&Token::LeftBrace)?;
        let mut elements = Vec::new();
        if self.peek().token != Token::RightBrace {
            loop {
                if self.peek().token == Token::Ellipsis {
                    self.advance();
                    let rest = self.parse_binding_pattern()?;
                    elements.push(ObjectBindingElement {
                        key: match &rest {
                            BindingPattern::Identifier(name) => name.clone(),
                            _ => {
                                return Err(Error::ParseError(
                                    "Invalid rest pattern in object".into(),
                                ))
                            }
                        },
                        value: rest,
                        shorthand: true,
                        default_value: None,
                    });
                    break;
                }
                let key = match self.advance().token {
                    Token::Identifier(name) => name,
                    token => {
                        return Err(Error::ParseError(format!(
                            "Expected property name, got {:?}",
                            token
                        )))
                    }
                };
                if self.peek().token == Token::Colon {
                    self.advance();
                    let value = self.parse_binding_pattern()?;
                    let default = if self.peek().token == Token::Assign {
                        self.advance();
                        Some(self.parse_expression()?.inner)
                    } else {
                        None
                    };
                    elements.push(ObjectBindingElement {
                        key: key.clone(),
                        value,
                        shorthand: false,
                        default_value: default,
                    });
                } else if self.peek().token == Token::Assign {
                    self.advance();
                    let default_value = self.parse_expression()?.inner;
                    elements.push(ObjectBindingElement {
                        key: key.clone(),
                        value: BindingPattern::Identifier(key),
                        shorthand: true,
                        default_value: Some(default_value),
                    });
                } else {
                    elements.push(ObjectBindingElement {
                        key: key.clone(),
                        value: BindingPattern::Identifier(key),
                        shorthand: true,
                        default_value: None,
                    });
                }
                if self.peek().token == Token::Comma {
                    self.advance();
                    if self.peek().token == Token::RightBrace {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::RightBrace)?;
        Ok(BindingPattern::Object(elements))
    }

    pub(crate) fn parse_function_declaration(&mut self) -> Result<SpannedNode<Statement>> {
        let is_async = if self.peek().token == Token::Async {
            self.advance();
            true
        } else {
            false
        };
        self.expect(&Token::Function)?;
        let is_generator = if self.peek().token == Token::Star {
            self.advance();
            true
        } else {
            false
        };
        let name = match self.advance().token {
            Token::Identifier(name) => name,
            token => {
                return Err(Error::ParseError(format!(
                    "Expected function name, got {:?}",
                    token
                )))
            }
        };
        if self.peek().token == Token::Less {
            self.advance();
            while self.peek().token != Token::Greater && self.peek().token != Token::Eof {
                if self.peek().token == Token::Comma {
                    self.advance();
                    continue;
                }
                self.advance();
            }
            self.expect(&Token::Greater)?;
        }
        self.expect(&Token::LeftParen)?;
        let (params, param_types, defaults, rest_param) = self.parse_typed_params()?;
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
        Ok(self.spanned(Statement::FunctionDeclaration {
            name,
            params,
            param_types: Some(param_types),
            defaults,
            rest_param,
            return_type,
            body,
            is_async,
            is_generator,
        }))
    }

    pub(crate) fn parse_return_statement(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::Return)?;
        let value =
            if self.peek().token != Token::Semicolon && self.peek().token != Token::RightBrace {
                Some(self.parse_expression()?.inner)
            } else {
                None
            };
        self.expect(&Token::Semicolon)?;
        Ok(self.spanned(Statement::ReturnStatement(value)))
    }

    pub(crate) fn parse_yield_statement(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::Yield)?;
        let value =
            if self.peek().token != Token::Semicolon && self.peek().token != Token::RightBrace {
                Some(self.parse_expression()?.inner)
            } else {
                None
            };
        self.expect(&Token::Semicolon)?;
        Ok(self.spanned(Statement::YieldStatement(value)))
    }

    pub(crate) fn parse_if_statement(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::If)?;
        self.expect(&Token::LeftParen)?;
        let condition = self.parse_expression()?.inner;
        self.expect(&Token::RightParen)?;
        let consequent = Box::new(self.parse_statement()?);
        let alternate = if self.peek().token == Token::Else {
            self.advance();
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        Ok(self.spanned(Statement::IfStatement {
            condition,
            consequent,
            alternate,
        }))
    }

    pub(crate) fn parse_while_statement(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::While)?;
        self.expect(&Token::LeftParen)?;
        let condition = self.parse_expression()?.inner;
        self.expect(&Token::RightParen)?;
        let body = Box::new(self.parse_statement()?);
        Ok(self.spanned(Statement::WhileStatement { condition, body }))
    }

    pub(crate) fn parse_block_statement(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::LeftBrace)?;
        let body = self.parse_block_body()?;
        self.expect(&Token::RightBrace)?;
        Ok(self.spanned(Statement::BlockStatement(body)))
    }

    pub(crate) fn parse_for_statement(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::For)?;

        let is_for_await = if self.peek().token == Token::Await {
            self.advance();
            true
        } else {
            false
        };

        self.expect(&Token::LeftParen)?;

        if self.peek().token == Token::Semicolon {
            self.advance();
            let condition = if self.peek().token != Token::Semicolon {
                Some(self.parse_expression()?.inner)
            } else {
                None
            };
            self.expect(&Token::Semicolon)?;
            let update = if self.peek().token != Token::RightParen {
                Some(self.parse_expression()?.inner)
            } else {
                None
            };
            self.expect(&Token::RightParen)?;
            let body = Box::new(self.parse_statement()?);
            return Ok(self.spanned(Statement::ForStatement {
                init: None,
                condition,
                update,
                body,
            }));
        }

        if self.peek().token == Token::Let
            || self.peek().token == Token::Const
            || self.peek().token == Token::Var
        {
            let kind = match self.peek().token {
                Token::Var => VarKind::Var,
                Token::Let => VarKind::Let,
                Token::Const => VarKind::Const,
                _ => unreachable!(),
            };
            self.advance();
            let id = self.parse_binding_pattern()?;
            if self.peek().token == Token::Colon {
                self.advance();
                self.parse_type_annotation()?;
            }
            if self.peek().token == Token::In {
                self.advance();
                let right = self.parse_expression()?.inner;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(self.spanned(Statement::ForInStatement {
                    left: ForInLeft::VariableDeclaration { kind, id },
                    right,
                    body,
                }));
            }
            if self.peek().token == Token::Of {
                self.advance();
                let right = self.parse_expression()?.inner;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(self.spanned(Statement::ForOfStatement {
                    left: ForInLeft::VariableDeclaration { kind, id },
                    right,
                    body,
                    is_async: is_for_await,
                }));
            }
            if !matches!(id, BindingPattern::Identifier(_)) {
                return Err(Error::ParseError("Expected assignment in for-loop".into()));
            }
            let mut declarations = Vec::new();
            let decl_id = id;
            let init_val = if self.peek().token == Token::Assign {
                self.advance();
                Some(self.parse_expression()?.inner)
            } else {
                None
            };
            declarations.push(VariableDeclarator {
                id: decl_id,
                type_annotation: None,
                init: init_val,
            });
            let init = Some(Box::new(ForInit::Variable(Box::new(
                self.spanned(Statement::VariableDeclaration { kind, declarations }),
            ))));
            self.expect(&Token::Semicolon)?;
            let condition = if self.peek().token != Token::Semicolon {
                Some(self.parse_expression()?.inner)
            } else {
                None
            };
            self.expect(&Token::Semicolon)?;
            let update = if self.peek().token != Token::RightParen {
                Some(self.parse_expression()?.inner)
            } else {
                None
            };
            self.expect(&Token::RightParen)?;
            let body = Box::new(self.parse_statement()?);
            return Ok(self.spanned(Statement::ForStatement {
                init,
                condition,
                update,
                body,
            }));
        }

        if let Token::Identifier(id) = self.peek().token.clone() {
            self.advance();
            if self.peek().token == Token::In {
                self.advance();
                let right = self.parse_expression()?.inner;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(self.spanned(Statement::ForInStatement {
                    left: ForInLeft::Identifier(id),
                    right,
                    body,
                }));
            }
            if self.peek().token == Token::Of {
                self.advance();
                let right = self.parse_expression()?.inner;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(self.spanned(Statement::ForOfStatement {
                    left: ForInLeft::Identifier(id),
                    right,
                    body,
                    is_async: is_for_await,
                }));
            }
            self.pos -= 1;
        }

        if matches!(self.peek().token, Token::LeftBracket | Token::LeftBrace) {
            let pattern = self.parse_binding_pattern()?;
            if self.peek().token == Token::In {
                self.advance();
                let right = self.parse_expression()?.inner;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(self.spanned(Statement::ForInStatement {
                    left: ForInLeft::Pattern(pattern),
                    right,
                    body,
                }));
            }
            if self.peek().token == Token::Of {
                self.advance();
                let right = self.parse_expression()?.inner;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(self.spanned(Statement::ForOfStatement {
                    left: ForInLeft::Pattern(pattern),
                    right,
                    body,
                    is_async: is_for_await,
                }));
            }
            return Err(Error::ParseError(
                "Expected 'in' or 'of' after destructuring pattern in for-loop".into(),
            ));
        }

        let init_expr = self.parse_expression()?.inner;
        let init = Some(Box::new(ForInit::Expression(init_expr)));
        self.expect(&Token::Semicolon)?;
        let condition = if self.peek().token != Token::Semicolon {
            Some(self.parse_expression()?.inner)
        } else {
            None
        };
        self.expect(&Token::Semicolon)?;
        let update = if self.peek().token != Token::RightParen {
            Some(self.parse_expression()?.inner)
        } else {
            None
        };
        self.expect(&Token::RightParen)?;
        let body = Box::new(self.parse_statement()?);
        Ok(self.spanned(Statement::ForStatement {
            init,
            condition,
            update,
            body,
        }))
    }

    pub(crate) fn parse_do_while_statement(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::Do)?;
        let body = Box::new(self.parse_statement()?);
        self.expect(&Token::While)?;
        self.expect(&Token::LeftParen)?;
        let condition = self.parse_expression()?.inner;
        self.expect(&Token::RightParen)?;
        if self.peek().token == Token::Semicolon {
            self.advance();
        }
        Ok(self.spanned(Statement::DoWhileStatement { condition, body }))
    }

    pub(crate) fn parse_switch_statement(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::Switch)?;
        self.expect(&Token::LeftParen)?;
        let discriminant = self.parse_expression()?.inner;
        self.expect(&Token::RightParen)?;
        self.expect(&Token::LeftBrace)?;
        let mut cases = Vec::new();
        while self.peek().token != Token::RightBrace && self.peek().token != Token::Eof {
            let test = if self.peek().token == Token::Case {
                self.advance();
                Some(self.parse_expression()?.inner)
            } else {
                self.expect(&Token::Default)?;
                None
            };
            self.expect(&Token::Colon)?;
            let mut consequent = Vec::new();
            while self.peek().token != Token::Case
                && self.peek().token != Token::Default
                && self.peek().token != Token::RightBrace
                && self.peek().token != Token::Eof
            {
                consequent.push(self.parse_statement()?);
            }
            cases.push(SwitchCase { test, consequent });
        }
        self.expect(&Token::RightBrace)?;
        Ok(self.spanned(Statement::SwitchStatement {
            discriminant,
            cases,
        }))
    }

    pub(crate) fn parse_try_statement(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::Try)?;
        self.expect(&Token::LeftBrace)?;
        let block = self.parse_block_body()?;
        self.expect(&Token::RightBrace)?;

        let handler = if self.peek().token == Token::Catch {
            self.advance();
            // Optional catch binding: catch { } or catch (e) { }
            let param = if self.peek().token == Token::LeftParen {
                self.advance();
                let p = match self.advance().token {
                    Token::Identifier(name) => name,
                    t => {
                        return Err(Error::ParseError(format!(
                            "Expected parameter, got {:?}",
                            t
                        )))
                    }
                };
                if self.peek().token == Token::Colon {
                    self.advance();
                    self.parse_type_annotation()?;
                }
                self.expect(&Token::RightParen)?;
                p
            } else {
                // Auto-generate a catch variable name
                "__catch_err".to_string()
            };
            self.expect(&Token::LeftBrace)?;
            let body = self.parse_block_body()?;
            self.expect(&Token::RightBrace)?;
            Some(CatchClause { param, body })
        } else {
            None
        };

        let finalizer = if self.peek().token == Token::Finally {
            self.advance();
            self.expect(&Token::LeftBrace)?;
            let body = self.parse_block_body()?;
            self.expect(&Token::RightBrace)?;
            Some(body)
        } else {
            None
        };

        Ok(self.spanned(Statement::TryStatement {
            block,
            handler,
            finalizer,
        }))
    }

    pub(crate) fn parse_throw_statement(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::Throw)?;
        let argument = self.parse_expression()?.inner;
        self.expect(&Token::Semicolon)?;
        Ok(self.spanned(Statement::ThrowStatement(argument)))
    }

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
            while matches!(
                self.peek().token,
                Token::Public | Token::Private | Token::Protected | Token::Readonly
            ) {
                self.advance();
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
            } else if self.peek().token == Token::Set && !is_async {
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
            } else {
                let name = match self.advance().token {
                    Token::Identifier(name) => name,
                    t => {
                        return Err(Error::ParseError(format!(
                            "Expected method name, got {:?}",
                            t
                        )))
                    }
                };
                // Consume optional generic type parameters: method<T>(...)
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

    pub(crate) fn parse_import_declaration(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::Import)?;
        // Skip 'type' keyword for type-only imports: import type { ... } from "..."
        if self.peek().token == Token::Type {
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

        // Handle 'export type': parse as type alias export
        if self.peek().token == Token::Type {
            let type_alias = self.parse_type_alias_declaration()?;
            return Ok(self.spanned(Statement::ExportDeclaration {
                kind: ExportDeclarationKind::Local(Box::new(type_alias)),
            }));
        }

        if self.peek().token == Token::Default {
            self.advance();
            // export default can be followed by a declaration or an expression
            let decl = match self.peek().token {
                Token::Function | Token::Class | Token::Const | Token::Let | Token::Var => {
                    self.parse_statement()?
                }
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

        if self.peek().token == Token::LeftBrace {
            self.advance();
            let mut specifiers = Vec::new();
            while self.peek().token != Token::RightBrace {
                if self.peek().token == Token::Comma {
                    self.advance();
                    continue;
                }
                let local = match self.advance().token {
                    Token::Identifier(name) => name,
                    t => {
                        return Err(Error::ParseError(format!(
                            "Expected identifier, got {:?}",
                            t
                        )))
                    }
                };
                let exported = if self.peek().token == Token::As {
                    self.advance();
                    match self.advance().token {
                        Token::Identifier(name) => Some(name),
                        t => {
                            return Err(Error::ParseError(format!(
                                "Expected identifier after 'as', got {:?}",
                                t
                            )))
                        }
                    }
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
            // Skip optional modifiers like 'readonly', 'public', 'private', 'protected', 'static'
            while matches!(
                self.peek().token,
                Token::Readonly | Token::Public | Token::Private | Token::Protected | Token::Static
            ) {
                self.advance();
            }
            let name = match self.advance().token {
                Token::Identifier(n) => n,
                t => {
                    return Err(Error::ParseError(format!(
                        "Expected property name, got {:?}",
                        t
                    )))
                }
            };
            // Skip optional marker for methods: setup?(): void
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
        self.expect(&Token::Type)?;
        let name = match self.advance().token {
            Token::Identifier(name) => name,
            t => {
                return Err(Error::ParseError(format!(
                    "Expected type name, got {:?}",
                    t
                )))
            }
        };
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
