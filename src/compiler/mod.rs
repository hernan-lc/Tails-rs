use crate::compiler::type_checker::Type;
use crate::errors::Result;
use crate::objects::Value;
use rustc_hash::FxHashMap;

pub mod bytecode;
pub mod lexer;
pub mod parser;
pub mod type_checker;

pub struct Compiler {
    type_checking: bool,
    known_globals: FxHashMap<String, Type>,
}

impl Compiler {
    pub fn new(type_checking: bool) -> Self {
        Self {
            type_checking,
            known_globals: FxHashMap::default(),
        }
    }

    pub fn with_globals(type_checking: bool, known_globals: FxHashMap<String, Type>) -> Self {
        Self {
            type_checking,
            known_globals,
        }
    }

    pub fn add_global(&mut self, name: String, ty: Type) {
        self.known_globals.insert(name, ty);
    }

    pub fn set_known_globals(&mut self, globals: FxHashMap<String, Type>) {
        self.known_globals = globals;
    }

    pub fn compile(&self, source: &str) -> Result<CompiledModule> {
        let mut tokens = lexer::tokenize(source)?;
        let ast = parser::parse(&mut tokens)?;

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
    pub rest_param: Option<String>,
    pub bytecode_index: usize,
    pub param_count: usize,
    pub closure_var_count: usize,
    pub is_generator: bool,
    pub source_line: Option<usize>,
    pub is_arrow: bool,
}

#[cfg(test)]
mod instruction_size_regression {
    //! Phase 1D: regression test for the size of the `Instruction`
    //! enum. Boxing the `Vec<u16>` and `Vec<String>` payloads
    //! (see the `MakeClosure` / `ExportNamed` variants below) is what
    //! keeps the variant size from dominating the dispatch table.
    #[test]
    fn instruction_enum_size_is_bounded() {
        use std::mem::size_of;
        let s = size_of::<super::Instruction>();
        // After Phase 1D, the largest single-payload variant is
        // `ImportNamed(String, String, String)` at 72 bytes; with
        // `Box<Vec<u16>>` and `Box<Vec<String>>` the rest are <=24
        // bytes. The total enum size is set by the largest variant.
        // We allow some headroom for future variants and to make the
        // test stable across rustc versions that may add niche
        // optimisations.
        assert!(
            s <= 80,
            "Instruction grew past the Phase 1D budget (got {s} bytes, expected <= 80); a new variant was probably added without boxing its payload"
        );
    }
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
    IncLocal(u16, i64),
    AddLocal(u16, u16),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Power,
    Negate,
    Not,
    BitNot,
    UnaryPlus,
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
    // Phase 1D: `Vec<u16>` boxed so the instruction stays at 8 bytes
    // (it would otherwise be 24 bytes, dominating the enum size).
    // Inner `Vec` is heap-allocated once at compile time and read-only
    // at run time, so the extra indirection is a net win.
    MakeClosure(u32, Box<Vec<u16>>),
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
    // Phase 1D: same rationale as `MakeClosure` above — the export
    // name list is heap-allocated once at compile time and read-only
    // at run time, so a boxed `Vec<String>` keeps this variant at 8
    // bytes instead of 24.
    ExportNamed(Box<Vec<String>>),
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
    ReExportAll(String),
    // Map/Set fast-path bytecodes
    MapGet,
    MapSet(u16),
    MapHas,
    MapDelete,
    SetAdd,
    SetHas,
    SetDelete,
}
