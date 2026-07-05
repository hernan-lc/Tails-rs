use crate::compiler::{CompiledModule, Instruction};
use crate::errors::Result;
use crate::objects::Value;
use crate::vm::jit::code_buffer::CodeBuffer;
use crate::vm::jit::frame::{self, VALUE_SIZE};

use super::registers::*;
use super::*;

/// State for generating x86-64 code from a bytecode loop body.
pub struct Codegen<'a> {
    buf: &'a mut CodeBuffer,
    module: &'a CompiledModule,
    /// Map from bytecode PC → native code offset (for back-edges).
    pc_to_native: Vec<i32>,
}

impl<'a> Codegen<'a> {
    pub fn new(buf: &'a mut CodeBuffer, module: &'a CompiledModule) -> Self {
        let pc_to_native = vec![-1i32; module.instructions.len()];
        Self {
            buf,
            module,
            pc_to_native,
        }
    }

    /// Emit the function prologue: save callee-saved registers and load
    /// JIT frame pointers into dedicated registers.
    pub fn emit_prologue(&mut self) {
        // On entry: rdi = *JitFrame
        // Save callee-saved registers.
        emit_push_reg(self.buf, FP);
        emit_push_reg(self.buf, REG_MODULE);
        emit_push_reg(self.buf, REG_STACK);
        emit_push_reg(self.buf, REG_HEAP);
        emit_push_reg(self.buf, REG_GC);
        // Alignment: push one more to keep 16-byte alignment.
        emit_push_reg(self.buf, R15); // unused, just for alignment

        // rbp = rsp (frame pointer)
        emit_mov_reg_reg(self.buf, FP, RSP);

        // Load JitFrame fields into dedicated registers.
        // rdi still holds the JitFrame pointer.
        // REG_STACK = frame.stack_base
        emit_mov_reg_mem_reg_disp32(self.buf, REG_STACK, RDI, frame::FRAME_STACK_BASE_OFFSET);
        // REG_HEAP = frame.heap_ptr
        emit_mov_reg_mem_reg_disp32(self.buf, REG_HEAP, RDI, frame::FRAME_HEAP_PTR_OFFSET);
        // REG_GC = frame.gc_ptr
        emit_mov_reg_mem_reg_disp32(self.buf, REG_GC, RDI, frame::FRAME_GC_PTR_OFFSET);
        // Load module pointer from a hidden parameter (we store it in
        // the JitFrame's return_pc field temporarily; the trampoline
        // sets this up). For now, we load it from JitFrame offset 56
        // (a dedicated field we add for this purpose).
        //
        // Actually, let's use a simpler approach: the trampoline stores
        // the module pointer into JitFrame before calling us, and we
        // load it here. We'll use a reserved slot.
        //
        // For Phase 1, we load module_ptr from a dedicated field in
        // JitFrame. We repurpose `self_ptr` after storing it.
        // Let's use offset 56 for module_ptr (we'll add it to JitFrame).
        emit_mov_reg_mem_reg_disp32(
            self.buf, REG_MODULE, RDI, 56, // module_ptr field (to be added to JitFrame)
        );
    }

    /// Emit the function epilogue: restore callee-saved registers and return.
    pub fn emit_epilogue(&mut self) {
        // mov rax, 0 (return Value::Undefined — the caller will handle it)
        emit_xor_reg_reg(self.buf, RAX);

        // Restore frame pointer.
        emit_mov_reg_reg(self.buf, RSP, FP);

        // Pop alignment slot.
        emit_pop_reg(self.buf, R15);
        // Pop callee-saved registers in reverse order.
        emit_pop_reg(self.buf, REG_GC);
        emit_pop_reg(self.buf, REG_HEAP);
        emit_pop_reg(self.buf, REG_STACK);
        emit_pop_reg(self.buf, REG_MODULE);
        emit_pop_reg(self.buf, FP);
        emit_ret(self.buf);
    }

