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
                        if name == "keyof" {
                            let inner = self.parse_primary_type()?;
                            return Ok(TypeAnnotation::Keyof(Box::new(inner)));
                        }
                        // Handle dotted type names like v.InferInput or A.B.C
                        let mut full_name = name;
                        while self.peek().token == Token::Dot {
                            self.advance(); // consume '.'
                            if let Token::Identifier(ref prop) = self.peek().token {
                                let prop = prop.clone();
                                self.advance();
                                full_name = format!("{}.{}", full_name, prop);
                            } else {
                                break;
                            }
                        }
                        if self.peek().token == Token::Less {
                            self.advance();
                            let mut args = vec![self.parse_type_annotation()?];
                            while self.peek().token == Token::Comma {
                                self.advance();
                                if matches!(self.peek().token, Token::Greater | Token::ShiftRight) {
                                    break;
                                }
                                args.push(self.parse_type_annotation()?);
                            }
                            match self.peek().token {
                                Token::Greater => {
                                    self.advance();
                                }
                                Token::ShiftRight => {
                                    // `>>` is two `>` - consume one, leave the other
                                    self.peek_token_mut().token = Token::Greater;
                                }
                                _ => {
                                    return Err(Error::ParseError(format!(
                                        "Expected '>' to close generic arguments, got {:?}",
                                        self.peek().token
                                    )));
                                }
                            }
                            Ok(TypeAnnotation::Generic {
                                name: full_name,
                                args,
                            })
                        } else if let Token::Identifier(ref is_name) = self.peek().token {
                            if is_name == "is" {
                                self.advance();
                                let ty = self.parse_type_annotation()?;
                                Ok(TypeAnnotation::TypePredicate {
                                    param_name: full_name,
                                    ty: Box::new(ty),
                                })
                            } else {
                                Ok(TypeAnnotation::Named(full_name))
                            }
                        } else {
                            Ok(TypeAnnotation::Named(full_name))
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
            Token::Typeof => {
                self.advance();
                let inner = self.parse_primary_type()?;
                Ok(TypeAnnotation::Typeof(Box::new(inner)))
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
                        while matches!(&self.peek().token, Token::Identifier(s) if s == "readonly")
                        {
                            self.advance();
                        }
                        // Handle indexed signatures [key: string] and mapped types [K in keyof T]
                        if self.peek().token == Token::LeftBracket {
                            self.advance();
                            let mut depth = 1u32;
                            while depth > 0 && self.peek().token != Token::Eof {
                                match self.peek().token {
                                    Token::LeftBracket => {
                                        depth += 1;
                                        self.advance();
                                    }
                                    Token::RightBracket => {
                                        depth -= 1;
                                        if depth > 0 {
                                            self.advance();
                                        } else {
                                            self.advance();
                                            break;
                                        }
                                    }
                                    _ => {
                                        self.advance();
                                    }
                                }
                            }
                            // Handle `?` for optional
                            if self.peek().token == Token::Question {
                                self.advance();
                            }
                            // Expect `: Type` or `(params): ReturnType`
                            if self.peek().token == Token::LeftParen {
                                // Computed method signature: [key](param): Type
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
                                if self.peek().token == Token::Colon {
                                    self.advance();
                                    let _return_type = self.parse_type_annotation()?;
                                }
                            } else if self.peek().token == Token::Colon {
                                self.advance();
                                let _ty = self.parse_type_annotation()?;
                            }
                            // Consume separator (, or ;)
                            if self.peek().token == Token::Semicolon
                                || self.peek().token == Token::Comma
                            {
                                self.advance();
                            }
                            if self.peek().token == Token::RightBrace {
                                break;
                            }
                            continue;
                        }
                        let name = match self.advance().token {
                            Token::Identifier(n) => n,
                            t => {
                                return Err(Error::ParseError(format!(
                                    "Expected property name in type at {}:{}, got {:?}",
                                    self.current_span.line, self.current_span.col, t
                                )))
                            }
                        };
                        // Handle optional method signature: method?(): void
                        // If ? is followed by (, it's an optional method — skip the ?
                        if self.peek().token == Token::Question {
                            let saved = self.pos;
                            self.advance(); // skip ?
                            if self.peek().token == Token::LeftParen {
                                // It's an optional method, ? already skipped
                            } else {
                                // It's an optional property, restore position
                                self.pos = saved;
                            }
                        }
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
                if self.is_function_type_after_paren() {
                    let mut param_types = Vec::new();
                    if self.peek().token != Token::RightParen {
                        loop {
                            if self.peek().token == Token::RightParen {
                                break;
                            }
                            if self.peek().token == Token::Ellipsis {
                                self.advance();
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
                } else {
                    let inner = self.parse_type_annotation()?;
                    self.expect(&Token::RightParen)?;
                    Ok(inner)
                }
            }
            Token::New => {
                // Constructor type: new (params) => ReturnType
                self.advance();
                self.expect(&Token::LeftParen)?;
                let mut param_types = Vec::new();
                if self.peek().token != Token::RightParen {
                    loop {
                        if self.peek().token == Token::RightParen {
                            break;
                        }
                        if self.peek().token == Token::Ellipsis {
                            self.advance();
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
                Ok(TypeAnnotation::Constructor {
                    params: param_types,
                    return_type,
                })
            }
            _ => Ok(TypeAnnotation::Any),
        }?;
        if self.peek().token == Token::LeftBracket {
            // Check if this is T[] (array type) or T["key"] (indexed access type)
            if self.pos + 1 < self.tokens.len()
                && self.tokens[self.pos + 1].token == Token::RightBracket
            {
                self.advance();
                self.expect(&Token::RightBracket)?;
                Ok(TypeAnnotation::Array(Box::new(base)))
            } else {
                // Indexed access type like T["_input"] - skip the brackets and content
                self.advance();
                while self.peek().token != Token::RightBracket && self.peek().token != Token::Eof {
                    self.advance();
                }
                if self.peek().token == Token::RightBracket {
                    self.advance();
                }
                Ok(base)
            }
        } else {
            Ok(base)
        }
    }
}
