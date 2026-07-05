pub mod codegen;
pub mod registers;

use crate::vm::jit::code_buffer::CodeBuffer;

// Register encoding (3-bit, used in ModR/M and REX).
pub const RAX: u8 = 0;
pub const RCX: u8 = 1;
pub const RDX: u8 = 2;
pub const RBX: u8 = 3;
pub const RSP: u8 = 4;
pub const RBP: u8 = 5;
pub const RSI: u8 = 6;
pub const RDI: u8 = 7;
pub const R8: u8 = 0;
pub const R9: u8 = 1;
pub const R10: u8 = 2;
pub const R11: u8 = 3;
pub const R12: u8 = 4;
pub const R13: u8 = 5;
pub const R14: u8 = 6;
pub const R15: u8 = 7;

/// Emit a REX.W prefix (for 64-bit operand size).
pub fn emit_rex_w(buf: &mut CodeBuffer, reg: u8, rm: u8) {
    // REX.W = 0x48, plus REX.R and REX.B bits if needed.
    let mut rex: u8 = 0x48;
    if reg >= 8 {
        rex |= 0x04; // REX.R
    }
    if rm >= 8 {
        rex |= 0x01; // REX.B
    }
    buf.emit_byte(rex);
}

/// Emit a REX prefix without W (for 8-bit or 32-bit when needed).
pub fn emit_rex(buf: &mut CodeBuffer, w: bool, reg: u8, rm: u8) {
    let mut rex: u8 = 0x40;
    if w {
        rex |= 0x08; // REX.W
    }
    if reg >= 8 {
        rex |= 0x04;
    }
    if rm >= 8 {
        rex |= 0x01;
    }
    if rex != 0x40 {
        buf.emit_byte(rex);
    }
}

/// ModR/M byte: mod=11 (register-register), reg, rm.
pub fn modrm_rr(reg: u8, rm: u8) -> u8 {
    0xC0 | ((reg & 7) << 3) | (rm & 7)
}

/// ModR/M byte with disp8: mod=01 (r/m + disp8), reg, rm.
pub fn modrm_disp8(reg: u8, rm: u8, _disp: i8) -> u8 {
    0x40 | ((reg & 7) << 3) | (rm & 7)
}

/// Emit `mov reg64, imm64` (REX.W + B8+reg + imm64).
pub fn emit_mov_reg_imm64(buf: &mut CodeBuffer, reg: u8, imm: u64) {
    emit_rex_w(buf, 0, reg);
    buf.emit_byte(0xB8 + (reg & 7));
    buf.emit64(imm);
}

/// Emit `mov reg64, [rbp + offset]` (REX.W 8B /r disp8 or disp32).
pub fn emit_mov_reg_rbp_offset(buf: &mut CodeBuffer, reg: u8, offset: i32) {
    if (-128..=127).contains(&offset) {
        emit_rex_w(buf, 0, reg);
        buf.emit_byte(0x8B);
        buf.emit_byte(modrm_disp8(RBP, reg, offset as i8));
        buf.emit_byte(offset as u8);
    } else {
        emit_rex_w(buf, 0, reg);
        buf.emit_byte(0x8B);
        // mod=10 (disp32), reg, rbp
        buf.emit_byte(0x80 | ((reg & 7) << 3) | RBP);
        buf.emit32(offset as u32);
    }
}

/// Emit `mov [rbp + offset], reg64`.
pub fn emit_mov_rbp_offset_reg(buf: &mut CodeBuffer, offset: i32, reg: u8) {
    if (-128..=127).contains(&offset) {
        emit_rex_w(buf, 0, reg);
        buf.emit_byte(0x89);
        buf.emit_byte(modrm_disp8(RBP, reg, offset as i8));
        buf.emit_byte(offset as u8);
    } else {
        emit_rex_w(buf, 0, reg);
        buf.emit_byte(0x89);
        buf.emit_byte(0x80 | ((reg & 7) << 3) | RBP);
        buf.emit32(offset as u32);
    }
}

/// Emit `mov reg64, [reg64 + offset]` where offset is disp32.
pub fn emit_mov_reg_mem_reg_disp32(buf: &mut CodeBuffer, dst: u8, base: u8, disp: i32) {
    emit_rex_w(buf, 0, dst);
    buf.emit_byte(0x8B);
    // mod=10 (disp32), reg=dst, r/m=base
    buf.emit_byte(0x80 | ((dst & 7) << 3) | (base & 7));
    buf.emit32(disp as u32);
}

/// Emit `mov [reg64 + offset], reg64` (store).
pub fn emit_mov_mem_reg_disp32_reg(buf: &mut CodeBuffer, base: u8, disp: i32, src: u8) {
    emit_rex_w(buf, 0, src);
    buf.emit_byte(0x89);
    // mod=10 (disp32), reg=src, r/m=base
    buf.emit_byte(0x80 | ((src & 7) << 3) | (base & 7));
    buf.emit32(disp as u32);
}

