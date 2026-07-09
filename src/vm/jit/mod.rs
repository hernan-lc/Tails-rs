pub mod code_buffer;
pub mod frame;
pub mod trampoline;
pub mod x86_64;

use crate::compiler::{CompiledModule, Instruction};
use crate::errors::{Error, Result};
use code_buffer::CodeBuffer;
use rustc_hash::FxHashMap;

/// Profiling state for a single bytecode PC.
#[derive(Default)]
struct PcProfile {
    hit_count: u32,
}

/// Baseline JIT compiler for hot loops.
///
/// Detects loops whose back-edges are executed repeatedly and compiles
/// their bodies to x86-64 native code.  The compiled code operates on
/// the same `Vec<Value>` stack as the interpreter, so there is no
/// overhead for value boxing/unboxing.
pub struct JitCompiler {
    /// Per-PC execution counters.
    profiles: Vec<PcProfile>,
    /// Compilation threshold — a loop back-edge must be taken this many
    /// times before we compile.
    threshold: u32,
    /// Map from bytecode PC → compiled native entry point.
    compiled: FxHashMap<usize, extern "C" fn(*mut frame::JitFrame) -> i64>,
    /// Whether JIT compilation is enabled.
    enabled: bool,
}

impl Default for JitCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl JitCompiler {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            // Lowered from 1000 so short hot loops (≤ a few hundred iters)
            // still get compiled; tick still fires every 128 back-edges.
            threshold: 100,
            compiled: FxHashMap::default(),
            enabled: true,
        }
    }

    /// Ensure the profile array is large enough for the given module.
    fn ensure_capacity(&mut self, module: &CompiledModule) {
        let needed = module.instructions.len();
        if self.profiles.len() < needed {
            self.profiles.resize_with(needed, PcProfile::default);
        }
    }

    /// Record a hit at the given bytecode PC.  Returns `Some(native_fn)`
    /// if compilation was triggered and the result is ready to call.
    pub fn tick(
        &mut self,
        pc: usize,
        module: &CompiledModule,
    ) -> Option<extern "C" fn(*mut frame::JitFrame) -> i64> {
        if !self.enabled {
            return None;
        }
        self.ensure_capacity(module);
        let profile = &mut self.profiles[pc];
        profile.hit_count += 1;
        if profile.hit_count >= self.threshold {
            // Already compiled?
            if let Some(&entry) = self.compiled.get(&pc) {
                return Some(entry);
            }
            // Find the loop body range (pc back to LoopBranch or start).
            let loop_range = Self::find_loop_range(module, pc);
            if let Some((start, end)) = loop_range {
                if let Ok(entry) = self.compile_loop(module, start, end) {
                    self.compiled.insert(pc, entry);
                    return Some(entry);
                }
            }
        }
        None
    }

    /// Look up a previously compiled entry point for `pc`.
    pub fn get_compiled(&self, pc: usize) -> Option<extern "C" fn(*mut frame::JitFrame) -> i64> {
        self.compiled.get(&pc).copied()
    }

    /// Returns true if JIT is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable JIT.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Given a LoopBranch PC, find the contiguous bytecode range of the
    /// loop body (from the instruction after the loop init to the
    /// LoopBranch itself).
    fn find_loop_range(module: &CompiledModule, loop_branch_pc: usize) -> Option<(usize, usize)> {
        if loop_branch_pc >= module.instructions.len() {
            return None;
        }
        match &module.instructions[loop_branch_pc] {
            Instruction::LoopBranch { body_pc, .. } => {
                Some((*body_pc as usize, loop_branch_pc + 1))
            }
            _ => None,
        }
    }

    /// Compile a loop body (bytecode range) to native code.
    fn compile_loop(
        &mut self,
        module: &CompiledModule,
        start_pc: usize,
        end_pc: usize,
    ) -> Result<extern "C" fn(*mut frame::JitFrame) -> i64> {
        let mut buf = CodeBuffer::new(4096);
        let mut gen = x86_64::codegen::Codegen::new(&mut buf, module);

        // Emit function prologue.
        gen.emit_prologue();

        // Translate each instruction in the loop body.
        let mut pc = start_pc;
        while pc < end_pc {
            let instr = &module.instructions[pc];
            let emitted = gen.emit_instruction(pc, instr)?;
            if !emitted {
                // Unsupported instruction — bail out and fall back to
                // interpreter for this loop.
                return Err(Error::RuntimeError(
                    "JIT: unsupported instruction in loop body".into(),
                ));
            }
            pc += 1;
        }

        // Emit function epilogue.
        gen.emit_epilogue();

        let entry = buf.finalize();
        // SAFETY: we just emitted valid machine code and mprotect'd
        // the page to RX.
        Ok(unsafe {
            std::mem::transmute::<*const u8, extern "C" fn(*mut frame::JitFrame) -> i64>(entry)
        })
    }
}
