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
    },
    ReturnStatement(Option<Expression>),
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
        declaration: Box<Statement>,
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
    VariableDeclaration {
        kind: VarKind,
        id: String,
    },
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
pub enum ClassMember {
    Method {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
        is_static: bool,
        is_async: bool,
    },
    Property {
        name: String,
        is_static: bool,
    },
    Constructor {
        params: Vec<String>,
        body: Vec<Statement>,
    },
    Getter {
        name: String,
        body: Vec<Statement>,
        is_static: bool,
    },
    Setter {
        name: String,
        param: String,
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
pub enum VarKind {
    Var,
    Let,
    Const,
}

#[derive(Debug, Clone)]
pub struct VariableDeclarator {
    pub id: String,
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
    FunctionExpression {
        name: Option<String>,
        params: Vec<String>,
        param_types: Option<Vec<Option<TypeAnnotation>>>,
        return_type: Option<TypeAnnotation>,
        body: Vec<Statement>,
        is_async: bool,
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
        properties: Vec<(String, Expression)>,
    },
    TypeAssertion {
        expression: Box<Expression>,
        type_annotation: TypeAnnotation,
    },
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