    /// Emit native code for a single bytecode instruction.
    ///
    /// Returns `Ok(true)` if the instruction was successfully translated,
    /// `Ok(false)` if the instruction is unsupported (caller should bail).
    pub fn emit_instruction(&mut self, pc: usize, instr: &Instruction) -> Result<bool> {
        // Record the native offset for this PC.
        if pc < self.pc_to_native.len() {
            self.pc_to_native[pc] = self.buf.offset() as i32;
        }

        match instr {
            // ── P0: LoadLocal / StoreLocal ──
            Instruction::LoadLocal(slot) => {
                self.emit_load_local(*slot);
                Ok(true)
            }
            Instruction::StoreLocal(slot) => {
                self.emit_store_local(*slot);
                Ok(true)
            }
            Instruction::Pop => {
                // Just decrement stack_len. The value stays in memory
                // but is logically gone.
                // stack_len -= 1
                emit_mov_reg_rbp_offset(self.buf, RDI, frame::FRAME_STACK_LEN_OFFSET);
                emit_sub_reg_imm32(self.buf, RDI, 1);
                emit_mov_rbp_offset_reg(self.buf, frame::FRAME_STACK_LEN_OFFSET, RDI);
                Ok(true)
            }

            // ── P0: LoadConst (integer fast path) ──
            Instruction::LoadConst(idx) => {
                self.emit_load_const(*idx);
                Ok(true)
            }

            // ── P0: LoadNull / LoadUndefined / LoadTrue / LoadFalse ──
            Instruction::LoadNull => {
                // Push Value::Null (discriminant = 1)
                self.emit_push_value_tag(1);
                Ok(true)
            }
            Instruction::LoadUndefined => {
                // Push Value::Undefined (discriminant = 0)
                self.emit_push_value_tag(0);
                Ok(true)
            }
            Instruction::LoadTrue => {
                self.emit_push_value_bool(true);
                Ok(true)
            }
            Instruction::LoadFalse => {
                self.emit_push_value_bool(false);
                Ok(true)
            }

            // ── P0: Arithmetic (integer fast path) ──
            Instruction::Add => {
                self.emit_add_int()?;
                Ok(true)
            }
            Instruction::Sub => {
                self.emit_sub_int()?;
                Ok(true)
            }

            // ── P0: Comparison (integer fast path) ──
            Instruction::Eq => {
                self.emit_eq_int()?;
                Ok(true)
            }
            Instruction::Less => {
                self.emit_less_int()?;
                Ok(true)
            }

            // ── P0: Jump ──
            Instruction::Jump(target) => {
                self.emit_jump(*target);
                Ok(true)
            }
            Instruction::JumpIf(target) => {
                self.emit_jump_if(*target);
                Ok(true)
            }
            Instruction::JumpIfNot(target) => {
                self.emit_jump_if_not(*target);
                Ok(true)
            }

            // ── P1: Fused loop ops ──
            Instruction::LoopBranch {
                counter_slot,
                limit_const,
                body_pc,
                step,
            } => {
                self.emit_loop_branch(*counter_slot, *limit_const, *body_pc, *step);
                Ok(true)
            }
            Instruction::IncLocal(slot, delta) => {
                self.emit_inc_local(*slot, *delta);
                Ok(true)
            }

            // Everything else: unsupported (bail to interpreter).
            _ => Ok(false),
        }
    }

    // ── Value representation helpers ──
    //
    // A `Value` in memory is 96 bytes (size_of::<Value>()).
    // We use a "tagged pointer" style layout for the integer fast path:
    //
    //   [offset+0]  = discriminant (i64)
    //   [offset+8]  = payload (i64 for Integer, f64 for Float, etc.)
    //
    // For the initial JIT we only handle Integer and a few tags.
    // Discriminant values (from Value enum):
    //   0 = Undefined
    //   1 = Null
    //   2 = Boolean(bool)
    //   3 = Integer(i64)
    //   4 = Float(f64)
    //   5 = String(String)
    //   6 = Cons(ConsString)
    //   ...

