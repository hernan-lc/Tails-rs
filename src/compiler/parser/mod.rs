mod expressions;
mod program;
mod statements;
mod types;

pub use types::{TypeAnnotation, TypeLiteral, TypedParams};

use crate::compiler::lexer::{SpannedToken, Token};
use crate::errors::{Error, Result, Span};
use crate::well_known as wk;

#[derive(Debug, Clone)]
pub struct SpannedNode<T> {
    pub inner: T,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Program(Vec<SpannedNode<Statement>>),
    Statement(Box<SpannedNode<Statement>>),
    Expression(Box<SpannedNode<Expression>>),
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
        param_patterns: Vec<Option<BindingPattern>>,
        param_types: Option<Vec<Option<TypeAnnotation>>>,
        defaults: Vec<Option<Expression>>,
        rest_param: Option<String>,
        return_type: Option<TypeAnnotation>,
        body: Vec<SpannedNode<Statement>>,
        is_async: bool,
        is_generator: bool,
    },
    ReturnStatement(Option<Expression>),
    YieldStatement(Option<Expression>),
    IfStatement {
        condition: Expression,
        consequent: Box<SpannedNode<Statement>>,
        alternate: Option<Box<SpannedNode<Statement>>>,
    },
    WhileStatement {
        condition: Expression,
        body: Box<SpannedNode<Statement>>,
    },
    BlockStatement(Vec<SpannedNode<Statement>>),
    ForStatement {
        init: Option<Box<ForInit>>,
        condition: Option<Expression>,
        update: Option<Expression>,
        body: Box<SpannedNode<Statement>>,
    },
    ForInStatement {
        left: ForInLeft,
        right: Expression,
        body: Box<SpannedNode<Statement>>,
    },
    ForOfStatement {
        left: ForInLeft,
        right: Expression,
        body: Box<SpannedNode<Statement>>,
        is_async: bool,
    },
    DoWhileStatement {
        condition: Expression,
        body: Box<SpannedNode<Statement>>,
    },
    SwitchStatement {
        discriminant: Expression,
        cases: Vec<SwitchCase>,
    },
    /// `break;` or `break label;`
    BreakStatement(Option<String>),
    /// `continue;` or `continue label;`
    ContinueStatement(Option<String>),
    /// `label: statement` — used by content-type and other loops that
    /// `continue label` / `break label`.
    LabeledStatement {
        label: String,
        body: Box<SpannedNode<Statement>>,
    },
    /// Bare `;` empty statement.
    EmptyStatement,
    TryStatement {
        block: Vec<SpannedNode<Statement>>,
        handler: Option<CatchClause>,
        finalizer: Option<Vec<SpannedNode<Statement>>>,
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
        declaration: Box<SpannedNode<Statement>>,
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
    Variable(Box<SpannedNode<Statement>>),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum ForInLeft {
    Identifier(String),
    Pattern(BindingPattern),
    VariableDeclaration { kind: VarKind, id: BindingPattern },
}

#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub test: Option<Expression>,
    pub consequent: Vec<SpannedNode<Statement>>,
}

