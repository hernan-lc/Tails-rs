use super::super::*;
use crate::errors::{Error, Result};

impl<'a> Parser<'a> {
    pub(crate) fn parse_function_declaration(&mut self) -> Result<SpannedNode<Statement>> {
        let is_async = if self.peek().token == Token::Async {
            self.advance();
            true
        } else {
            false
        };
        let func_keyword_span = self.current_span;
        self.expect(&Token::Function)?;
        let is_generator = if self.peek().token == Token::Star {
            self.advance();
            true
        } else {
            false
        };
        let name = match self.advance().token {
            Token::Identifier(name) => name,
            Token::Get => "get".to_string(),
            Token::Set => "set".to_string(),
            Token::Delete => "delete".to_string(),
            Token::Typeof => "typeof".to_string(),
            Token::Void => "void".to_string(),
            Token::Of => "of".to_string(),
            Token::As => "as".to_string(),
            Token::From => "from".to_string(),
            Token::Enum => "enum".to_string(),
            Token::Interface => "interface".to_string(),
            Token::Yield => "yield".to_string(),
            Token::Await => "await".to_string(),
            Token::Static => "static".to_string(),
            token => {
                return Err(Error::ParseError(format!(
                    "Expected function name, got {:?}",
                    token
                )))
            }
        };
        self.skip_type_parameters();
        self.expect(&Token::LeftParen)?;
        let (params, param_types, defaults, rest_param, param_patterns) =
            self.parse_typed_params()?;
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
        let saved_span = self.current_span;
        self.current_span = func_keyword_span;
        let result = self.spanned(Statement::FunctionDeclaration {
            name,
            params,
            param_patterns,
            param_types: Some(param_types),
            defaults,
            rest_param,
            return_type,
            body,
            is_async,
            is_generator,
        });
        self.current_span = saved_span;
        Ok(result)
    }

    pub(crate) fn parse_return_statement(&mut self) -> Result<SpannedNode<Statement>> {
        self.expect(&Token::Return)?;
        // Restricted production (ASI): a LineTerminator after `return` ends the
        // statement with no argument — so `return\n expr` is `return; expr`, not
        // `return expr`. Also stop at `;` / `}`.
        let value = if self.peek().token != Token::Semicolon
            && self.peek().token != Token::RightBrace
            && self.peek().token != Token::Eof
            && self.peek().span.line == self.current_span.line
        {
            Some(self.parse_expression()?.inner)
        } else {
            None
        };
        self.expect_statement_semicolon()?;
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
        self.expect_statement_semicolon()?;
        Ok(self.spanned(Statement::YieldStatement(value)))
    }
}