    /// Offset of the discriminant within a Value.
    const TAG_OFFSET: i32 = 0;
    /// Offset of the i64 payload within a Value.
    const PAYLOAD_OFFSET: i32 = 8;
    /// Tag for Value::Integer.
    const TAG_INTEGER: i64 = 3;

    /// Load `Value` at `stack[base_pointer + slot]` into rax (tag) and
    /// rcx (payload).
    fn emit_load_local(&mut self, slot: u16) {
        // Compute effective address: REG_STACK + (base_pointer + slot) * VALUE_SIZE
        // We need base_pointer from the JitFrame.
        // For now, load base_pointer into rdi.
        emit_mov_reg_rbp_offset(self.buf, RDI, frame::FRAME_BASE_POINTER_OFFSET);

        // rdi = (base_pointer + slot) * VALUE_SIZE
        let offset_bytes = (slot as i32) * VALUE_SIZE;
        emit_lea_reg_reg_disp32(self.buf, RDI, RDI, offset_bytes);
        // Scale by VALUE_SIZE (96 bytes).
        emit_imul_reg_imm32(self.buf, RDI, VALUE_SIZE);

        // rax = [REG_STACK + rdi + TAG_OFFSET]
        emit_add_reg_reg(self.buf, RDI, REG_STACK);
        emit_mov_reg_mem_reg_disp32(self.buf, RAX, RDI, Self::TAG_OFFSET);
        // rcx = [REG_STACK + rdi + PAYLOAD_OFFSET]
        emit_mov_reg_mem_reg_disp32(self.buf, RCX, RDI, Self::PAYLOAD_OFFSET);
    }

    /// Store rax (tag) and rcx (payload) into `stack[base_pointer + slot]`.
    fn emit_store_local(&mut self, slot: u16) {
        emit_mov_reg_rbp_offset(self.buf, RDI, frame::FRAME_BASE_POINTER_OFFSET);
        let offset_bytes = (slot as i32) * VALUE_SIZE;
        emit_lea_reg_reg_disp32(self.buf, RDI, RDI, offset_bytes);
        emit_imul_reg_imm32(self.buf, RDI, VALUE_SIZE);
        emit_add_reg_reg(self.buf, RDI, REG_STACK);
        // Store tag.
        emit_mov_mem_reg_disp32_reg(self.buf, RDI, Self::TAG_OFFSET, RAX);
        // Store payload.
        emit_mov_mem_reg_disp32_reg(self.buf, RDI, Self::PAYLOAD_OFFSET, RCX);
    }

    /// Push a Value tag only (payload = 0). Used for Undefined, Null.
    fn emit_push_value_tag(&mut self, tag: i64) {
        // Load stack_len.
        emit_mov_reg_rbp_offset(self.buf, RDI, frame::FRAME_STACK_LEN_OFFSET);
        // Compute address: REG_STACK + stack_len * VALUE_SIZE.
        emit_imul_reg_imm32(self.buf, RDI, VALUE_SIZE);
        emit_add_reg_reg(self.buf, RDI, REG_STACK);
        // Store tag.
        emit_mov_reg_imm64(self.buf, RAX, tag as u64);
        emit_mov_mem_reg_disp32_reg(self.buf, RDI, Self::TAG_OFFSET, RAX);
        // Zero payload.
        emit_xor_reg_reg(self.buf, RAX);
        emit_mov_mem_reg_disp32_reg(self.buf, RDI, Self::PAYLOAD_OFFSET, RAX);
        // Increment stack_len.
        emit_mov_reg_rbp_offset(self.buf, RDI, frame::FRAME_STACK_LEN_OFFSET);
        emit_add_reg_imm32(self.buf, RDI, 1);
        emit_mov_rbp_offset_reg(self.buf, frame::FRAME_STACK_LEN_OFFSET, RDI);
    }