#[derive(Debug, Clone)]
pub struct CatchClause {
    pub param: String,
    pub body: Vec<SpannedNode<Statement>>,
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
    pub default: Option<Expression>,
    pub pattern: Option<BindingPattern>,
}
#[derive(Debug, Clone)]
pub enum ClassMember {
    Method {
        name: String,
        params: Vec<String>,
        param_types: Option<Vec<Option<TypeAnnotation>>>,
        return_type: Option<TypeAnnotation>,
        body: Vec<SpannedNode<Statement>>,
        is_static: bool,
        is_async: bool,
        /// Computed member name (e.g. `[Symbol.iterator]() {}`). `None` for
        /// ordinary string/number/identifier names.
        computed: Option<Expression>,
    },
    Property {
        name: String,
        is_static: bool,
        init: Option<Expression>,
        /// Computed member name (e.g. `[key] = value`). `None` for ordinary names.
        computed: Option<Expression>,
    },
    Constructor {
        params: Vec<ConstructorParam>,
        body: Vec<SpannedNode<Statement>>,
    },
    Getter {
        name: String,
        return_type: Option<TypeAnnotation>,
        body: Vec<SpannedNode<Statement>>,
        is_static: bool,
        /// Computed member name (e.g. `get [Symbol.iterator]() {}`).
        computed: Option<Expression>,
    },
    Setter {
        name: String,
        param: String,
        param_type: Option<TypeAnnotation>,
        body: Vec<SpannedNode<Statement>>,
        is_static: bool,
        /// Computed member name (e.g. `set [Symbol.iterator](v) {}`).
        computed: Option<Expression>,
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
    Local(Box<SpannedNode<Statement>>),
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
    Pattern(BindingPattern, Box<Option<Expression>>),
    Rest(Box<BindingPattern>),
    Skip,
}

#[derive(Debug, Clone)]
pub struct ObjectBindingElement {
    pub key: String,
    pub value: BindingPattern,
    pub shorthand: bool,
    pub default_value: Option<Expression>,
    /// `...rest` element — `key`/`value` name the rest binding.
    pub is_rest: bool,
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
    BigIntLiteral(String),
    StringLiteral(String),
    RegexLiteral {
        pattern: String,
        flags: String,
    },
    BooleanLiteral(bool),
    NullLiteral,
    UndefinedLiteral,
    NaNLiteral,
    InfinityLiteral,
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
        param_patterns: Vec<Option<BindingPattern>>,
        param_types: Option<Vec<Option<TypeAnnotation>>>,
        defaults: Vec<Option<Expression>>,
        rest_param: Option<String>,
        return_type: Option<TypeAnnotation>,
        body: Vec<SpannedNode<Statement>>,
        is_async: bool,
        is_generator: bool,
    },
    ArrowFunction {
        params: Vec<String>,
        param_patterns: Vec<Option<BindingPattern>>,
        param_types: Option<Vec<Option<TypeAnnotation>>>,
        defaults: Vec<Option<Expression>>,
        rest_param: Option<String>,
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
    TaggedTemplate {
        tag: Box<Expression>,
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
    ImportExpression {
        source: Box<Expression>,
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
    MetaProperty {
        meta: String,
        property: String,
    },
}

#[derive(Debug, Clone)]
pub struct ObjectProperty {
    pub key: String,
    pub value: Expression,
    pub shorthand: bool,
    pub computed: bool,
    pub computed_key: Option<Expression>,
    pub is_getter: bool,
    pub is_setter: bool,
}

#[derive(Debug, Clone)]
pub enum ArrowFunctionBody {
    Expression(Expression),
    Block(Vec<SpannedNode<Statement>>),
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
    XorAssign,
    BitAndAssign,
    BitOrAssign,
    ShiftLeftAssign,
    ShiftRightAssign,
    UnsignedShiftRightAssign,
    NullishCoalescingAssign,
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
    UnsignedShiftRight,
    Power,
    Instanceof,
    In,
    NullishCoalescing,
    Comma,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negate,
    Not,
    Typeof,
    Void,
    Delete,
    BitNot,
    UnaryPlus,
}

pub fn parse(tokens: &mut [SpannedToken]) -> Result<AstNode> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

pub(crate) fn token_keyword_string(t: &Token) -> Option<String> {
    match t {
        Token::Identifier(n) => Some(n.clone()),
        Token::String(s) => Some(s.clone()),
        Token::Catch => Some(wk::CATCH.to_string()),
        Token::Finally => Some(wk::FINALLY.to_string()),
        Token::Throw => Some("throw".to_string()),
        Token::Get => Some("get".to_string()),
        Token::Set => Some("set".to_string()),
        Token::Delete => Some("delete".to_string()),
        Token::New => Some("new".to_string()),
        Token::This => Some("this".to_string()),
        Token::Return => Some("return".to_string()),
        Token::If => Some("if".to_string()),
        Token::Else => Some("else".to_string()),
        Token::While => Some("while".to_string()),
        Token::For => Some("for".to_string()),
        Token::Do => Some("do".to_string()),
        Token::Function => Some("function".to_string()),
        Token::Class => Some("class".to_string()),
        Token::Switch => Some("switch".to_string()),
        Token::Case => Some("case".to_string()),
        Token::Break => Some("break".to_string()),
        Token::Continue => Some("continue".to_string()),
        Token::Typeof => Some("typeof".to_string()),
        Token::Instanceof => Some("instanceof".to_string()),
        Token::In => Some("in".to_string()),
        Token::Void => Some("void".to_string()),
        Token::Const => Some("const".to_string()),
        Token::Let => Some("let".to_string()),
        Token::Var => Some("var".to_string()),
        Token::Super => Some("super".to_string()),
        Token::Extends => Some("extends".to_string()),
        Token::Static => Some("static".to_string()),
        Token::Public => Some("public".to_string()),
        Token::Private => Some("private".to_string()),
        Token::Protected => Some("protected".to_string()),
        Token::Readonly => Some("readonly".to_string()),
        Token::Import => Some("import".to_string()),
        Token::Export => Some("export".to_string()),
        Token::Default => Some("default".to_string()),
        Token::From => Some("from".to_string()),
        Token::As => Some("as".to_string()),
        Token::Async => Some("async".to_string()),
        Token::Await => Some("await".to_string()),
        Token::Try => Some("try".to_string()),
        Token::Constructor => Some(wk::CONSTRUCTOR.to_string()),
        Token::Of => Some("of".to_string()),
        Token::Enum => Some("enum".to_string()),
        Token::Interface => Some("interface".to_string()),
        Token::Yield => Some("yield".to_string()),
        Token::Type => Some("type".to_string()),
        _ => None,
    }
}

pub(crate) struct Parser<'a> {
    tokens: &'a mut [SpannedToken],
    pos: usize,
    current_span: Span,
    eof_token: SpannedToken,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a mut [SpannedToken]) -> Self {
        Self {
            tokens,
            pos: 0,
            current_span: Span::new(1, 1, 0),
            eof_token: SpannedToken {
                token: Token::Eof,
                span: Span::new(0, 0, 0),
            },
        }
    }

    fn spanned<T>(&self, node: T) -> SpannedNode<T> {
        SpannedNode {
            inner: node,
            span: Some(self.current_span),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn current_span(&self) -> Span {
        self.current_span
    }

    pub(crate) fn peek(&self) -> &SpannedToken {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            &self.eof_token
        }
    }

    pub(crate) fn peek_token_mut(&mut self) -> &mut SpannedToken {
        if self.pos < self.tokens.len() {
            &mut self.tokens[self.pos]
        } else {
            &mut self.eof_token
        }
    }

    pub(crate) fn advance(&mut self) -> SpannedToken {
        let st = self.tokens.get(self.pos).cloned().unwrap_or(SpannedToken {
            token: Token::Eof,
            span: self.current_span,
        });
        self.current_span = st.span;
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        st
    }

    pub(crate) fn expect(&mut self, expected: &Token) -> Result<()> {
        let st = self.advance();
        if st.token == *expected {
            Ok(())
        } else {
            Err(Error::ParseError(format!(
                "Expected {:?} at {}:{}, got {:?}",
                expected, st.span.line, st.span.col, st.token
            )))
        }
    }

    #[allow(dead_code)]
    pub(crate) fn expect_identifier(&mut self, context: &str) -> Result<String> {
        let st = self.advance();
        match st.token {
            Token::Identifier(name) => Ok(name),
            t => Err(Error::ParseError(format!(
                "Expected {}, got {:?}",
                context, t
            ))),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn optional_return_type(&mut self) -> Result<Option<TypeAnnotation>> {
        if self.peek().token == Token::Colon {
            self.advance();
            Ok(Some(self.parse_type_annotation()?))
        } else {
            Ok(None)
        }
    }

    #[allow(dead_code)]
    pub(crate) fn consume_optional_semicolon(&mut self) {
        if self.peek().token == Token::Semicolon {
            self.advance();
        }
    }

    /// Consume a statement-terminating semicolon, applying Automatic Semicolon
    /// Insertion (ECMAScript):
    /// 1. Explicit `;`
    /// 2. End of input
    /// 3. Next token starts on a later line (line-break ASI)
    /// 4. Next token is `}` (block close ASI — e.g. `return value}` / `x = fn() { return y }`)
    ///
    /// For-loop headers do NOT use this and must keep their mandatory semicolons.
    pub(crate) fn expect_statement_semicolon(&mut self) -> Result<()> {
        let next = self.peek().token.clone();
        if next == Token::Semicolon {
            self.advance();
            return Ok(());
        }
        // ASI: EOF, line break before next token, or closing `}` of a block.
        if next == Token::Eof
            || next == Token::RightBrace
            || self.peek().span.line > self.current_span.line
        {
            return Ok(());
        }
        Err(Error::ParseError(format!(
            "Expected {:?} at {}:{}, got {:?}",
            Token::Semicolon,
            self.peek().span.line,
            self.peek().span.col,
            next
        )))
    }

    pub(crate) fn is_function_type_after_paren(&self) -> bool {
        let mut depth = 1;
        let mut pos = self.pos;
        while pos < self.tokens.len() && depth > 0 {
            match self.tokens[pos].token {
                Token::LeftParen | Token::LeftBrace | Token::LeftBracket | Token::Less => {
                    depth += 1
                }
                Token::RightParen | Token::RightBrace | Token::RightBracket | Token::Greater => {
                    depth -= 1
                }
                _ => {}
            }
            if depth > 0 {
                pos += 1;
            }
        }
        if pos >= self.tokens.len() {
            return false;
        }
        let next_pos = pos + 1;
        next_pos < self.tokens.len() && matches!(self.tokens[next_pos].token, Token::Arrow)
    }

    pub(crate) fn parse_block_body(&mut self) -> Result<Vec<SpannedNode<Statement>>> {
        let mut statements = Vec::new();
        while self.peek().token != Token::RightBrace && self.peek().token != Token::Eof {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    pub(crate) fn parse_expression_statement(&mut self) -> Result<SpannedNode<Statement>> {
        let expr = self.parse_expression_with_comma()?;
        if self.peek().token == Token::Semicolon {
            self.advance();
        }
        Ok(self.spanned(Statement::Expression(expr.inner)))
    }

    pub(crate) fn parse_typed_params(&mut self) -> Result<TypedParams> {
        let mut params = Vec::new();
        let mut param_patterns = Vec::new();
        let mut param_types = Vec::new();
        let mut defaults = Vec::new();
        let mut rest_param = None;
        if self.peek().token != Token::RightParen {
            loop {
                if self.peek().token == Token::This {
                    self.advance();
                    if self.peek().token == Token::Colon {
                        self.advance();
                        let _ = self.parse_type_annotation()?;
                    }
                    if self.peek().token == Token::Comma {
                        self.advance();
                        if self.peek().token == Token::RightParen {
                            break;
                        }
                    } else {
                        break;
                    }
                    continue;
                }
                if self.peek().token == Token::Ellipsis {
                    self.advance();
                    let st = self.advance();
                    let param = match token_keyword_string(&st.token) {
                        Some(name) => name,
                        None => {
                            return Err(Error::ParseError(format!(
                                "Expected parameter name after '...', got {:?}",
                                st.token
                            )))
                        }
                    };
                    if self.peek().token == Token::Colon {
                        self.advance();
                        let _ = self.parse_type_annotation()?;
                    }
                    rest_param = Some(param);
                    break;
                }
                let (param, pattern) = match self.peek().token.clone() {
                    Token::LeftBracket | Token::LeftBrace => {
                        let pattern = self.parse_binding_pattern()?;
                        (format!("__destr_{}", params.len()), Some(pattern))
                    }
                    _ => {
                        // Allow reserved words as parameter names (e.g. `from` in object-inspect).
                        if let Some(name) = token_keyword_string(&self.peek().token) {
                            self.advance();
                            (name, None)
                        } else {
                            return Err(Error::ParseError(format!(
                                "Expected parameter name, got {:?}",
                                self.peek().token
                            )));
                        }
                    }
                };
                let ty = if self.peek().token == Token::Colon {
                    self.advance();
                    Some(self.parse_type_annotation()?)
                } else if self.peek().token == Token::Question {
                    self.advance();
                    if self.peek().token == Token::Colon {
                        self.advance();
                        Some(self.parse_type_annotation()?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let default = if self.peek().token == Token::Assign {
                    self.advance();
                    Some(self.parse_assignment()?.inner)
                } else {
                    None
                };
                params.push(param);
                param_patterns.push(pattern);
                param_types.push(ty);
                defaults.push(default);
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
        Ok((params, param_types, defaults, rest_param, param_patterns))
    }

    pub(crate) fn parse_constructor_params(&mut self) -> Result<Vec<ConstructorParam>> {
        let mut params = Vec::new();
        if self.peek().token != Token::RightParen {
            loop {
                let mut access_modifiers = Vec::new();
                loop {
                    match self.peek().token {
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
                        Token::Identifier(ref s) if s == "readonly" => {
                            self.advance();
                            access_modifiers.push(AccessModifier::Readonly);
                        }
                        _ => break,
                    }
                }
                if self.peek().token == Token::Ellipsis {
                    self.advance();
                }
                let (param, pattern) = match self.peek().token.clone() {
                    Token::LeftBracket | Token::LeftBrace => {
                        let pattern = self.parse_binding_pattern()?;
                        (format!("__destr_{}", params.len()), Some(pattern))
                    }
                    _ => {
                        let param = match self.advance().token {
                            Token::Identifier(name) => name,
                            token => match token_keyword_string(&token) {
                                Some(name) => name,
                                None => {
                                    return Err(Error::ParseError(format!(
                                        "Expected parameter name, got {:?}",
                                        token
                                    )))
                                }
                            },
                        };
                        (param, None)
                    }
                };
                let type_annotation = if self.peek().token == Token::Colon {
                    self.advance();
                    Some(self.parse_type_annotation()?)
                } else if self.peek().token == Token::Question {
                    self.advance();
                    if self.peek().token == Token::Colon {
                        self.advance();
                        Some(self.parse_type_annotation()?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let default = if self.peek().token == Token::Assign {
                    self.advance();
                    Some(self.parse_assignment()?.inner)
                } else {
                    None
                };
                params.push(ConstructorParam {
                    name: param,
                    type_annotation,
                    access_modifiers,
                    default,
                    pattern,
                });
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
        Ok(params)
    }

    pub(crate) fn token_to_key_string(&mut self) -> Result<String> {
        let st = self.advance();
        match st.token {
            Token::Number(n) => Ok(n.to_string()),
            _ => match token_keyword_string(&st.token) {
                Some(s) => Ok(s),
                None => Err(Error::ParseError(format!(
                    "Expected property key, got {:?}",
                    st.token
                ))),
            },
        }
    }

    pub(crate) fn token_to_property_name(&mut self) -> Result<Expression> {
        let st = self.advance();
        match token_keyword_string(&st.token) {
            Some(s) => Ok(Expression::Identifier(s)),
            None => Err(Error::ParseError(format!(
                "Expected property name, got {:?}",
                st.token
            ))),
        }
    }

    pub(crate) fn skip_type_parameters(&mut self) {
        if self.peek().token == Token::Less {
            let mut depth = 1u32;
            self.advance();
            while depth > 0 && self.peek().token != Token::Eof {
                match self.peek().token {
                    Token::Less => {
                        depth += 1;
                        self.advance();
                    }
                    Token::Greater => {
                        depth -= 1;
                        self.advance();
                    }
                    Token::ShiftLeft => {
                        depth += 2;
                        self.advance();
                    }
                    Token::ShiftRight => {
                        if depth >= 2 {
                            depth -= 2;
                            self.advance();
                        } else {
                            depth = 0;
                            self.peek_token_mut().token = Token::Greater;
                        }
                    }
                    _ => {
                        self.advance();
                    }
                }
            }
        }
    }

    pub(crate) fn advance_as_ident(&mut self) -> String {
        let st = self.advance();
        match st.token {
            Token::Identifier(n) => n,
            Token::String(s) => s,
            other => match token_keyword_string(&other) {
                Some(s) => s,
                None => format!("{:?}", other),
            },
        }
    }
}
