use super::*;
use crate::errors::{Error, Result};

impl<'a> Parser<'a> {
    pub(crate) fn parse_type_annotation(&mut self) -> Result<TypeAnnotation> {
        self.parse_union_type()
    }

    fn parse_union_type(&mut self) -> Result<TypeAnnotation> {
        let mut types = vec![self.parse_intersection_type()?];
        while self.peek().token == Token::BitOr {
            self.advance();
            types.push(self.parse_intersection_type()?);
        }
        if types.len() == 1 {
            Ok(types.remove(0))
        } else {
            Ok(TypeAnnotation::Union(types))
        }
    }

    fn parse_intersection_type(&mut self) -> Result<TypeAnnotation> {
        let mut types = vec![self.parse_primary_type()?];
        while self.peek().token == Token::BitAnd {
            self.advance();
            types.push(self.parse_primary_type()?);
        }
        if types.len() == 1 {
            Ok(types.remove(0))
        } else {
            Ok(TypeAnnotation::Intersection(types))
        }
    }

    fn parse_primary_type(&mut self) -> Result<TypeAnnotation> {
        let base = match self.peek().token.clone() {
            Token::Identifier(name) => {
                self.advance();
                match name.as_str() {
                    "number" => Ok(TypeAnnotation::Number),
                    "string" => Ok(TypeAnnotation::String),
                    "boolean" => Ok(TypeAnnotation::Boolean),
                    "null" => Ok(TypeAnnotation::Null),
                    "undefined" => Ok(TypeAnnotation::Undefined),
                    "void" => Ok(TypeAnnotation::Void),
                    "any" => Ok(TypeAnnotation::Any),
                    "unknown" => Ok(TypeAnnotation::Unknown),
                    "never" => Ok(TypeAnnotation::Never),
                    _ => {
                        if self.peek().token == Token::Less {
                            self.advance();
                            let mut args = vec![self.parse_type_annotation()?];
                            while self.peek().token == Token::Comma {
                                self.advance();
                                if self.peek().token == Token::Greater {
                                    break;
                                }
                                args.push(self.parse_type_annotation()?);
                            }
                            self.expect(&Token::Greater)?;
                            Ok(TypeAnnotation::Generic { name, args })
                        } else {
                            Ok(TypeAnnotation::Named(name))
                        }
                    }
                }
            }
            Token::Void => {
                self.advance();
                Ok(TypeAnnotation::Void)
            }
            Token::Number(n) => {
                self.advance();
                Ok(TypeAnnotation::Literal(TypeLiteral::Number(n)))
            }
            Token::String(s) => {
                self.advance();
                Ok(TypeAnnotation::Literal(TypeLiteral::String(s)))
            }
            Token::LeftBracket => {
                self.advance();
                if self.peek().token == Token::RightBracket {
                    self.advance();
                    return Ok(TypeAnnotation::Array(Box::new(TypeAnnotation::Any)));
                }
                let first = self.parse_type_annotation()?;
                let mut elements = vec![first];
                while self.peek().token == Token::Comma {
                    self.advance();
                    if self.peek().token == Token::RightBracket {
                        break;
                    }
                    elements.push(self.parse_type_annotation()?);
                }
                self.expect(&Token::RightBracket)?;
                Ok(TypeAnnotation::Tuple(elements))
            }
            Token::LeftBrace => {
                self.advance();
                let mut properties = Vec::new();
                if self.peek().token != Token::RightBrace {
                    loop {
                        // Skip optional modifiers like 'readonly'
                        while matches!(self.peek().token, Token::Readonly) {
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
                        if self.peek().token == Token::LeftParen {
                            self.advance();
                            let mut param_types = Vec::new();
                            if self.peek().token != Token::RightParen {
                                loop {
                                    if matches!(self.peek().token, Token::Identifier(_)) {
                                        self.advance();
                                        if self.peek().token == Token::Colon {
                                            self.advance();
                                            param_types.push(self.parse_type_annotation()?);
                                        } else {
                                            param_types.push(TypeAnnotation::Any);
                                        }
                                    } else {
                                        param_types.push(self.parse_type_annotation()?);
                                    }
                                    if self.peek().token == Token::Comma {
                                        self.advance();
                                        if self.peek().token == Token::RightParen {
                                            break;
                                        }
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
                            properties.push((
                                name,
                                TypeAnnotation::Function {
                                    params: param_types,
                                    return_type: Box::new(return_type),
                                },
                                false,
                            ));
                        } else {
                            let optional = if self.peek().token == Token::Question {
                                self.advance();
                                true
                            } else {
                                false
                            };
                            self.expect(&Token::Colon)?;
                            let ty = self.parse_type_annotation()?;
                            properties.push((name, ty, optional));
                        }
                        // Handle both ',' and ';' as property separators
                        if self.peek().token == Token::Comma
                            || self.peek().token == Token::Semicolon
                        {
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
                Ok(TypeAnnotation::Object(properties))
            }
            Token::LeftParen => {
                self.advance();
                let mut param_types = Vec::new();
                if self.peek().token != Token::RightParen {
                    loop {
                        if self.peek().token == Token::RightParen {
                            break;
                        }
                        if matches!(self.peek().token, Token::Identifier(_)) {
                            self.advance();
                            if self.peek().token == Token::Colon {
                                self.advance();
                                param_types.push(self.parse_type_annotation()?);
                            } else {
                                param_types.push(TypeAnnotation::Any);
                            }
                        } else {
                            param_types.push(self.parse_type_annotation()?);
                        }
                        if self.peek().token == Token::Comma {
                            self.advance();
                            if self.peek().token == Token::RightParen {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                self.expect(&Token::RightParen)?;
                self.expect(&Token::Arrow)?;
                let return_type = Box::new(self.parse_type_annotation()?);
                Ok(TypeAnnotation::Function {
                    params: param_types,
                    return_type,
                })
            }
            _ => Ok(TypeAnnotation::Any),
        }?;
        if self.peek().token == Token::LeftBracket {
            self.advance();
            self.expect(&Token::RightBracket)?;
            Ok(TypeAnnotation::Array(Box::new(base)))
        } else {
            Ok(base)
        }
    }
}
