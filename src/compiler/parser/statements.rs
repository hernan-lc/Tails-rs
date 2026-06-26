use super::*;
use crate::errors::{Error, Result};

impl<'a> Parser<'a> {
    pub(crate) fn parse_variable_declaration(&mut self) -> Result<Statement> {
        let kind = match self.advance() {
            Token::Var => VarKind::Var,
            Token::Let => VarKind::Let,
            Token::Const => VarKind::Const,
            _ => unreachable!(),
        };
        let mut declarations = Vec::new();
        loop {
            let id = match self.advance() {
                Token::Identifier(name) => name,
                token => return Err(Error::ParseError(format!("Expected identifier, got {:?}", token))),
            };
            let type_annotation = if self.peek() == &Token::Colon {
                self.advance();
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
            let init = if self.peek() == &Token::Assign {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            declarations.push(VariableDeclarator { id, type_annotation, init });
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::Semicolon)?;
        Ok(Statement::VariableDeclaration { kind, declarations })
    }

    pub(crate) fn parse_function_declaration(&mut self) -> Result<Statement> {
        let is_async = if self.peek() == &Token::Async {
            self.advance();
            true
        } else {
            false
        };
        self.expect(&Token::Function)?;
        let name = match self.advance() {
            Token::Identifier(name) => name,
            token => return Err(Error::ParseError(format!("Expected function name, got {:?}", token))),
        };
        if self.peek() == &Token::Less {
            self.advance();
            while self.peek() != &Token::Greater && self.peek() != &Token::Eof {
                if self.peek() == &Token::Comma { self.advance(); continue; }
                self.advance();
            }
            self.expect(&Token::Greater)?;
        }
        self.expect(&Token::LeftParen)?;
        let (params, param_types) = self.parse_typed_params()?;
        self.expect(&Token::RightParen)?;
        let return_type = if self.peek() == &Token::Colon {
            self.advance();
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        self.expect(&Token::LeftBrace)?;
        let body = self.parse_block_body()?;
        self.expect(&Token::RightBrace)?;
        Ok(Statement::FunctionDeclaration { name, params, param_types: Some(param_types), return_type, body, is_async })
    }

    pub(crate) fn parse_return_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::Return)?;
        let value = if self.peek() != &Token::Semicolon && self.peek() != &Token::RightBrace {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.expect(&Token::Semicolon)?;
        Ok(Statement::ReturnStatement(value))
    }

    pub(crate) fn parse_if_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::If)?;
        self.expect(&Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(&Token::RightParen)?;
        let consequent = Box::new(self.parse_statement()?);
        let alternate = if self.peek() == &Token::Else {
            self.advance();
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        Ok(Statement::IfStatement { condition, consequent, alternate })
    }

    pub(crate) fn parse_while_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::While)?;
        self.expect(&Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(&Token::RightParen)?;
        let body = Box::new(self.parse_statement()?);
        Ok(Statement::WhileStatement { condition, body })
    }

    pub(crate) fn parse_block_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::LeftBrace)?;
        let body = self.parse_block_body()?;
        self.expect(&Token::RightBrace)?;
        Ok(Statement::BlockStatement(body))
    }

    pub(crate) fn parse_for_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::For)?;
        self.expect(&Token::LeftParen)?;

        if self.peek() == &Token::Semicolon {
            self.advance();
            let condition = if self.peek() != &Token::Semicolon { Some(self.parse_expression()?) } else { None };
            self.expect(&Token::Semicolon)?;
            let update = if self.peek() != &Token::RightParen { Some(self.parse_expression()?) } else { None };
            self.expect(&Token::RightParen)?;
            let body = Box::new(self.parse_statement()?);
            return Ok(Statement::ForStatement { init: None, condition, update, body });
        }

        if self.peek() == &Token::Let || self.peek() == &Token::Const || self.peek() == &Token::Var {
            let kind = match self.peek() {
                Token::Var => VarKind::Var,
                Token::Let => VarKind::Let,
                Token::Const => VarKind::Const,
                _ => unreachable!(),
            };
            self.advance();
            let id = match self.peek().clone() {
                Token::Identifier(n) => { self.advance(); n }
                _ => return Err(Error::ParseError("Expected identifier in for-loop".into())),
            };
            if self.peek() == &Token::In {
                self.advance();
                let right = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForInStatement {
                    left: ForInLeft::VariableDeclaration { kind, id }, right, body,
                });
            }
            if self.peek() == &Token::Of {
                self.advance();
                let right = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForOfStatement {
                    left: ForInLeft::VariableDeclaration { kind, id }, right, body, is_async: false,
                });
            }
            let init_expr = Expression::Identifier(id);
            let mut declarations = Vec::new();
            let decl_id = match &init_expr {
                Expression::Identifier(n) => n.clone(),
                _ => unreachable!(),
            };
            let init_val = if self.peek() == &Token::Assign {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            declarations.push(VariableDeclarator { id: decl_id, type_annotation: None, init: init_val });
            let init = Some(Box::new(ForInit::Variable(Statement::VariableDeclaration { kind, declarations })));
            self.expect(&Token::Semicolon)?;
            let condition = if self.peek() != &Token::Semicolon { Some(self.parse_expression()?) } else { None };
            self.expect(&Token::Semicolon)?;
            let update = if self.peek() != &Token::RightParen { Some(self.parse_expression()?) } else { None };
            self.expect(&Token::RightParen)?;
            let body = Box::new(self.parse_statement()?);
            return Ok(Statement::ForStatement { init, condition, update, body });
        }

        if let Token::Identifier(id) = self.peek().clone() {
            self.advance();
            if self.peek() == &Token::In {
                self.advance();
                let right = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForInStatement {
                    left: ForInLeft::Identifier(id), right, body,
                });
            }
            if self.peek() == &Token::Of {
                self.advance();
                let right = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForOfStatement {
                    left: ForInLeft::Identifier(id), right, body, is_async: false,
                });
            }
            self.pos -= 1;
        }

        let init_expr = self.parse_expression()?;
        let init = Some(Box::new(ForInit::Expression(init_expr)));
        self.expect(&Token::Semicolon)?;
        let condition = if self.peek() != &Token::Semicolon { Some(self.parse_expression()?) } else { None };
        self.expect(&Token::Semicolon)?;
        let update = if self.peek() != &Token::RightParen { Some(self.parse_expression()?) } else { None };
        self.expect(&Token::RightParen)?;
        let body = Box::new(self.parse_statement()?);
        Ok(Statement::ForStatement { init, condition, update, body })
    }

    pub(crate) fn parse_do_while_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::Do)?;
        let body = Box::new(self.parse_statement()?);
        self.expect(&Token::While)?;
        self.expect(&Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(&Token::RightParen)?;
        if self.peek() == &Token::Semicolon { self.advance(); }
        Ok(Statement::DoWhileStatement { condition, body })
    }

    pub(crate) fn parse_switch_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::Switch)?;
        self.expect(&Token::LeftParen)?;
        let discriminant = self.parse_expression()?;
        self.expect(&Token::RightParen)?;
        self.expect(&Token::LeftBrace)?;
        let mut cases = Vec::new();
        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            let test = if self.peek() == &Token::Case {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                self.expect(&Token::Default)?;
                None
            };
            self.expect(&Token::Colon)?;
            let mut consequent = Vec::new();
            while self.peek() != &Token::Case && self.peek() != &Token::Default
                && self.peek() != &Token::RightBrace && self.peek() != &Token::Eof
            {
                consequent.push(self.parse_statement()?);
            }
            cases.push(SwitchCase { test, consequent });
        }
        self.expect(&Token::RightBrace)?;
        Ok(Statement::SwitchStatement { discriminant, cases })
    }

    pub(crate) fn parse_try_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::Try)?;
        self.expect(&Token::LeftBrace)?;
        let block = self.parse_block_body()?;
        self.expect(&Token::RightBrace)?;

        let handler = if self.peek() == &Token::Catch {
            self.advance();
            self.expect(&Token::LeftParen)?;
            let param = match self.advance() {
                Token::Identifier(name) => name,
                t => return Err(Error::ParseError(format!("Expected parameter, got {:?}", t))),
            };
            self.expect(&Token::RightParen)?;
            self.expect(&Token::LeftBrace)?;
            let body = self.parse_block_body()?;
            self.expect(&Token::RightBrace)?;
            Some(CatchClause { param, body })
        } else {
            None
        };

        let finalizer = if self.peek() == &Token::Finally {
            self.advance();
            self.expect(&Token::LeftBrace)?;
            let body = self.parse_block_body()?;
            self.expect(&Token::RightBrace)?;
            Some(body)
        } else {
            None
        };

        Ok(Statement::TryStatement { block, handler, finalizer })
    }

    pub(crate) fn parse_throw_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::Throw)?;
        let argument = self.parse_expression()?;
        self.expect(&Token::Semicolon)?;
        Ok(Statement::ThrowStatement(argument))
    }

    pub(crate) fn parse_class_declaration(&mut self) -> Result<Statement> {
        self.expect(&Token::Class)?;
        let name = match self.advance() {
            Token::Identifier(name) => name,
            t => return Err(Error::ParseError(format!("Expected class name, got {:?}", t))),
        };
        let superclass = if self.peek() == &Token::Extends {
            self.advance();
            Some(self.parse_call()?)
        } else {
            None
        };
        self.expect(&Token::LeftBrace)?;
        let body = self.parse_class_body()?;
        self.expect(&Token::RightBrace)?;
        Ok(Statement::ClassDeclaration { name, superclass: superclass.map(Box::new), body })
    }

    pub(crate) fn parse_class_body(&mut self) -> Result<Vec<ClassMember>> {
        let mut members = Vec::new();
        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            let is_static = if self.peek() == &Token::Static { self.advance(); true } else { false };
            let is_async = if self.peek() == &Token::Async { self.advance(); true } else { false };

            if self.peek() == &Token::Constructor {
                self.advance();
                self.expect(&Token::LeftParen)?;
                let params = self.parse_params()?;
                self.expect(&Token::RightParen)?;
                self.expect(&Token::LeftBrace)?;
                let body = self.parse_block_body()?;
                self.expect(&Token::RightBrace)?;
                members.push(ClassMember::Constructor { params, body });
            } else if self.peek() == &Token::Get && !is_async {
                self.advance();
                let name = match self.advance() {
                    Token::Identifier(name) => name,
                    t => return Err(Error::ParseError(format!("Expected property name after 'get', got {:?}", t))),
                };
                self.expect(&Token::LeftParen)?;
                self.expect(&Token::RightParen)?;
                self.expect(&Token::LeftBrace)?;
                let body = self.parse_block_body()?;
                self.expect(&Token::RightBrace)?;
                members.push(ClassMember::Getter { name, body, is_static });
            } else if self.peek() == &Token::Set && !is_async {
                self.advance();
                let name = match self.advance() {
                    Token::Identifier(name) => name,
                    t => return Err(Error::ParseError(format!("Expected property name after 'set', got {:?}", t))),
                };
                self.expect(&Token::LeftParen)?;
                let param = match self.advance() {
                    Token::Identifier(name) => name,
                    t => return Err(Error::ParseError(format!("Expected parameter name, got {:?}", t))),
                };
                self.expect(&Token::RightParen)?;
                self.expect(&Token::LeftBrace)?;
                let body = self.parse_block_body()?;
                self.expect(&Token::RightBrace)?;
                members.push(ClassMember::Setter { name, param, body, is_static });
            } else {
                let name = match self.advance() {
                    Token::Identifier(name) => name,
                    t => return Err(Error::ParseError(format!("Expected method name, got {:?}", t))),
                };
                if self.peek() == &Token::LeftParen {
                    self.advance();
                    let params = self.parse_params()?;
                    self.expect(&Token::RightParen)?;
                    self.expect(&Token::LeftBrace)?;
                    let body = self.parse_block_body()?;
                    self.expect(&Token::RightBrace)?;
                    members.push(ClassMember::Method { name, params, body, is_static, is_async });
                } else {
                    members.push(ClassMember::Property { name, is_static });
                    if self.peek() == &Token::Semicolon { self.advance(); }
                }
            }
        }
        Ok(members)
    }

    pub(crate) fn parse_import_declaration(&mut self) -> Result<Statement> {
        self.expect(&Token::Import)?;
        let mut specifiers = Vec::new();

        if matches!(self.peek(), Token::String(_)) {
            let source = match self.advance() { Token::String(s) => s, _ => unreachable!() };
            if self.peek() == &Token::Semicolon { self.advance(); }
            return Ok(Statement::ImportDeclaration { specifiers: vec![], source });
        }

        if self.peek() == &Token::LeftBrace {
            self.advance();
            while self.peek() != &Token::RightBrace {
                let imported = match self.advance() {
                    Token::Identifier(name) => name,
                    t => return Err(Error::ParseError(format!("Expected identifier, got {:?}", t))),
                };
                let local = if self.peek() == &Token::As {
                    self.advance();
                    match self.advance() {
                        Token::Identifier(name) => name,
                        t => return Err(Error::ParseError(format!("Expected identifier, got {:?}", t))),
                    }
                } else {
                    imported.clone()
                };
                specifiers.push(ImportSpecifier { local, imported: Some(imported) });
                if self.peek() == &Token::Comma { self.advance(); }
            }
            self.expect(&Token::RightBrace)?;
        } else         if self.peek() == &Token::Star {
            self.advance();
            self.expect(&Token::As)?;
            let local = match self.advance() {
                Token::Identifier(name) => name,
                t => return Err(Error::ParseError(format!("Expected identifier, got {:?}", t))),
            };
            specifiers.push(ImportSpecifier { local, imported: Some("*".to_string()) });
        } else if matches!(self.peek(), Token::Identifier(_)) {
            let local = match self.advance() {
                Token::Identifier(name) => name,
                t => return Err(Error::ParseError(format!("Expected identifier, got {:?}", t))),
            };
            specifiers.push(ImportSpecifier { local: local.clone(), imported: Some(local) });
        }

        if self.peek() == &Token::From { self.advance(); }
        let source = match self.advance() {
            Token::String(s) => s,
            t => return Err(Error::ParseError(format!("Expected string, got {:?}", t))),
        };
        if self.peek() == &Token::Semicolon { self.advance(); }
        Ok(Statement::ImportDeclaration { specifiers, source })
    }

    pub(crate) fn parse_export_declaration(&mut self) -> Result<Statement> {
        self.expect(&Token::Export)?;

        if self.peek() == &Token::Default {
            self.advance();
            let decl = self.parse_statement()?;
            return Ok(Statement::ExportDefaultDeclaration { declaration: Box::new(decl) });
        }

        if self.peek() == &Token::LeftBrace {
            self.advance();
            while self.peek() != &Token::RightBrace {
                self.advance();
                if self.peek() == &Token::Comma { self.advance(); }
            }
            self.expect(&Token::RightBrace)?;
            if self.peek() == &Token::From { self.advance(); self.advance(); }
            if self.peek() == &Token::Semicolon { self.advance(); }
            return Ok(Statement::ExportDeclaration {
                declaration: Box::new(Statement::Expression(Expression::UndefinedLiteral)),
            });
        }

        let decl = self.parse_statement()?;
        Ok(Statement::ExportDeclaration { declaration: Box::new(decl) })
    }

    pub(crate) fn parse_interface_declaration(&mut self) -> Result<Statement> {
        self.expect(&Token::Interface)?;
        let name = match self.advance() {
            Token::Identifier(name) => name,
            t => return Err(Error::ParseError(format!("Expected interface name, got {:?}", t))),
        };
        let mut extends = Vec::new();
        if self.peek() == &Token::Extends {
            self.advance();
            loop {
                match self.advance() {
                    Token::Identifier(n) => extends.push(n),
                    t => return Err(Error::ParseError(format!("Expected identifier, got {:?}", t))),
                }
                if self.peek() == &Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::LeftBrace)?;
        let mut members = Vec::new();
        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            if self.peek() == &Token::Comma {
                self.advance();
                continue;
            }
            let name = match self.advance() {
                Token::Identifier(n) => n,
                t => return Err(Error::ParseError(format!("Expected property name, got {:?}", t))),
            };
            if self.peek() == &Token::LeftParen {
                self.advance();
                let mut params = Vec::new();
                if self.peek() != &Token::RightParen {
                    loop {
                        let pname = match self.advance() {
                            Token::Identifier(n) => n,
                            t => return Err(Error::ParseError(format!("Expected param name, got {:?}", t))),
                        };
                        let ptype = if self.peek() == &Token::Colon {
                            self.advance();
                            self.parse_type_annotation()?
                        } else {
                            TypeAnnotation::Any
                        };
                        params.push((pname, ptype));
                        if self.peek() == &Token::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(&Token::RightParen)?;
                let return_type = if self.peek() == &Token::Colon {
                    self.advance();
                    self.parse_type_annotation()?
                } else {
                    TypeAnnotation::Any
                };
                if self.peek() == &Token::Semicolon { self.advance(); }
                members.push(InterfaceMember::Method { name, params, return_type });
            } else {
                let optional = if self.peek() == &Token::Question {
                    self.advance();
                    true
                } else {
                    false
                };
                self.expect(&Token::Colon)?;
                let type_annotation = self.parse_type_annotation()?;
                if self.peek() == &Token::Semicolon { self.advance(); }
                members.push(InterfaceMember::Property { name, type_annotation, optional });
            }
        }
        self.expect(&Token::RightBrace)?;
        if self.peek() == &Token::Semicolon { self.advance(); }
        Ok(Statement::InterfaceDeclaration { name, extends, members })
    }

    pub(crate) fn parse_type_alias_declaration(&mut self) -> Result<Statement> {
        self.expect(&Token::Type)?;
        let name = match self.advance() {
            Token::Identifier(name) => name,
            t => return Err(Error::ParseError(format!("Expected type name, got {:?}", t))),
        };
        self.expect(&Token::Assign)?;
        let type_annotation = self.parse_type_annotation()?;
        if self.peek() == &Token::Semicolon { self.advance(); }
        Ok(Statement::TypeAliasDeclaration { name, type_annotation })
    }

    pub(crate) fn parse_enum_declaration(&mut self) -> Result<Statement> {
        self.expect(&Token::Enum)?;
        let name = match self.advance() {
            Token::Identifier(name) => name,
            t => return Err(Error::ParseError(format!("Expected enum name, got {:?}", t))),
        };
        self.expect(&Token::LeftBrace)?;
        let mut members = Vec::new();
        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            let member_name = match self.advance() {
                Token::Identifier(n) => n,
                t => return Err(Error::ParseError(format!("Expected enum member name, got {:?}", t))),
            };
            let value = if self.peek() == &Token::Assign {
                self.advance();
                match self.peek().clone() {
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
            members.push(EnumMember { name: member_name, value });
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::RightBrace)?;
        if self.peek() == &Token::Semicolon { self.advance(); }
        Ok(Statement::EnumDeclaration { name, members })
    }
}
