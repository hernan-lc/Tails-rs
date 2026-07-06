use super::super::*;
use crate::errors::{Error, Result};

impl<'a> Parser<'a> {
    fn parse_for_in_of(
        &mut self,
        left: ForInLeft,
        is_for_await: bool,
    ) -> Result<Option<SpannedNode<Statement>>> {
        if self.peek().token == Token::In {
            self.advance();
            let right = self.parse_expression()?.inner;
            self.expect(&Token::RightParen)?;
            let body = Box::new(self.parse_statement()?);
            return Ok(Some(self.spanned(Statement::ForInStatement {
                left,
                right,
                body,
            })));
        }
        if self.peek().token == Token::Of {
            self.advance();
            let right = self.parse_expression()?.inner;
            self.expect(&Token::RightParen)?;
            let body = Box::new(self.parse_statement()?);
            return Ok(Some(self.spanned(Statement::ForOfStatement {
                left,
                right,
                body,
                is_async: is_for_await,
            })));
        }
        Ok(None)
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
            if let Some(stmt) = self.parse_for_in_of(
                ForInLeft::VariableDeclaration {
                    kind: kind.clone(),
                    id: id.clone(),
                },
                is_for_await,
            )? {
                return Ok(stmt);
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
            if let Some(stmt) = self.parse_for_in_of(ForInLeft::Identifier(id), is_for_await)? {
                return Ok(stmt);
            }
            self.pos -= 1;
        }

        if matches!(self.peek().token, Token::LeftBracket | Token::LeftBrace) {
            let pattern = self.parse_binding_pattern()?;
            if let Some(stmt) = self.parse_for_in_of(ForInLeft::Pattern(pattern), is_for_await)? {
                return Ok(stmt);
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
}