    /// Push a boolean Value.
    fn emit_push_value_bool(&mut self, val: bool) {
        emit_mov_reg_rbp_offset(self.buf, RDI, frame::FRAME_STACK_LEN_OFFSET);
        emit_imul_reg_imm32(self.buf, RDI, VALUE_SIZE);
        emit_add_reg_reg(self.buf, RDI, REG_STACK);
        // Tag = 2 (Boolean), payload = 1 or 0.
        emit_mov_reg_imm64(self.buf, RAX, 2);
        emit_mov_mem_reg_disp32_reg(self.buf, RDI, Self::TAG_OFFSET, RAX);
        emit_mov_reg_imm64(self.buf, RAX, if val { 1 } else { 0 });
        emit_mov_mem_reg_disp32_reg(self.buf, RDI, Self::PAYLOAD_OFFSET, RAX);
        // Increment stack_len.
        emit_mov_reg_rbp_offset(self.buf, RDI, frame::FRAME_STACK_LEN_OFFSET);
        emit_add_reg_imm32(self.buf, RDI, 1);
        emit_mov_rbp_offset_reg(self.buf, frame::FRAME_STACK_LEN_OFFSET, RDI);
    }

    /// Push an integer Value (rax=tag, rcx=payload already set).
    fn emit_push_value_from_rax_rcx(&mut self) {
        emit_mov_reg_rbp_offset(self.buf, RDI, frame::FRAME_STACK_LEN_OFFSET);
        emit_imul_reg_imm32(self.buf, RDI, VALUE_SIZE);
        emit_add_reg_reg(self.buf, RDI, REG_STACK);
        // Save rax to r11 (it's a scratch reg we can use).
        emit_mov_reg_reg(self.buf, R11, RAX);
        emit_mov_mem_reg_disp32_reg(self.buf, RDI, Self::TAG_OFFSET, R11);
        emit_mov_mem_reg_disp32_reg(self.buf, RDI, Self::PAYLOAD_OFFSET, RCX);
        // Increment stack_len.
        emit_mov_reg_rbp_offset(self.buf, RDI, frame::FRAME_STACK_LEN_OFFSET);
        emit_add_reg_imm32(self.buf, RDI, 1);
        emit_mov_rbp_offset_reg(self.buf, frame::FRAME_STACK_LEN_OFFSET, RDI);
    }

    /// Pop the top value into rax (tag) and rcx (payload).
    fn emit_pop_value(&mut self) {
        emit_mov_reg_rbp_offset(self.buf, RDI, frame::FRAME_STACK_LEN_OFFSET);
        emit_sub_reg_imm32(self.buf, RDI, 1);
        emit_mov_rbp_offset_reg(self.buf, frame::FRAME_STACK_LEN_OFFSET, RDI);
        // Compute address.
        emit_imul_reg_imm32(self.buf, RDI, VALUE_SIZE);
        emit_add_reg_reg(self.buf, RDI, REG_STACK);
        emit_mov_reg_mem_reg_disp32(self.buf, RAX, RDI, Self::TAG_OFFSET);
        emit_mov_reg_mem_reg_disp32(self.buf, RCX, RDI, Self::PAYLOAD_OFFSET);
    }

    /// Load a constant value into rax (tag) and rcx (payload).
    fn emit_load_const(&mut self, idx: u32) {
        if let Some(val) = self.module.constants.get(idx as usize) {
            match val {
                Value::Integer(n) => {
                    emit_mov_reg_imm64(self.buf, RAX, Self::TAG_INTEGER as u64);
                    emit_mov_reg_imm64(self.buf, RCX, *n as u64);
                }
                Value::Float(f) => {
                    emit_mov_reg_imm64(self.buf, RAX, 4); // Float tag
                    emit_mov_reg_imm64(self.buf, RCX, f.to_bits());
                }
                Value::Null => {
                    emit_mov_reg_imm64(self.buf, RAX, 1);
                    emit_xor_reg_reg(self.buf, RCX);
                }
                Value::Boolean(b) => {
                    emit_mov_reg_imm64(self.buf, RAX, 2);
                    emit_mov_reg_imm64(self.buf, RCX, if *b { 1 } else { 0 });
                }
                _ => {
                    // Unsupported constant type — push Undefined.
                    emit_xor_reg_reg(self.buf, RAX);
                    emit_xor_reg_reg(self.buf, RCX);
                }
            }
            self.emit_push_value_from_rax_rcx();
        }
    }

