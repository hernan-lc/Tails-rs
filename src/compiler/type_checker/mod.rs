mod compatibility;
mod expressions;
mod narrowing;
mod operators;
mod statements;

use crate::compiler::parser::{AstNode, BinaryOperator, Expression, Statement, VarKind};
use crate::errors::{Error, Result};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    NumberLiteral(f64),
    String,
    StringLiteral(String),
    Boolean,
    BooleanLiteral(bool),
    Null,
    Undefined,
    Void,
    Never,
    Unknown,
    Any,
    Union(Vec<Type>),
    Intersection(Vec<Type>),
    Array(Box<Type>),
    Tuple(Vec<Type>),
    Object(Vec<ObjectProperty>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Named(String),
    Generic {
        name: String,
        args: Vec<Type>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectProperty {
    pub name: String,
    pub ty: Type,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct InterfaceDecl {
    pub name: String,
    pub extends: Vec<String>,
    pub properties: Vec<ObjectProperty>,
}

#[derive(Debug, Clone)]
pub struct TypeAliasDecl {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub members: Vec<(String, Type)>,
}

pub struct TypeChecker {
    scopes: Vec<HashMap<String, Type>>,
    interfaces: HashMap<String, InterfaceDecl>,
    type_aliases: HashMap<String, TypeAliasDecl>,
    enums: HashMap<String, EnumDecl>,
    narrowed_types: HashMap<String, Type>,
    known_globals: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self::with_globals(HashMap::new())
    }

    pub fn with_globals(known_globals: HashMap<String, Type>) -> Self {
        Self {
            scopes: vec![HashMap::new()],
            interfaces: HashMap::new(),
            type_aliases: HashMap::new(),
            enums: HashMap::new(),
            narrowed_types: HashMap::new(),
            known_globals,
        }
    }

    pub fn check(ast: &AstNode) -> Result<()> {
        let mut checker = Self::new();
        checker.check_node(ast)
    }

    pub fn check_with_globals(ast: &AstNode, known_globals: HashMap<String, Type>) -> Result<()> {
        let mut checker = Self::with_globals(known_globals);
        checker.check_node(ast)
    }

    fn check_node(&mut self, node: &AstNode) -> Result<()> {
        match node {
            AstNode::Program(statements) => {
                for stmt in statements {
                    self.check_statement(&stmt.inner)?;
                }
                Ok(())
            }
            _ => Err(Error::InternalError("Invalid root node".into())),
        }
    }

    pub(crate) fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub(crate) fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub(crate) fn define_variable(&mut self, name: &str, type_: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), type_);
        }
    }

    pub(crate) fn define_pattern_variables(
        &mut self,
        pattern: &crate::compiler::parser::BindingPattern,
        type_: &Type,
    ) {
        use crate::compiler::parser::BindingPattern;
        match pattern {
            BindingPattern::Identifier(name) => {
                self.define_variable(name, type_.clone());
            }
            BindingPattern::Array(elements) => {
                for element in elements {
                    match element {
                        crate::compiler::parser::ArrayBindingElement::Pattern(pat, _) => {
                            self.define_pattern_variables(pat, type_);
                        }
                        crate::compiler::parser::ArrayBindingElement::Rest(pat) => {
                            self.define_pattern_variables(pat, type_);
                        }
                        crate::compiler::parser::ArrayBindingElement::Skip => {}
                    }
                }
            }
            BindingPattern::Object(elements) => {
                for element in elements {
                    self.define_pattern_variables(&element.value, type_);
                }
            }
        }
    }

    pub(crate) fn get_variable_type(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        self.known_globals.get(name).cloned()
    }

    pub(crate) fn resolve_annotation(
        &self,
        ann: &crate::compiler::parser::TypeAnnotation,
    ) -> Result<Type> {
        use crate::compiler::parser::TypeAnnotation;
        match ann {
            TypeAnnotation::Number => Ok(Type::Number),
            TypeAnnotation::String => Ok(Type::String),
            TypeAnnotation::Boolean => Ok(Type::Boolean),
            TypeAnnotation::Null => Ok(Type::Null),
            TypeAnnotation::Undefined => Ok(Type::Undefined),
            TypeAnnotation::Void => Ok(Type::Void),
            TypeAnnotation::Any => Ok(Type::Any),
            TypeAnnotation::Unknown => Ok(Type::Unknown),
            TypeAnnotation::Never => Ok(Type::Never),
            TypeAnnotation::Named(name) => Ok(Type::Named(name.clone())),
            TypeAnnotation::Array(inner) => {
                let inner_ty = self.resolve_annotation(inner)?;
                Ok(Type::Array(Box::new(inner_ty)))
            }
            TypeAnnotation::Tuple(elements) => {
                let elem_tys: Vec<Type> = elements
                    .iter()
                    .map(|e| self.resolve_annotation(e))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Type::Tuple(elem_tys))
            }
            TypeAnnotation::Union(variants) => {
                let var_tys: Vec<Type> = variants
                    .iter()
                    .map(|v| self.resolve_annotation(v))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Type::Union(var_tys))
            }
            TypeAnnotation::Intersection(types) => {
                let tys: Vec<Type> = types
                    .iter()
                    .map(|t| self.resolve_annotation(t))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Type::Intersection(tys))
            }
            TypeAnnotation::Object(props) => {
                let mut obj_props = Vec::new();
                for (name, ty_ann, optional) in props {
                    let ty = self.resolve_annotation(ty_ann)?;
                    obj_props.push(ObjectProperty {
                        name: name.clone(),
                        ty,
                        optional: *optional,
                    });
                }
                Ok(Type::Object(obj_props))
            }
            TypeAnnotation::Function {
                params,
                return_type,
            } => {
                let param_tys: Vec<Type> = params
                    .iter()
                    .map(|p| self.resolve_annotation(p))
                    .collect::<Result<Vec<_>>>()?;
                let ret = self.resolve_annotation(return_type)?;
                Ok(Type::Function {
                    params: param_tys,
                    return_type: Box::new(ret),
                })
            }
            TypeAnnotation::Literal(lit) => match lit {
                crate::compiler::parser::TypeLiteral::Number(n) => Ok(Type::NumberLiteral(*n)),
                crate::compiler::parser::TypeLiteral::String(s) => {
                    Ok(Type::StringLiteral(s.clone()))
                }
                crate::compiler::parser::TypeLiteral::Boolean(b) => Ok(Type::BooleanLiteral(*b)),
            },
            TypeAnnotation::Generic { name, args } => {
                let arg_tys: Vec<Type> = args
                    .iter()
                    .map(|a| self.resolve_annotation(a))
                    .collect::<Result<Vec<_>>>()?;
                Ok(Type::Generic {
                    name: name.clone(),
                    args: arg_tys,
                })
            }
            TypeAnnotation::Constructor {
                params,
                return_type,
            } => {
                let param_tys: Vec<Type> = params
                    .iter()
                    .map(|p| self.resolve_annotation(p))
                    .collect::<Result<Vec<_>>>()?;
                let ret = self.resolve_annotation(return_type)?;
                Ok(Type::Function {
                    params: param_tys,
                    return_type: Box::new(ret),
                })
            }
            TypeAnnotation::TypePredicate { ty, .. } => self.resolve_annotation(ty),
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
