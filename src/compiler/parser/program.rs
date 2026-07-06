use super::*;
use crate::errors::Result;

impl<'a> Parser<'a> {
    pub(crate) fn parse_program(&mut self) -> Result<AstNode> {
        let mut statements = Vec::new();
        while self.peek().token != Token::Eof {
            statements.push(self.parse_statement()?);
        }
        Ok(AstNode::Program(statements))
    }

    pub(crate) fn parse_statement(&mut self) -> Result<SpannedNode<Statement>> {
        match self.peek().token.clone() {
            Token::Const | Token::Let | Token::Var => self.parse_variable_declaration(),
            Token::Function => self.parse_function_declaration(),
            Token::Async => {
                let next_is_function = self
                    .tokens
                    .get(self.pos + 1)
                    .map(|t| t.token == Token::Function)
                    .unwrap_or(false);
                if next_is_function {
                    self.parse_function_declaration()
                } else {
                    self.parse_expression_statement()
                }
            }
            Token::Return => self.parse_return_statement(),
            Token::Yield => self.parse_yield_statement(),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::LeftBrace => self.parse_block_statement(),
            Token::For => self.parse_for_statement(),
            Token::Do => self.parse_do_while_statement(),
            Token::Switch => self.parse_switch_statement(),
            Token::Break => {
                self.advance();
                self.expect(&Token::Semicolon)?;
                Ok(self.spanned(Statement::BreakStatement))
            }
            Token::Continue => {
                self.advance();
                self.expect(&Token::Semicolon)?;
                Ok(self.spanned(Statement::ContinueStatement))
            }
            Token::Try => self.parse_try_statement(),
            Token::Throw => self.parse_throw_statement(),
            Token::Class => self.parse_class_declaration(),
            Token::Import => self.parse_import_declaration(),
            Token::Export => self.parse_export_declaration(),
            Token::Interface => self.parse_interface_declaration(),
            Token::Enum => self.parse_enum_declaration(),
            Token::Identifier(ref s) if s == "type" => {
                let next_is_ident = self
                    .tokens
                    .get(self.pos + 1)
                    .map(|t| matches!(t.token, Token::Identifier(_)))
                    .unwrap_or(false);
                if next_is_ident {
                    self.parse_type_alias_declaration()
                } else {
                    self.parse_expression_statement()
                }
            }
            _ => self.parse_expression_statement(),
        }
    }
}