/// Emit `push reg64`.
pub fn emit_push_reg(buf: &mut CodeBuffer, reg: u8) {
    if reg >= 8 {
        buf.emit_byte(0x41); // REX.B
    }
    buf.emit_byte(0x50 + (reg & 7));
}

/// Emit `pop reg64`.
pub fn emit_pop_reg(buf: &mut CodeBuffer, reg: u8) {
    if reg >= 8 {
        buf.emit_byte(0x41);
    }
    buf.emit_byte(0x58 + (reg & 7));
}

/// Emit `ret`.
pub fn emit_ret(buf: &mut CodeBuffer) {
    buf.emit_byte(0xC3);
}

/// Emit `add reg64, imm32` (sign-extended). Uses `REX.W 81 /0 id`.
pub fn emit_add_reg_imm32(buf: &mut CodeBuffer, reg: u8, imm: i32) {
    if imm == 1 {
        // `inc reg64` — use REX.W FF /0
        emit_rex_w(buf, 0, reg);
        buf.emit_byte(0xFF);
        buf.emit_byte(0xC0 | (reg & 7));
        return;
    }
    emit_rex_w(buf, 0, reg);
    buf.emit_byte(0x81);
    buf.emit_byte(0xC0 | (reg & 7));
    buf.emit32(imm as u32);
}

/// Emit `sub reg64, imm32`.
pub fn emit_sub_reg_imm32(buf: &mut CodeBuffer, reg: u8, imm: i32) {
    emit_rex_w(buf, 0, reg);
    buf.emit_byte(0x81);
    buf.emit_byte(0xE8 | (reg & 7));
    buf.emit32(imm as u32);
}

/// Emit `cmp reg64, reg64`.
pub fn emit_cmp_reg_reg(buf: &mut CodeBuffer, left: u8, right: u8) {
    emit_rex_w(buf, 0, right);
    buf.emit_byte(0x39); // CMP r/m64, r64
    buf.emit_byte(modrm_rr(right, left));
}

/// Emit `cmp reg64, imm32` (sign-extended).
pub fn emit_cmp_reg_imm32(buf: &mut CodeBuffer, reg: u8, imm: i32) {
    emit_rex_w(buf, 0, reg);
    buf.emit_byte(0x81);
    buf.emit_byte(0xF8 | (reg & 7));
    buf.emit32(imm as u32);
}

/// Emit `test reg64, reg64`.
pub fn emit_test_reg_reg(buf: &mut CodeBuffer, left: u8, right: u8) {
    emit_rex_w(buf, 0, right);
    buf.emit_byte(0x85);
    buf.emit_byte(modrm_rr(right, left));
}

/// Emit `jmp rel32` and return the offset to patch later.
pub fn emit_jmp_rel32(buf: &mut CodeBuffer) -> usize {
    buf.emit_byte(0xE9);
    let off = buf.offset();
    buf.emit32(0); // placeholder
    off
}

/// Patch a `jmp rel32` at `patch_offset` to jump to `target_offset`.
pub fn patch_jmp(buf: &mut CodeBuffer, patch_offset: usize, target_offset: usize) {
    let disp = (target_offset as i64 - (patch_offset + 4) as i64) as i32;
    buf.patch32(patch_offset, disp as u32);
}

/// Conditional jump: emit `jcc rel32` and return patch offset.
/// `cc` is the condition code (0x8=O, 0x4=E/Z, 0x5=NE/NZ, 0xC=G, 0xD=LE, etc.)
pub fn emit_jcc_rel32(buf: &mut CodeBuffer, cc: u8) -> usize {
    buf.emit_byte(0x0F);
    buf.emit_byte(0x80 + cc);
    let off = buf.offset();
    buf.emit32(0); // placeholder
    off
}

/// Emit `mov reg64, reg64`.
pub fn emit_mov_reg_reg(buf: &mut CodeBuffer, dst: u8, src: u8) {
    if dst == src {
        return; // nop
    }
    emit_rex_w(buf, 0, src);
    buf.emit_byte(0x89);
    buf.emit_byte(modrm_rr(src, dst));
}

/// Emit `lea reg64, [reg64 + offset]` (disp32).
pub fn emit_lea_reg_reg_disp32(buf: &mut CodeBuffer, dst: u8, base: u8, disp: i32) {
    emit_rex_w(buf, 0, dst);
    buf.emit_byte(0x8D);
    buf.emit_byte(0x80 | ((dst & 7) << 3) | (base & 7));
    buf.emit32(disp as u32);
}

/// Emit `xor reg64, reg64` (zero register).
pub fn emit_xor_reg_reg(buf: &mut CodeBuffer, reg: u8) {
    // In 64-bit mode, `xor r32, r32` implicitly zero-extends to 64 bits.
    buf.emit_byte(0x31);
    buf.emit_byte(modrm_rr(reg, reg));
}

/// Emit `nop` (single byte).
pub fn emit_nop(buf: &mut CodeBuffer) {
    buf.emit_byte(0x90);
}