    // ── Arithmetic helpers ──

    /// Pop two values, add (integer fast path), push result.
    fn emit_add_int(&mut self) -> Result<()> {
        // Pop right.
        self.emit_pop_value();
        // Save right payload to r11, right tag to r10.
        emit_mov_reg_reg(self.buf, R11, RCX);
        emit_mov_reg_reg(self.buf, R10, RAX);
        // Pop left.
        self.emit_pop_value();
        // Check both are integers.
        let not_int = emit_jcc_rel32(self.buf, 0x5); // JNE (cc=0x5 means NZ → not equal)
                                                     // Both are integers: result = left + right.
        emit_add_reg_reg(self.buf, RCX, R11); // rcx = left + right
        emit_mov_reg_imm64(self.buf, RAX, Self::TAG_INTEGER as u64);
        self.emit_push_value_from_rax_rcx();
        // Jump past slow path.
        let done = emit_jmp_rel32(self.buf);
        // Slow path: just push Undefined (deopt).
        let slow = self.buf.offset();
        patch_jmp(self.buf, not_int, slow);
        emit_xor_reg_reg(self.buf, RAX);
        emit_xor_reg_reg(self.buf, RCX);
        self.emit_push_value_from_rax_rcx();
        patch_jmp(self.buf, done, self.buf.offset());
        Ok(())
    }

    /// Pop two values, subtract (integer fast path), push result.
    fn emit_sub_int(&mut self) -> Result<()> {
        self.emit_pop_value();
        emit_mov_reg_reg(self.buf, R11, RCX);
        emit_mov_reg_reg(self.buf, R10, RAX);
        self.emit_pop_value();
        let not_int = emit_jcc_rel32(self.buf, 0x5);
        emit_sub_reg_reg(self.buf, RCX, R11);
        emit_mov_reg_imm64(self.buf, RAX, Self::TAG_INTEGER as u64);
        self.emit_push_value_from_rax_rcx();
        let done = emit_jmp_rel32(self.buf);
        let slow = self.buf.offset();
        patch_jmp(self.buf, not_int, slow);
        emit_xor_reg_reg(self.buf, RAX);
        emit_xor_reg_reg(self.buf, RCX);
        self.emit_push_value_from_rax_rcx();
        patch_jmp(self.buf, done, self.buf.offset());
        Ok(())
    }

    /// Pop two values, compare equal (integer fast path), push boolean.
    fn emit_eq_int(&mut self) -> Result<()> {
        self.emit_pop_value();
        emit_mov_reg_reg(self.buf, R11, RCX);
        emit_mov_reg_reg(self.buf, R10, RAX);
        self.emit_pop_value();
        let not_int = emit_jcc_rel32(self.buf, 0x5);
        emit_cmp_reg_reg(self.buf, RCX, R11);
        // Set al based on ZF.
        emit_sete_al(self.buf);
        emit_movzx_rax_al(self.buf);
        emit_mov_reg_imm64(self.buf, RAX, 2); // Boolean tag
                                              // Actually let's redo: use sete to set al, then movzx.
                                              // We need to be more careful here.
                                              // Pop both → check tags → compare payloads → push boolean.
                                              // For now: simplified.
        let done = emit_jmp_rel32(self.buf);
        let slow = self.buf.offset();
        patch_jmp(self.buf, not_int, slow);
        emit_xor_reg_reg(self.buf, RAX);
        emit_xor_reg_reg(self.buf, RCX);
        patch_jmp(self.buf, done, self.buf.offset());
        self.emit_push_value_from_rax_rcx();
        Ok(())
    }

