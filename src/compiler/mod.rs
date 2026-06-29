use crate::compiler::type_checker::Type;
use crate::errors::Result;
use crate::objects::Value;
use std::collections::HashMap;

pub mod bytecode;
pub mod lexer;
pub mod parser;
pub mod type_checker;

pub struct Compiler {
    type_checking: bool,
    known_globals: HashMap<String, Type>,
}

impl Compiler {
    pub fn new(type_checking: bool) -> Self {
        Self {
            type_checking,
            known_globals: HashMap::new(),
        }
    }

    pub fn with_globals(type_checking: bool, known_globals: HashMap<String, Type>) -> Self {
        Self {
            type_checking,
            known_globals,
        }
    }

    pub fn add_global(&mut self, name: String, ty: Type) {
        self.known_globals.insert(name, ty);
    }

    pub fn set_known_globals(&mut self, globals: HashMap<String, Type>) {
        self.known_globals = globals;
    }

    pub fn compile(&self, source: &str) -> Result<CompiledModule> {
        let tokens = lexer::tokenize(source)?;
        let ast = parser::parse(&tokens)?;

        if self.type_checking {
            type_checker::TypeChecker::check_with_globals(&ast, self.known_globals.clone())?;
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
    pub source_lines: Vec<Option<usize>>,
    pub source_cols: Vec<Option<usize>>,
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
    pub is_generator: bool,
    pub source_line: Option<usize>,
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
    JumpIfUndefined(u32),
    JumpIfNotUndefined(u32),
    Call(u16),
    CallMethod(u16),
    Construct(u16),
    LoadThis,
    Dup,
    Rot3Right,
    Return,
    Pop,
    MakeFunction(u32),
    MakeClosure(u32, Vec<u16>),
    NewObject,
    SetProperty,
    GetProperty,
    OptionalGetProperty,
    OptionalCall(u16),
    NullishCoalescing,
    NewArray(u32),
    ArrayPush,
    SpreadArray,
    SpreadObject,
    GetKeys,
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
    ImportModule(String),
    ImportNamed(String, String, String),
    ImportDefault(String, String),
    ImportAll(String, String),
    NativeImport(String, String),
    ExportNamed(Vec<String>),
    ExportDefault,
    StoreModuleExport(String),
    PopModuleExports,
    Await,
    DynamicImport,
    Yield,
    BlockEnter,
    BlockExit,
    LoadGlobalOrUndefined(String),
    TypeOfGlobal(String),
    GetIterator,
    GetAsyncIterator,
    IteratorNext(u32),
    AsyncIteratorNext(u32),
    IteratorClose,
}
