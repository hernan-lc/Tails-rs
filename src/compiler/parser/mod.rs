mod expressions;
mod statements;
mod types;

use crate::compiler::lexer::{TemplatePart, Token};
use crate::errors::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation {
    Number,
    String,
    Boolean,
    Null,
    Undefined,
    Void,
    Any,
    Unknown,
    Never,
    Named(String),
    Array(Box<TypeAnnotation>),
    Tuple(Vec<TypeAnnotation>),
    Union(Vec<TypeAnnotation>),
    Intersection(Vec<TypeAnnotation>),
    Object(Vec<(String, TypeAnnotation, bool)>),
    Function {
        params: Vec<TypeAnnotation>,
        return_type: Box<TypeAnnotation>,
    },
    Literal(TypeLiteral),
    Generic {
        name: String,
        args: Vec<TypeAnnotation>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeLiteral {
    Number(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Program(Vec<Statement>),
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub enum InterfaceMember {
    Property {
        name: String,
        type_annotation: TypeAnnotation,
        optional: bool,
    },
    Method {
        name: String,
        params: Vec<(String, TypeAnnotation)>,
        return_type: TypeAnnotation,
    },
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    VariableDeclaration {
        kind: VarKind,
        declarations: Vec<VariableDeclarator>,
    },
    FunctionDeclaration {
        name: String,
        params: Vec<String>,
        param_types: Option<Vec<Option<TypeAnnotation>>>,
        return_type: Option<TypeAnnotation>,
        body: Vec<Statement>,
        is_async: bool,
        is_generator: bool,
    },
    ReturnStatement(Option<Expression>),
    YieldStatement(Option<Expression>),
    IfStatement {
        condition: Expression,
        consequent: Box<Statement>,
        alternate: Option<Box<Statement>>,
    },
    WhileStatement {
        condition: Expression,
        body: Box<Statement>,
    },
    BlockStatement(Vec<Statement>),
    ForStatement {
        init: Option<Box<ForInit>>,
        condition: Option<Expression>,
        update: Option<Expression>,
        body: Box<Statement>,
    },
    ForInStatement {
        left: ForInLeft,
        right: Expression,
        body: Box<Statement>,
    },
    ForOfStatement {
        left: ForInLeft,
        right: Expression,
        body: Box<Statement>,
        is_async: bool,
    },
    DoWhileStatement {
        condition: Expression,
        body: Box<Statement>,
    },
    SwitchStatement {
        discriminant: Expression,
        cases: Vec<SwitchCase>,
    },
    BreakStatement,
    ContinueStatement,
    TryStatement {
        block: Vec<Statement>,
        handler: Option<CatchClause>,
        finalizer: Option<Vec<Statement>>,
    },
    ThrowStatement(Expression),
    ClassDeclaration {
        name: String,
        superclass: Option<Box<Expression>>,
        body: Vec<ClassMember>,
    },
    ImportDeclaration {
        specifiers: Vec<ImportSpecifier>,
        source: String,
    },
    ExportDeclaration {
        kind: ExportDeclarationKind,
    },
    ExportDefaultDeclaration {
        declaration: Box<Statement>,
    },
    InterfaceDeclaration {
        name: String,
        extends: Vec<String>,
        members: Vec<InterfaceMember>,
    },
    TypeAliasDeclaration {
        name: String,
        type_annotation: TypeAnnotation,
    },
    EnumDeclaration {
        name: String,
        members: Vec<EnumMember>,
    },
}

#[derive(Debug, Clone)]
pub struct EnumMember {
    pub name: String,
    pub value: Option<TypeLiteral>,
}

#[derive(Debug, Clone)]
pub enum ForInit {
    Variable(Statement),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub enum ForInLeft {
    Identifier(String),
    VariableDeclaration { kind: VarKind, id: String },
}

#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub test: Option<Expression>,
    pub consequent: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct CatchClause {
    pub param: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum AccessModifier {
    Public,
    Private,
    Protected,
    Readonly,
}

#[derive(Debug, Clone)]
pub struct ConstructorParam {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub access_modifiers: Vec<AccessModifier>,
}

#[derive(Debug, Clone)]
pub enum ClassMember {
    Method {
        name: String,
        params: Vec<String>,
        param_types: Option<Vec<Option<TypeAnnotation>>>,
        return_type: Option<TypeAnnotation>,
        body: Vec<Statement>,
        is_static: bool,
        is_async: bool,
    },
    Property {
        name: String,
        is_static: bool,
    },
    Constructor {
        params: Vec<ConstructorParam>,
        body: Vec<Statement>,
    },
    Getter {
        name: String,
        return_type: Option<TypeAnnotation>,
        body: Vec<Statement>,
        is_static: bool,
    },
    Setter {
        name: String,
        param: String,
        param_type: Option<TypeAnnotation>,
        body: Vec<Statement>,
        is_static: bool,
    },
}

#[derive(Debug, Clone)]
pub struct ImportSpecifier {
    pub local: String,
    pub imported: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExportSpecifier {
    pub local: String,
    pub exported: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ExportDeclarationKind {
    Local(Box<Statement>),
    ReExport {
        specifiers: Vec<ExportSpecifier>,
        source: String,
    },
}

#[derive(Debug, Clone)]
pub enum VarKind {
    Var,
    Let,
    Const,
}

#[derive(Debug, Clone)]
pub enum BindingPattern {
    Identifier(String),
    Array(Vec<ArrayBindingElement>),
    Object(Vec<ObjectBindingElement>),
}

#[derive(Debug, Clone)]
pub enum ArrayBindingElement {
    Pattern(BindingPattern, Option<Expression>),
    Rest(Box<BindingPattern>),
    Skip,
}

#[derive(Debug, Clone)]
pub struct ObjectBindingElement {
    pub key: String,
    pub value: BindingPattern,
    pub shorthand: bool,
    pub default_value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct VariableDeclarator {
    pub id: BindingPattern,
    pub type_annotation: Option<TypeAnnotation>,
    pub init: Option<Expression>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    NumberLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),
    NullLiteral,
    UndefinedLiteral,
    NaNLiteral,
    Identifier(String),
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    Assignment {
        target: Box<Expression>,
        value: Box<Expression>,
        op: Option<CompoundAssignmentOp>,
    },
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },
    Member {
        object: Box<Expression>,
        property: Box<Expression>,
        computed: bool,
    },
    OptionalMember {
        object: Box<Expression>,
        property: Box<Expression>,
        computed: bool,
    },
    OptionalCall {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },
    FunctionExpression {
        name: Option<String>,
        params: Vec<String>,
        param_types: Option<Vec<Option<TypeAnnotation>>>,
        return_type: Option<TypeAnnotation>,
        body: Vec<Statement>,
        is_async: bool,
        is_generator: bool,
    },
    ArrowFunction {
        params: Vec<String>,
        param_types: Option<Vec<Option<TypeAnnotation>>>,
        return_type: Option<TypeAnnotation>,
        body: Box<ArrowFunctionBody>,
        is_async: bool,
    },
    NewExpression {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },
    ConditionalExpression {
        test: Box<Expression>,
        consequent: Box<Expression>,
        alternate: Box<Expression>,
    },
    UpdateExpression {
        op: UpdateOperator,
        operand: Box<Expression>,
        prefix: bool,
    },
    TemplateLiteral {
        quasis: Vec<String>,
        expressions: Vec<Expression>,
    },
    ClassExpression {
        name: Option<String>,
        superclass: Option<Box<Expression>>,
        body: Vec<ClassMember>,
    },
    AwaitExpression {
        argument: Box<Expression>,
    },
    SuperCall {
        args: Vec<Expression>,
    },
    SuperMember {
        property: Box<Expression>,
        computed: bool,
    },
    ArrayLiteral {
        elements: Vec<Expression>,
    },
    ObjectLiteral {
        properties: Vec<ObjectProperty>,
    },
    SpreadElement {
        argument: Box<Expression>,
    },
    RestElement {
        argument: Box<BindingPattern>,
    },
    TypeAssertion {
        expression: Box<Expression>,
        type_annotation: TypeAnnotation,
    },
}

#[derive(Debug, Clone)]
pub struct ObjectProperty {
    pub key: String,
    pub value: Expression,
    pub shorthand: bool,
    pub computed: bool,
    pub computed_key: Option<Expression>,
}

#[derive(Debug, Clone)]
pub enum ArrowFunctionBody {
    Expression(Expression),
    Block(Vec<Statement>),
}

#[derive(Debug, Clone)]
pub enum CompoundAssignmentOp {
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    AndAssign,
    OrAssign,
}

#[derive(Debug, Clone)]
pub enum UpdateOperator {
    Increment,
    Decrement,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    StrictEq,
    NotEqual,
    StrictNotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
    Power,
    Instanceof,
    In,
    NullishCoalescing,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negate,
    Not,
    Typeof,
    Void,
    Delete,
    BitNot,
}

pub fn parse(tokens: &[Token]) -> Result<AstNode> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

pub(crate) struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    pub(crate) fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    pub(crate) fn advance(&mut self) -> Token {
        let token = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    pub(crate) fn expect(&mut self, expected: &Token) -> Result<()> {
        let token = self.advance();
        if token == *expected {
            Ok(())
        } else {
            Err(Error::ParseError(format!(
                "Expected {:?}, got {:?}",
                expected, token
            )))
        }
    }

    fn parse_program(&mut self) -> Result<AstNode> {
        let mut statements = Vec::new();
        while self.peek() != &Token::Eof {
            statements.push(self.parse_statement()?);
        }
        Ok(AstNode::Program(statements))
    }

    pub(crate) fn parse_statement(&mut self) -> Result<Statement> {
        match self.peek().clone() {
            Token::Const | Token::Let | Token::Var => self.parse_variable_declaration(),
            Token::Function => self.parse_function_declaration(),
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
                Ok(Statement::BreakStatement)
            }
            Token::Continue => {
                self.advance();
                self.expect(&Token::Semicolon)?;
                Ok(Statement::ContinueStatement)
            }
            Token::Try => self.parse_try_statement(),
            Token::Throw => self.parse_throw_statement(),
            Token::Class => self.parse_class_declaration(),
            Token::Import => self.parse_import_declaration(),
            Token::Export => self.parse_export_declaration(),
            Token::Interface => self.parse_interface_declaration(),
            Token::Type => self.parse_type_alias_declaration(),
            Token::Enum => self.parse_enum_declaration(),
            _ => self.parse_expression_statement(),
        }
    }

    pub(crate) fn parse_block_body(&mut self) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();
        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    pub(crate) fn parse_expression_statement(&mut self) -> Result<Statement> {
        let expr = self.parse_expression()?;
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        Ok(Statement::Expression(expr))
    }

    pub(crate) fn parse_typed_params(
        &mut self,
    ) -> Result<(Vec<String>, Vec<Option<TypeAnnotation>>)> {
        let mut params = Vec::new();
        let mut param_types = Vec::new();
        if self.peek() != &Token::RightParen {
            loop {
                let param = match self.advance() {
                    Token::Identifier(name) => name,
                    token => {
                        return Err(Error::ParseError(format!(
                            "Expected parameter name, got {:?}",
                            token
                        )))
                    }
                };
                let ty = if self.peek() == &Token::Colon {
                    self.advance();
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };
                params.push(param);
                param_types.push(ty);
                if self.peek() == &Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        Ok((params, param_types))
    }

    pub(crate) fn parse_constructor_params(&mut self) -> Result<Vec<ConstructorParam>> {
        let mut params = Vec::new();
        if self.peek() != &Token::RightParen {
            loop {
                let mut access_modifiers = Vec::new();
                loop {
                    match self.peek() {
                        Token::Public => {
                            self.advance();
                            access_modifiers.push(AccessModifier::Public);
                        }
                        Token::Private => {
                            self.advance();
                            access_modifiers.push(AccessModifier::Private);
                        }
                        Token::Protected => {
                            self.advance();
                            access_modifiers.push(AccessModifier::Protected);
                        }
                        Token::Readonly => {
                            self.advance();
                            access_modifiers.push(AccessModifier::Readonly);
                        }
                        _ => break,
                    }
                }
                let param = match self.advance() {
                    Token::Identifier(name) => name,
                    token => {
                        return Err(Error::ParseError(format!(
                            "Expected parameter name, got {:?}",
                            token
                        )))
                    }
                };
                let type_annotation = if self.peek() == &Token::Colon {
                    self.advance();
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };
                params.push(ConstructorParam {
                    name: param,
                    type_annotation,
                    access_modifiers,
                });
                if self.peek() == &Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        Ok(params)
    }

    pub(crate) fn token_to_key_string(&mut self) -> Result<String> {
        match self.advance() {
            Token::Identifier(n) => Ok(n),
            Token::String(s) => Ok(s),
            Token::Number(n) => Ok(n.to_string()),
            Token::Catch => Ok("catch".to_string()),
            Token::Finally => Ok("finally".to_string()),
            Token::Throw => Ok("throw".to_string()),
            Token::Get => Ok("get".to_string()),
            Token::Set => Ok("set".to_string()),
            Token::Delete => Ok("delete".to_string()),
            Token::New => Ok("new".to_string()),
            Token::This => Ok("this".to_string()),
            Token::Return => Ok("return".to_string()),
            Token::If => Ok("if".to_string()),
            Token::Else => Ok("else".to_string()),
            Token::While => Ok("while".to_string()),
            Token::For => Ok("for".to_string()),
            Token::Do => Ok("do".to_string()),
            Token::Function => Ok("function".to_string()),
            Token::Class => Ok("class".to_string()),
            Token::Switch => Ok("switch".to_string()),
            Token::Case => Ok("case".to_string()),
            Token::Break => Ok("break".to_string()),
            Token::Continue => Ok("continue".to_string()),
            Token::Typeof => Ok("typeof".to_string()),
            Token::Instanceof => Ok("instanceof".to_string()),
            Token::In => Ok("in".to_string()),
            Token::Void => Ok("void".to_string()),
            Token::Const => Ok("const".to_string()),
            Token::Let => Ok("let".to_string()),
            Token::Var => Ok("var".to_string()),
            Token::Super => Ok("super".to_string()),
            Token::Extends => Ok("extends".to_string()),
            Token::Static => Ok("static".to_string()),
            Token::Public => Ok("public".to_string()),
            Token::Private => Ok("private".to_string()),
            Token::Protected => Ok("protected".to_string()),
            Token::Readonly => Ok("readonly".to_string()),
            Token::Import => Ok("import".to_string()),
            Token::Export => Ok("export".to_string()),
            Token::Default => Ok("default".to_string()),
            Token::From => Ok("from".to_string()),
            Token::As => Ok("as".to_string()),
            Token::Async => Ok("async".to_string()),
            Token::Await => Ok("await".to_string()),
            Token::Try => Ok("try".to_string()),
            Token::Constructor => Ok("constructor".to_string()),
            Token::Of => Ok("of".to_string()),
            t => Err(Error::ParseError(format!(
                "Expected property key, got {:?}",
                t
            ))),
        }
    }

    pub(crate) fn token_to_property_name(&mut self) -> Result<Expression> {
        match self.advance() {
            Token::Identifier(n) => Ok(Expression::Identifier(n)),
            Token::Catch => Ok(Expression::Identifier("catch".to_string())),
            Token::Finally => Ok(Expression::Identifier("finally".to_string())),
            Token::Throw => Ok(Expression::Identifier("throw".to_string())),
            Token::Get => Ok(Expression::Identifier("get".to_string())),
            Token::Set => Ok(Expression::Identifier("set".to_string())),
            Token::Delete => Ok(Expression::Identifier("delete".to_string())),
            Token::New => Ok(Expression::Identifier("new".to_string())),
            Token::This => Ok(Expression::Identifier("this".to_string())),
            Token::Return => Ok(Expression::Identifier("return".to_string())),
            Token::If => Ok(Expression::Identifier("if".to_string())),
            Token::Else => Ok(Expression::Identifier("else".to_string())),
            Token::While => Ok(Expression::Identifier("while".to_string())),
            Token::For => Ok(Expression::Identifier("for".to_string())),
            Token::Do => Ok(Expression::Identifier("do".to_string())),
            Token::Function => Ok(Expression::Identifier("function".to_string())),
            Token::Class => Ok(Expression::Identifier("class".to_string())),
            Token::Switch => Ok(Expression::Identifier("switch".to_string())),
            Token::Case => Ok(Expression::Identifier("case".to_string())),
            Token::Break => Ok(Expression::Identifier("break".to_string())),
            Token::Continue => Ok(Expression::Identifier("continue".to_string())),
            Token::Typeof => Ok(Expression::Identifier("typeof".to_string())),
            Token::Instanceof => Ok(Expression::Identifier("instanceof".to_string())),
            Token::In => Ok(Expression::Identifier("in".to_string())),
            Token::Void => Ok(Expression::Identifier("void".to_string())),
            Token::Const => Ok(Expression::Identifier("const".to_string())),
            Token::Let => Ok(Expression::Identifier("let".to_string())),
            Token::Var => Ok(Expression::Identifier("var".to_string())),
            Token::Super => Ok(Expression::Identifier("super".to_string())),
            Token::Extends => Ok(Expression::Identifier("extends".to_string())),
            Token::Static => Ok(Expression::Identifier("static".to_string())),
            Token::Public => Ok(Expression::Identifier("public".to_string())),
            Token::Private => Ok(Expression::Identifier("private".to_string())),
            Token::Protected => Ok(Expression::Identifier("protected".to_string())),
            Token::Readonly => Ok(Expression::Identifier("readonly".to_string())),
            Token::Import => Ok(Expression::Identifier("import".to_string())),
            Token::Export => Ok(Expression::Identifier("export".to_string())),
            Token::Default => Ok(Expression::Identifier("default".to_string())),
            Token::From => Ok(Expression::Identifier("from".to_string())),
            Token::As => Ok(Expression::Identifier("as".to_string())),
            Token::Async => Ok(Expression::Identifier("async".to_string())),
            Token::Await => Ok(Expression::Identifier("await".to_string())),
            Token::Try => Ok(Expression::Identifier("try".to_string())),
            Token::Constructor => Ok(Expression::Identifier("constructor".to_string())),
            Token::Of => Ok(Expression::Identifier("of".to_string())),
            t => Err(Error::ParseError(format!(
                "Expected property name, got {:?}",
                t
            ))),
        }
    }
}
