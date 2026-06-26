use crate::errors::Result;
use crate::objects::Value;

pub mod lexer;
pub mod parser;
pub mod type_checker;
pub mod bytecode;

pub struct Compiler {
    type_checking: bool,
}

impl Compiler {
    pub fn new(type_checking: bool) -> Self {
        Self { type_checking }
    }

    pub fn compile(&self, source: &str) -> Result<CompiledModule> {
        let tokens = lexer::tokenize(source)?;
        let ast = parser::parse(&tokens)?;

        if self.type_checking {
            type_checker::TypeChecker::check(&ast)?;
        }

        bytecode::generate(&ast)
    }
}

#[derive(Debug, Clone)]
pub struct CompiledModule {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub functions: Vec<CompiledFunction>,
    pub class_infos: Vec<ClassInfo>,
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub constructor_func_idx: Option<u32>,
    pub methods: Vec<ClassMethodInfo>,
    pub superclass: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClassMethodInfo {
    pub name: String,
    pub func_idx: u32,
    pub is_static: bool,
    pub kind: ClassMethodKind,
}

#[derive(Debug, Clone)]
pub enum ClassMethodKind {
    Method,
    Getter,
    Setter,
}

#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub bytecode_index: usize,
    pub param_count: usize,
    pub closure_var_count: usize,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    LoadConst(u32),
    LoadNull,
    LoadUndefined,
    LoadTrue,
    LoadFalse,
    StoreGlobal(String),
    LoadGlobal(String),
    StoreLocal(u16),
    LoadLocal(u16),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Power,
    Negate,
    Not,
    BitNot,
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
    Jump(u32),
    JumpIf(u32),
    JumpIfNot(u32),
    Call(u16),
    CallMethod(u16),
    Construct(u16),
    LoadThis,
    Dup,
    Return,
    Pop,
    MakeFunction(u32),
    MakeClosure(u32, Vec<u16>),
    NewObject,
    SetProperty,
    GetProperty,
    NewArray(u32),
    ArrayPush,
    TypeOf,
    InstanceOf,
    In,
    Delete,
    Void,
    Throw,
    MakeClass(u32),
    SuperConstruct(u16),
    SuperGet,
    ToString,
    TryJump(u32, u32),
    PopTryHandler,
    LoadException,
    ReThrowIfPending,
    NotImplementedError(String),
}
