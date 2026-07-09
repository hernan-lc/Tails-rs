use super::super::*;
use crate::errors::Result;

impl<'a> Parser<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn parse_arrow_body(
        &mut self,
        params: Vec<String>,
        param_types: Option<Vec<Option<TypeAnnotation>>>,
        defaults: Vec<Option<Expression>>,
        rest_param: Option<String>,
        return_type: Option<TypeAnnotation>,
        is_async: bool,
        param_patterns: Vec<Option<crate::compiler::parser::BindingPattern>>,
    ) -> Result<SpannedNode<Expression>> {
        if self.peek().token == Token::LeftBrace {
            self.advance();
            let body = self.parse_block_body()?;
            self.expect(&Token::RightBrace)?;
            Ok(self.spanned(Expression::ArrowFunction {
                params,
                param_patterns,
                param_types,
                defaults,
                rest_param,
                return_type,
                body: Box::new(ArrowFunctionBody::Block(body)),
                is_async,
            }))
        } else {
            let expr = self.parse_assignment()?;
            Ok(self.spanned(Expression::ArrowFunction {
                params,
                param_patterns,
                param_types,
                defaults,
                rest_param,
                return_type,
                body: Box::new(ArrowFunctionBody::Expression(expr.inner)),
                is_async,
            }))
        }
    }
}
