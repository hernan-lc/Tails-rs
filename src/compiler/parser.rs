use crate::compiler::lexer::{Token, TemplatePart};
use crate::errors::{Error, Result};

#[derive(Debug, Clone)]
pub enum AstNode {
    Program(Vec<Statement>),
    Statement(Statement),
    Expression(Expression),
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
        body: Vec<Statement>,
        is_async: bool,
    },
    ArrowFunction {
        params: Vec<String>,
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

pub fn parse(tokens: &[Token]) -> Result<AstNode> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, expected: &Token) -> Result<()> {
        let token = self.advance();
        if token == *expected {
            Ok(())
        } else {
            Err(Error::ParseError(format!("Expected {:?}, got {:?}", expected, token)))
        }
    }

    fn parse_program(&mut self) -> Result<AstNode> {
        let mut statements = Vec::new();
        while self.peek() != &Token::Eof {
            statements.push(self.parse_statement()?);
        }
        Ok(AstNode::Program(statements))
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.peek().clone() {
            Token::Const | Token::Let | Token::Var => self.parse_variable_declaration(),
            Token::Function => self.parse_function_declaration(),
            Token::Return => self.parse_return_statement(),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::LeftBrace => self.parse_block_statement(),
            Token::For => self.parse_for_statement(),
            Token::Do => self.parse_do_while_statement(),
            Token::Switch => self.parse_switch_statement(),
            Token::Break => { self.advance(); self.expect(&Token::Semicolon)?; Ok(Statement::BreakStatement) }
            Token::Continue => { self.advance(); self.expect(&Token::Semicolon)?; Ok(Statement::ContinueStatement) }
            Token::Try => self.parse_try_statement(),
            Token::Throw => self.parse_throw_statement(),
            Token::Class => self.parse_class_declaration(),
            Token::Import => self.parse_import_declaration(),
            Token::Export => self.parse_export_declaration(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement> {
        let kind = match self.advance() {
            Token::Var => VarKind::Var,
            Token::Let => VarKind::Let,
            Token::Const => VarKind::Const,
            _ => unreachable!(),
        };
        let mut declarations = Vec::new();
        loop {
            let id = match self.advance() {
                Token::Identifier(name) => name,
                token => return Err(Error::ParseError(format!("Expected identifier, got {:?}", token))),
            };
            let init = if self.peek() == &Token::Assign {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            declarations.push(VariableDeclarator { id, init });
            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::Semicolon)?;
        Ok(Statement::VariableDeclaration { kind, declarations })
    }

    fn parse_function_declaration(&mut self) -> Result<Statement> {
        let is_async = if self.peek() == &Token::Async {
            self.advance();
            true
        } else {
            false
        };
        self.expect(&Token::Function)?;
        let name = match self.advance() {
            Token::Identifier(name) => name,
            token => return Err(Error::ParseError(format!("Expected function name, got {:?}", token))),
        };
        self.expect(&Token::LeftParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RightParen)?;
        self.expect(&Token::LeftBrace)?;
        let body = self.parse_block_body()?;
        self.expect(&Token::RightBrace)?;
        Ok(Statement::FunctionDeclaration { name, params, body, is_async })
    }

    fn parse_params(&mut self) -> Result<Vec<String>> {
        let mut params = Vec::new();
        if self.peek() != &Token::RightParen {
            loop {
                let param = match self.advance() {
                    Token::Identifier(name) => name,
                    token => return Err(Error::ParseError(format!("Expected parameter name, got {:?}", token))),
                };
                params.push(param);
                if self.peek() == &Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        Ok(params)
    }

    fn parse_return_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::Return)?;
        let value = if self.peek() != &Token::Semicolon && self.peek() != &Token::RightBrace {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.expect(&Token::Semicolon)?;
        Ok(Statement::ReturnStatement(value))
    }

    fn parse_if_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::If)?;
        self.expect(&Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(&Token::RightParen)?;
        let consequent = Box::new(self.parse_statement()?);
        let alternate = if self.peek() == &Token::Else {
            self.advance();
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        Ok(Statement::IfStatement { condition, consequent, alternate })
    }

    fn parse_while_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::While)?;
        self.expect(&Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(&Token::RightParen)?;
        let body = Box::new(self.parse_statement()?);
        Ok(Statement::WhileStatement { condition, body })
    }

    fn parse_block_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::LeftBrace)?;
        let body = self.parse_block_body()?;
        self.expect(&Token::RightBrace)?;
        Ok(Statement::BlockStatement(body))
    }

    fn parse_block_body(&mut self) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();
        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_expression_statement(&mut self) -> Result<Statement> {
        let expr = self.parse_expression()?;
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        Ok(Statement::Expression(expr))
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression> {
        let left = self.parse_ternary()?;
        match self.peek().clone() {
            Token::Assign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment { target: Box::new(left), value: Box::new(value), op: None })
            }
            Token::PlusAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment { target: Box::new(left), value: Box::new(value), op: Some(CompoundAssignmentOp::AddAssign) })
            }
            Token::MinusAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment { target: Box::new(left), value: Box::new(value), op: Some(CompoundAssignmentOp::SubAssign) })
            }
            Token::StarAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment { target: Box::new(left), value: Box::new(value), op: Some(CompoundAssignmentOp::MulAssign) })
            }
            Token::SlashAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment { target: Box::new(left), value: Box::new(value), op: Some(CompoundAssignmentOp::DivAssign) })
            }
            Token::PercentAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment { target: Box::new(left), value: Box::new(value), op: Some(CompoundAssignmentOp::ModAssign) })
            }
            Token::AndAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment { target: Box::new(left), value: Box::new(value), op: Some(CompoundAssignmentOp::AndAssign) })
            }
            Token::OrAssign => {
                self.advance();
                let value = self.parse_assignment()?;
                Ok(Expression::Assignment { target: Box::new(left), value: Box::new(value), op: Some(CompoundAssignmentOp::OrAssign) })
            }
            _ => Ok(left),
        }
    }

    fn parse_ternary(&mut self) -> Result<Expression> {
        let condition = self.parse_or()?;
        if self.peek() == &Token::Question {
            self.advance();
            let consequent = self.parse_assignment()?;
            self.expect(&Token::Colon)?;
            let alternate = self.parse_assignment()?;
            Ok(Expression::ConditionalExpression {
                test: Box::new(condition),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
            })
        } else {
            Ok(condition)
        }
    }

    fn parse_or(&mut self) -> Result<Expression> {
        let mut left = self.parse_and()?;
        while self.peek() == &Token::Or {
            self.advance();
            let right = self.parse_and()?;
            left = Expression::BinaryOp { op: BinaryOperator::Or, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expression> {
        let mut left = self.parse_equality()?;
        while self.peek() == &Token::And {
            self.advance();
            let right = self.parse_equality()?;
            left = Expression::BinaryOp { op: BinaryOperator::And, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression> {
        let mut left = self.parse_instanceof()?;
        loop {
            let op = match self.peek() {
                Token::Equal => Some(BinaryOperator::Eq),
                Token::StrictEqual => Some(BinaryOperator::StrictEq),
                Token::NotEqual => Some(BinaryOperator::NotEqual),
                Token::StrictNotEqual => Some(BinaryOperator::StrictNotEqual),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_instanceof()?;
                left = Expression::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_instanceof(&mut self) -> Result<Expression> {
        let mut left = self.parse_in()?;
        while self.peek() == &Token::Instanceof {
            self.advance();
            let right = self.parse_in()?;
            left = Expression::BinaryOp { op: BinaryOperator::Instanceof, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_in(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison()?;
        while self.peek() == &Token::In {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::BinaryOp { op: BinaryOperator::In, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut left = self.parse_shift()?;
        loop {
            let op = match self.peek() {
                Token::Less => Some(BinaryOperator::Less),
                Token::Greater => Some(BinaryOperator::Greater),
                Token::LessEqual => Some(BinaryOperator::LessEqual),
                Token::GreaterEqual => Some(BinaryOperator::GreaterEqual),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_shift()?;
                left = Expression::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Expression> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Token::ShiftLeft => Some(BinaryOperator::ShiftLeft),
                Token::ShiftRight => Some(BinaryOperator::ShiftRight),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_additive()?;
                left = Expression::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expression> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Token::Plus => Some(BinaryOperator::Add),
                Token::Minus => Some(BinaryOperator::Sub),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_multiplicative()?;
                left = Expression::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression> {
        let mut left = self.parse_power()?;
        loop {
            let op = match self.peek() {
                Token::Star => Some(BinaryOperator::Mul),
                Token::Slash => Some(BinaryOperator::Div),
                Token::Percent => Some(BinaryOperator::Mod),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_power()?;
                left = Expression::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expression> {
        let left = self.parse_unary()?;
        if self.peek() == &Token::Power {
            self.advance();
            let right = self.parse_unary()?;
            Ok(Expression::BinaryOp { op: BinaryOperator::Power, left: Box::new(left), right: Box::new(right) })
        } else {
            Ok(left)
        }
    }

    fn parse_unary(&mut self) -> Result<Expression> {
        match self.peek().clone() {
            Token::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp { op: UnaryOperator::Negate, operand: Box::new(operand) })
            }
            Token::Not => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp { op: UnaryOperator::Not, operand: Box::new(operand) })
            }
            Token::Typeof => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp { op: UnaryOperator::Typeof, operand: Box::new(operand) })
            }
            Token::Void => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp { op: UnaryOperator::Void, operand: Box::new(operand) })
            }
            Token::Delete => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp { op: UnaryOperator::Delete, operand: Box::new(operand) })
            }
            Token::BitNot => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UnaryOp { op: UnaryOperator::BitNot, operand: Box::new(operand) })
            }
            Token::Increment => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UpdateExpression { op: UpdateOperator::Increment, operand: Box::new(operand), prefix: true })
            }
            Token::Decrement => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expression::UpdateExpression { op: UpdateOperator::Decrement, operand: Box::new(operand), prefix: true })
            }
            Token::New => self.parse_new_expression(),
            Token::Await => {
                self.advance();
                let argument = self.parse_unary()?;
                Ok(Expression::AwaitExpression { argument: Box::new(argument) })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression> {
        let expr = self.parse_call()?;
        match self.peek() {
            Token::Increment => {
                self.advance();
                Ok(Expression::UpdateExpression { op: UpdateOperator::Increment, operand: Box::new(expr), prefix: false })
            }
            Token::Decrement => {
                self.advance();
                Ok(Expression::UpdateExpression { op: UpdateOperator::Decrement, operand: Box::new(expr), prefix: false })
            }
            _ => Ok(expr),
        }
    }

    fn parse_new_expression(&mut self) -> Result<Expression> {
        self.expect(&Token::New)?;
        let callee = self.parse_new_target()?;
        let args = if self.peek() == &Token::LeftParen {
            self.advance();
            let a = self.parse_args()?;
            self.expect(&Token::RightParen)?;
            a
        } else {
            Vec::new()
        };
        Ok(Expression::NewExpression { callee: Box::new(callee), args })
    }

    fn parse_new_target(&mut self) -> Result<Expression> {
        match self.peek().clone() {
            Token::Identifier(name) => {
                self.advance();
                let mut expr = Expression::Identifier(name);
                while self.peek() == &Token::Dot {
                    self.advance();
                    let prop = match self.advance() {
                        Token::Identifier(n) => Expression::Identifier(n),
                        t => return Err(Error::ParseError(format!("Expected property name, got {:?}", t))),
                    };
                    expr = Expression::Member { object: Box::new(expr), property: Box::new(prop), computed: false };
                }
                Ok(expr)
            }
            _ => Err(Error::ParseError(format!("Expected identifier after 'new', got {:?}", self.peek()))),
        }
    }

    fn parse_call(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.peek() == &Token::LeftParen {
                self.advance();
                let args = self.parse_args()?;
                self.expect(&Token::RightParen)?;
                expr = Expression::Call { callee: Box::new(expr), args };
            } else if self.peek() == &Token::Dot {
                self.advance();
                let property = match self.advance() {
                    Token::Identifier(name) => Expression::Identifier(name),
                    token => return Err(Error::ParseError(format!("Expected property name, got {:?}", token))),
                };
                expr = Expression::Member { object: Box::new(expr), property: Box::new(property), computed: false };
            } else if self.peek() == &Token::LeftBracket {
                self.advance();
                let property = self.parse_expression()?;
                self.expect(&Token::RightBracket)?;
                expr = Expression::Member { object: Box::new(expr), property: Box::new(property), computed: true };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_args(&mut self) -> Result<Vec<Expression>> {
        let mut args = Vec::new();
        if self.peek() != &Token::RightParen {
            loop {
                args.push(self.parse_assignment()?);
                if self.peek() == &Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(Expression::NumberLiteral(n))
            }
            Token::String(s) => {
                self.advance();
                Ok(Expression::StringLiteral(s))
            }
            Token::TemplateLiteral(parts) => {
                self.advance();
                self.parse_template_literal(parts)
            }
            Token::Identifier(name) => {
                self.advance();
                match name.as_str() {
                    "true" => Ok(Expression::BooleanLiteral(true)),
                    "false" => Ok(Expression::BooleanLiteral(false)),
                    "null" => Ok(Expression::NullLiteral),
                    "undefined" => Ok(Expression::UndefinedLiteral),
                    _ => {
                        if self.peek() == &Token::Arrow {
                            self.advance();
                            self.parse_arrow_body(vec![name], false)
                        } else {
                            Ok(Expression::Identifier(name))
                        }
                    }
                }
            }
            Token::LeftParen => {
                self.advance();
                if self.peek() == &Token::RightParen {
                    self.advance();
                    if self.peek() == &Token::Arrow {
                        self.advance();
                        return self.parse_arrow_body(vec![], false);
                    }
                    return Err(Error::ParseError("Unexpected )".into()));
                }
                if let Token::Identifier(_) = self.peek().clone() {
                    let saved = self.pos;
                    let first = match self.advance() { Token::Identifier(n) => n, _ => unreachable!() };
                    if self.peek() == &Token::Arrow {
                        self.advance();
                        return self.parse_arrow_body(vec![first], false);
                    }
                    if self.peek() == &Token::Comma {
                        let mut params = vec![first];
                        loop {
                            if self.peek() != &Token::Comma { break; }
                            self.advance();
                            if self.peek() == &Token::RightParen { break; }
                            match self.advance() {
                                Token::Identifier(n) => params.push(n),
                                t => return Err(Error::ParseError(format!("Expected parameter, got {:?}", t))),
                            }
                        }
                        self.expect(&Token::RightParen)?;
                        if self.peek() == &Token::Arrow {
                            self.advance();
                            return self.parse_arrow_body(params, false);
                        }
                        self.pos = saved;
                    } else {
                        self.pos = saved;
                    }
                }
                let expr = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                if self.peek() == &Token::Arrow {
                    let params = match &expr {
                        Expression::Identifier(name) => vec![name.clone()],
                        _ => return Err(Error::ParseError("Invalid arrow function parameter".into())),
                    };
                    self.advance();
                    return self.parse_arrow_body(params, false);
                }
                Ok(expr)
            }
            Token::Function => {
                self.advance();
                let name = if let Token::Identifier(_) = self.peek().clone() {
                    match self.advance() { Token::Identifier(n) => Some(n), _ => unreachable!() }
                } else {
                    None
                };
                self.expect(&Token::LeftParen)?;
                let params = self.parse_params()?;
                self.expect(&Token::RightParen)?;
                self.expect(&Token::LeftBrace)?;
                let body = self.parse_block_body()?;
                self.expect(&Token::RightBrace)?;
                Ok(Expression::FunctionExpression { name, params, body, is_async: false })
            }
            Token::Async => {
                self.advance();
                if self.peek() == &Token::Function {
                    self.advance();
                    let name = if let Token::Identifier(_) = self.peek().clone() {
                        match self.advance() { Token::Identifier(n) => Some(n), _ => unreachable!() }
                    } else {
                        None
                    };
                    self.expect(&Token::LeftParen)?;
                    let params = self.parse_params()?;
                    self.expect(&Token::RightParen)?;
                    self.expect(&Token::LeftBrace)?;
                    let body = self.parse_block_body()?;
                    self.expect(&Token::RightBrace)?;
                    Ok(Expression::FunctionExpression { name, params, body, is_async: true })
                } else {
                    self.expect(&Token::LeftParen)?;
                    let params = self.parse_params()?;
                    self.expect(&Token::RightParen)?;
                    if self.peek() == &Token::Arrow {
                        self.advance();
                        self.parse_arrow_body(params, true)
                    } else {
                        Err(Error::ParseError("Expected '=>' after async parameters".into()))
                    }
                }
            }
            Token::Class => {
                self.advance();
                let name = if let Token::Identifier(_) = self.peek().clone() {
                    match self.advance() { Token::Identifier(n) => Some(n), _ => unreachable!() }
                } else {
                    None
                };
                let superclass = if self.peek() == &Token::Extends {
                    self.advance();
                    Some(self.parse_call()?)
                } else {
                    None
                };
                self.expect(&Token::LeftBrace)?;
                let body = self.parse_class_body()?;
                self.expect(&Token::RightBrace)?;
                Ok(Expression::ClassExpression { name, superclass: superclass.map(Box::new), body })
            }
            Token::Super => {
                self.advance();
                if self.peek() == &Token::LeftParen {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(&Token::RightParen)?;
                    Ok(Expression::SuperCall { args })
                } else if self.peek() == &Token::Dot {
                    self.advance();
                    let property = match self.advance() {
                        Token::Identifier(name) => Expression::Identifier(name),
                        t => return Err(Error::ParseError(format!("Expected property name after 'super', got {:?}", t))),
                    };
                    Ok(Expression::SuperMember { property: Box::new(property), computed: false })
                } else if self.peek() == &Token::LeftBracket {
                    self.advance();
                    let property = self.parse_expression()?;
                    self.expect(&Token::RightBracket)?;
                    Ok(Expression::SuperMember { property: Box::new(property), computed: true })
                } else {
                    Err(Error::ParseError("Expected '.' or '(' after 'super'".into()))
                }
            }
            Token::This => {
                self.advance();
                Ok(Expression::Identifier("this".into()))
            }
            Token::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                if self.peek() != &Token::RightBracket {
                    loop {
                        elements.push(self.parse_expression()?);
                        if self.peek() != &Token::Comma { break; }
                        self.advance();
                        if self.peek() == &Token::RightBracket { break; }
                    }
                }
                self.expect(&Token::RightBracket)?;
                Ok(Expression::ArrayLiteral { elements })
            }
            Token::LeftBrace => {
                self.advance();
                let mut properties = Vec::new();
                if self.peek() != &Token::RightBrace {
                    loop {
                        let key = match self.advance() {
                            Token::Identifier(n) => n,
                            Token::String(s) => s,
                            t => return Err(Error::ParseError(format!("Expected property key, got {:?}", t))),
                        };
                        self.expect(&Token::Colon)?;
                        let value = self.parse_expression()?;
                        properties.push((key, value));
                        if self.peek() != &Token::Comma { break; }
                        self.advance();
                        if self.peek() == &Token::RightBrace { break; }
                    }
                }
                self.expect(&Token::RightBrace)?;
                Ok(Expression::ObjectLiteral { properties })
            }
            token => Err(Error::ParseError(format!("Unexpected token {:?}", token))),
        }
    }

    fn parse_template_literal(&mut self, parts: Vec<TemplatePart>) -> Result<Expression> {
        let mut quasis = Vec::new();
        let mut expressions = Vec::new();
        let mut text_buf = String::new();
        for part in parts {
            match part {
                TemplatePart::Text(t) => text_buf.push_str(&t),
                TemplatePart::Expression(expr_tokens) => {
                    quasis.push(text_buf.clone());
                    text_buf.clear();
                    let mut sub_parser = Parser::new(&expr_tokens);
                    let expr = sub_parser.parse_expression()?;
                    expressions.push(expr);
                }
            }
        }
        quasis.push(text_buf);
        Ok(Expression::TemplateLiteral { quasis, expressions })
    }

    fn parse_arrow_body(&mut self, params: Vec<String>, is_async: bool) -> Result<Expression> {
        if self.peek() == &Token::LeftBrace {
            self.advance();
            let body = self.parse_block_body()?;
            self.expect(&Token::RightBrace)?;
            Ok(Expression::ArrowFunction { params, body: Box::new(ArrowFunctionBody::Block(body)), is_async })
        } else {
            let expr = self.parse_assignment()?;
            Ok(Expression::ArrowFunction { params, body: Box::new(ArrowFunctionBody::Expression(expr)), is_async })
        }
    }

    fn parse_for_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::For)?;
        self.expect(&Token::LeftParen)?;

        if self.peek() == &Token::Semicolon {
            self.advance();
            let condition = if self.peek() != &Token::Semicolon { Some(self.parse_expression()?) } else { None };
            self.expect(&Token::Semicolon)?;
            let update = if self.peek() != &Token::RightParen { Some(self.parse_expression()?) } else { None };
            self.expect(&Token::RightParen)?;
            let body = Box::new(self.parse_statement()?);
            return Ok(Statement::ForStatement { init: None, condition, update, body });
        }

        if self.peek() == &Token::Let || self.peek() == &Token::Const || self.peek() == &Token::Var {
            let kind = match self.peek() {
                Token::Var => VarKind::Var,
                Token::Let => VarKind::Let,
                Token::Const => VarKind::Const,
                _ => unreachable!(),
            };
            self.advance();
            let id = match self.peek().clone() {
                Token::Identifier(n) => { self.advance(); n }
                _ => return Err(Error::ParseError("Expected identifier in for-loop".into())),
            };
            if self.peek() == &Token::In {
                self.advance();
                let right = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForInStatement {
                    left: ForInLeft::VariableDeclaration { kind, id }, right, body,
                });
            }
            if self.peek() == &Token::Of {
                self.advance();
                let right = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForOfStatement {
                    left: ForInLeft::VariableDeclaration { kind, id }, right, body, is_async: false,
                });
            }
            let init_expr = Expression::Identifier(id);
            let mut declarations = Vec::new();
            let decl_id = match &init_expr {
                Expression::Identifier(n) => n.clone(),
                _ => unreachable!(),
            };
            let init_val = if self.peek() == &Token::Assign {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            declarations.push(VariableDeclarator { id: decl_id, init: init_val });
            let init = Some(Box::new(ForInit::Variable(Statement::VariableDeclaration { kind, declarations })));
            self.expect(&Token::Semicolon)?;
            let condition = if self.peek() != &Token::Semicolon { Some(self.parse_expression()?) } else { None };
            self.expect(&Token::Semicolon)?;
            let update = if self.peek() != &Token::RightParen { Some(self.parse_expression()?) } else { None };
            self.expect(&Token::RightParen)?;
            let body = Box::new(self.parse_statement()?);
            return Ok(Statement::ForStatement { init, condition, update, body });
        }

        if let Token::Identifier(id) = self.peek().clone() {
            self.advance();
            if self.peek() == &Token::In {
                self.advance();
                let right = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForInStatement {
                    left: ForInLeft::Identifier(id), right, body,
                });
            }
            if self.peek() == &Token::Of {
                self.advance();
                let right = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Statement::ForOfStatement {
                    left: ForInLeft::Identifier(id), right, body, is_async: false,
                });
            }
            self.pos -= 1;
        }

        let init_expr = self.parse_expression()?;
        let init = Some(Box::new(ForInit::Expression(init_expr)));
        self.expect(&Token::Semicolon)?;
        let condition = if self.peek() != &Token::Semicolon { Some(self.parse_expression()?) } else { None };
        self.expect(&Token::Semicolon)?;
        let update = if self.peek() != &Token::RightParen { Some(self.parse_expression()?) } else { None };
        self.expect(&Token::RightParen)?;
        let body = Box::new(self.parse_statement()?);
        Ok(Statement::ForStatement { init, condition, update, body })
    }

    fn parse_do_while_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::Do)?;
        let body = Box::new(self.parse_statement()?);
        self.expect(&Token::While)?;
        self.expect(&Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(&Token::RightParen)?;
        if self.peek() == &Token::Semicolon { self.advance(); }
        Ok(Statement::DoWhileStatement { condition, body })
    }

    fn parse_switch_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::Switch)?;
        self.expect(&Token::LeftParen)?;
        let discriminant = self.parse_expression()?;
        self.expect(&Token::RightParen)?;
        self.expect(&Token::LeftBrace)?;
        let mut cases = Vec::new();
        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            let test = if self.peek() == &Token::Case {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                self.expect(&Token::Default)?;
                None
            };
            self.expect(&Token::Colon)?;
            let mut consequent = Vec::new();
            while self.peek() != &Token::Case && self.peek() != &Token::Default
                && self.peek() != &Token::RightBrace && self.peek() != &Token::Eof
            {
                consequent.push(self.parse_statement()?);
            }
            cases.push(SwitchCase { test, consequent });
        }
        self.expect(&Token::RightBrace)?;
        Ok(Statement::SwitchStatement { discriminant, cases })
    }

    fn parse_try_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::Try)?;
        self.expect(&Token::LeftBrace)?;
        let block = self.parse_block_body()?;
        self.expect(&Token::RightBrace)?;

        let handler = if self.peek() == &Token::Catch {
            self.advance();
            self.expect(&Token::LeftParen)?;
            let param = match self.advance() {
                Token::Identifier(name) => name,
                t => return Err(Error::ParseError(format!("Expected parameter, got {:?}", t))),
            };
            self.expect(&Token::RightParen)?;
            self.expect(&Token::LeftBrace)?;
            let body = self.parse_block_body()?;
            self.expect(&Token::RightBrace)?;
            Some(CatchClause { param, body })
        } else {
            None
        };

        let finalizer = if self.peek() == &Token::Finally {
            self.advance();
            self.expect(&Token::LeftBrace)?;
            let body = self.parse_block_body()?;
            self.expect(&Token::RightBrace)?;
            Some(body)
        } else {
            None
        };

        Ok(Statement::TryStatement { block, handler, finalizer })
    }

    fn parse_throw_statement(&mut self) -> Result<Statement> {
        self.expect(&Token::Throw)?;
        let argument = self.parse_expression()?;
        self.expect(&Token::Semicolon)?;
        Ok(Statement::ThrowStatement(argument))
    }

    fn parse_class_declaration(&mut self) -> Result<Statement> {
        self.expect(&Token::Class)?;
        let name = match self.advance() {
            Token::Identifier(name) => name,
            t => return Err(Error::ParseError(format!("Expected class name, got {:?}", t))),
        };
        let superclass = if self.peek() == &Token::Extends {
            self.advance();
            Some(self.parse_call()?)
        } else {
            None
        };
        self.expect(&Token::LeftBrace)?;
        let body = self.parse_class_body()?;
        self.expect(&Token::RightBrace)?;
        Ok(Statement::ClassDeclaration { name, superclass: superclass.map(Box::new), body })
    }

    fn parse_class_body(&mut self) -> Result<Vec<ClassMember>> {
        let mut members = Vec::new();
        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            let is_static = if self.peek() == &Token::Static { self.advance(); true } else { false };
            let is_async = if self.peek() == &Token::Async { self.advance(); true } else { false };

            if self.peek() == &Token::Constructor {
                self.advance();
                self.expect(&Token::LeftParen)?;
                let params = self.parse_params()?;
                self.expect(&Token::RightParen)?;
                self.expect(&Token::LeftBrace)?;
                let body = self.parse_block_body()?;
                self.expect(&Token::RightBrace)?;
                members.push(ClassMember::Constructor { params, body });
            } else if self.peek() == &Token::Get && !is_async {
                self.advance();
                let name = match self.advance() {
                    Token::Identifier(name) => name,
                    t => return Err(Error::ParseError(format!("Expected property name after 'get', got {:?}", t))),
                };
                self.expect(&Token::LeftParen)?;
                self.expect(&Token::RightParen)?;
                self.expect(&Token::LeftBrace)?;
                let body = self.parse_block_body()?;
                self.expect(&Token::RightBrace)?;
                members.push(ClassMember::Getter { name, body, is_static });
            } else if self.peek() == &Token::Set && !is_async {
                self.advance();
                let name = match self.advance() {
                    Token::Identifier(name) => name,
                    t => return Err(Error::ParseError(format!("Expected property name after 'set', got {:?}", t))),
                };
                self.expect(&Token::LeftParen)?;
                let param = match self.advance() {
                    Token::Identifier(name) => name,
                    t => return Err(Error::ParseError(format!("Expected parameter name, got {:?}", t))),
                };
                self.expect(&Token::RightParen)?;
                self.expect(&Token::LeftBrace)?;
                let body = self.parse_block_body()?;
                self.expect(&Token::RightBrace)?;
                members.push(ClassMember::Setter { name, param, body, is_static });
            } else {
                let name = match self.advance() {
                    Token::Identifier(name) => name,
                    t => return Err(Error::ParseError(format!("Expected method name, got {:?}", t))),
                };
                if self.peek() == &Token::LeftParen {
                    self.advance();
                    let params = self.parse_params()?;
                    self.expect(&Token::RightParen)?;
                    self.expect(&Token::LeftBrace)?;
                    let body = self.parse_block_body()?;
                    self.expect(&Token::RightBrace)?;
                    members.push(ClassMember::Method { name, params, body, is_static, is_async });
                } else {
                    members.push(ClassMember::Property { name, is_static });
                    if self.peek() == &Token::Semicolon { self.advance(); }
                }
            }
        }
        Ok(members)
    }

    fn parse_import_declaration(&mut self) -> Result<Statement> {
        self.expect(&Token::Import)?;
        let mut specifiers = Vec::new();

        if matches!(self.peek(), Token::String(_)) {
            let source = match self.advance() { Token::String(s) => s, _ => unreachable!() };
            if self.peek() == &Token::Semicolon { self.advance(); }
            return Ok(Statement::ImportDeclaration { specifiers: vec![], source });
        }

        if self.peek() == &Token::LeftBrace {
            self.advance();
            while self.peek() != &Token::RightBrace {
                let imported = match self.advance() {
                    Token::Identifier(name) => name,
                    t => return Err(Error::ParseError(format!("Expected identifier, got {:?}", t))),
                };
                let local = if self.peek() == &Token::As {
                    self.advance();
                    match self.advance() {
                        Token::Identifier(name) => name,
                        t => return Err(Error::ParseError(format!("Expected identifier, got {:?}", t))),
                    }
                } else {
                    imported.clone()
                };
                specifiers.push(ImportSpecifier { local, imported: Some(imported) });
                if self.peek() == &Token::Comma { self.advance(); }
            }
            self.expect(&Token::RightBrace)?;
        } else         if self.peek() == &Token::Star {
            self.advance();
            self.expect(&Token::As)?;
            let local = match self.advance() {
                Token::Identifier(name) => name,
                t => return Err(Error::ParseError(format!("Expected identifier, got {:?}", t))),
            };
            specifiers.push(ImportSpecifier { local, imported: None });
        } else if matches!(self.peek(), Token::Identifier(_)) {
            let local = match self.advance() {
                Token::Identifier(name) => name,
                t => return Err(Error::ParseError(format!("Expected identifier, got {:?}", t))),
            };
            specifiers.push(ImportSpecifier { local: local.clone(), imported: Some(local) });
        }

        if self.peek() == &Token::From { self.advance(); }
        let source = match self.advance() {
            Token::String(s) => s,
            t => return Err(Error::ParseError(format!("Expected string, got {:?}", t))),
        };
        if self.peek() == &Token::Semicolon { self.advance(); }
        Ok(Statement::ImportDeclaration { specifiers, source })
    }

    fn parse_export_declaration(&mut self) -> Result<Statement> {
        self.expect(&Token::Export)?;

        if self.peek() == &Token::Default {
            self.advance();
            let decl = self.parse_statement()?;
            return Ok(Statement::ExportDefaultDeclaration { declaration: Box::new(decl) });
        }

        if self.peek() == &Token::LeftBrace {
            self.advance();
            while self.peek() != &Token::RightBrace {
                self.advance();
                if self.peek() == &Token::Comma { self.advance(); }
            }
            self.expect(&Token::RightBrace)?;
            if self.peek() == &Token::From { self.advance(); self.advance(); }
            if self.peek() == &Token::Semicolon { self.advance(); }
            return Ok(Statement::ExportDeclaration {
                declaration: Box::new(Statement::Expression(Expression::UndefinedLiteral)),
            });
        }

        let decl = self.parse_statement()?;
        Ok(Statement::ExportDeclaration { declaration: Box::new(decl) })
    }
}