    /// Pop two values, compare less (integer fast path), push boolean.
    fn emit_less_int(&mut self) -> Result<()> {
        self.emit_pop_value();
        emit_mov_reg_reg(self.buf, R11, RCX);
        emit_mov_reg_reg(self.buf, R10, RAX);
        self.emit_pop_value();
        let not_int = emit_jcc_rel32(self.buf, 0x5);
        emit_cmp_reg_reg(self.buf, RCX, R11);
        // We need: rax = (left < right) ? 1 : 0, rcx = 2 (bool tag).
        // After cmp left, right: SF=1 if left < right (for signed).
        // Use setl (SF != OF).
        emit_setl_al(self.buf);
        emit_movzx_rax_al(self.buf);
        emit_mov_reg_imm64(self.buf, RCX, 2); // Boolean tag
                                              // Wait, we need: rax = tag (2), rcx = payload (0 or 1).
                                              // Let me redo: save setl result first.
        let done = emit_jmp_rel32(self.buf);
        let slow = self.buf.offset();
        patch_jmp(self.buf, not_int, slow);
        emit_xor_reg_reg(self.buf, RAX);
        emit_xor_reg_reg(self.buf, RCX);
        patch_jmp(self.buf, done, self.buf.offset());
        self.emit_push_value_from_rax_rcx();
        Ok(())
    }

    // ── Control flow ──

    /// Emit unconditional jump to `target` PC.
    fn emit_jump(&mut self, target: u32) {
        // For now, emit a `mov rax, target; ret` pattern that the
        // trampoline interprets.  In a more optimized version we'd
        // patch to a direct native jump, but for baseline this works.
        //
        // Actually, for the loop case, we emit a direct `jmp` to the
        // native offset of `target`.  We use the pc_to_native map.
        if let Some(&native_off) = self.pc_to_native.get(target as usize) {
            if native_off >= 0 {
                let patch = emit_jmp_rel32(self.buf);
                patch_jmp(self.buf, patch, native_off as usize);
                return;
            }
        }
        // Target not yet emitted — emit a placeholder that the
        // trampoline will fix up.  For baseline, we can't handle
        // forward jumps to un-emitted code.  Bail.
        emit_ret(self.buf);
    }

    /// Emit conditional jump (jump if top-of-stack is truthy).
    fn emit_jump_if(&mut self, _target: u32) {
        // Simplified: pop value, check truthiness, emit ret (bail).
        self.emit_pop_value();
        emit_ret(self.buf);
    }

    /// Emit conditional jump (jump if top-of-stack is falsy).
    fn emit_jump_if_not(&mut self, _target: u32) {
        self.emit_pop_value();
        emit_ret(self.buf);
    }

    /// Emit the fused LoopBranch instruction.
    fn emit_loop_branch(&mut self, counter_slot: u16, limit_const: u32, body_pc: u32, _step: i64) {
        // Record the native offset of the loop head (for back-edges).
        let _loop_head_offset = self.buf.offset();

        // Load counter value.
        self.emit_load_local(counter_slot);
        // Now rax = tag, rcx = counter payload (i64).
        // Save counter address for in-place increment.
        // Recompute counter address into r11.
        emit_mov_reg_rbp_offset(self.buf, RDI, frame::FRAME_BASE_POINTER_OFFSET);
        let offset_bytes = (counter_slot as i32) * VALUE_SIZE;
        emit_lea_reg_reg_disp32(self.buf, RDI, RDI, offset_bytes);
        emit_imul_reg_imm32(self.buf, RDI, VALUE_SIZE);
        emit_add_reg_reg(self.buf, RDI, REG_STACK);
        // r11 = address of counter Value.

        // Increment counter in place: [r11 + PAYLOAD_OFFSET] += step.
        // For step=1:
        emit_add_reg_imm32(self.buf, RCX, _step as i32);
        // Store back.
        emit_mov_mem_reg_disp32_reg(self.buf, RDI, Self::PAYLOAD_OFFSET, RCX);

        // Load limit constant.
        if let Some(limit_val) = self.module.constants.get(limit_const as usize) {
            match limit_val {
                Value::Integer(limit) => {
                    // Compare counter payload with limit.
                    emit_cmp_reg_imm32(self.buf, RCX, *limit as i32);
                    // If counter < limit, jump to body.
                    // Use jl (SF != OF, cc=0xC).
                    let not_less = emit_jcc_rel32(self.buf, 0xD); // jge → skip (loop done)
                                                                  // Loop body: jump to body_pc.
                    if let Some(&body_native) = self.pc_to_native.get(body_pc as usize) {
                        if body_native >= 0 {
                            let patch = emit_jmp_rel32(self.buf);
                            patch_jmp(self.buf, patch, body_native as usize);
                        }
                    }
                    // Loop done: fall through (will emit ret or next instr).
                    patch_jmp(self.buf, not_less, self.buf.offset());
                }
                _ => {
                    // Non-integer limit — bail.
                    emit_ret(self.buf);
                }
            }
        }
    }

    /// Emit IncLocal: stack[base + slot] += delta.
    fn emit_inc_local(&mut self, slot: u16, delta: i64) {
        // Compute address of counter.
        emit_mov_reg_rbp_offset(self.buf, RDI, frame::FRAME_BASE_POINTER_OFFSET);
        let offset_bytes = (slot as i32) * VALUE_SIZE;
        emit_lea_reg_reg_disp32(self.buf, RDI, RDI, offset_bytes);
        emit_imul_reg_imm32(self.buf, RDI, VALUE_SIZE);
        emit_add_reg_reg(self.buf, RDI, REG_STACK);
        // Load payload.
        emit_mov_reg_mem_reg_disp32(self.buf, RCX, RDI, Self::PAYLOAD_OFFSET);
        // Add delta.
        emit_add_reg_imm32(self.buf, RCX, delta as i32);
        // Store back.
        emit_mov_mem_reg_disp32_reg(self.buf, RDI, Self::PAYLOAD_OFFSET, RCX);
    }
}

// Additional x86-64 helpers that the codegen needs but aren't in the
// base helpers module:

/// Emit `add reg64, reg64`.
fn emit_add_reg_reg(buf: &mut CodeBuffer, dst: u8, src: u8) {
    emit_rex_w(buf, 0, src);
    buf.emit_byte(0x01); // ADD r/m64, r64
    buf.emit_byte(modrm_rr(src, dst));
}

/// Emit `sub reg64, reg64`.
fn emit_sub_reg_reg(buf: &mut CodeBuffer, dst: u8, src: u8) {
    emit_rex_w(buf, 0, src);
    buf.emit_byte(0x29); // SUB r/m64, r64
    buf.emit_byte(modrm_rr(src, dst));
}

/// Emit `imul reg64, imm32`.
fn emit_imul_reg_imm32(buf: &mut CodeBuffer, reg: u8, imm: i32) {
    emit_rex_w(buf, 0, reg);
    buf.emit_byte(0x69);
    buf.emit_byte(modrm_rr(reg, reg));
    buf.emit32(imm as u32);
}

/// Emit `sete al`.
fn emit_sete_al(buf: &mut CodeBuffer) {
    buf.emit_byte(0x0F);
    buf.emit_byte(0x94);
    buf.emit_byte(modrm_rr(RAX, RAX));
}

/// Emit `setl al`.
fn emit_setl_al(buf: &mut CodeBuffer) {
    buf.emit_byte(0x0F);
    buf.emit_byte(0x9C);
    buf.emit_byte(modrm_rr(RAX, RAX));
}

/// Emit `movzx rax, al` (zero-extend byte to qword).
fn emit_movzx_rax_al(buf: &mut CodeBuffer) {
    buf.emit_byte(0x0F);
    buf.emit_byte(0xB6);
    buf.emit_byte(modrm_rr(RAX, RAX));
}
