//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1676-C1700: Instruction Decoders & CPU Emulation -- the kind of C code found
//! in CPU emulators, ISA simulators, binary translators, and disassemblers where
//! bit-level instruction decoding and register-level emulation are essential.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world instruction decoder and CPU emulation patterns
//! commonly found in QEMU, Unicorn Engine, spike (RISC-V ISS), capstone, and
//! cycle-accurate CPU simulators -- all expressed as valid C99.
//!
//! Organization:
//! - C1676-C1680: RISC-V decoder (R-type, I-type, S-type, B-type, U-type)
//! - C1681-C1685: ARM decoder (data processing, load/store, branch, multiply, coprocessor)
//! - C1686-C1690: x86 decoder (prefix parsing, ModR/M, SIB byte, displacement, immediate)
//! - C1691-C1695: MIPS decoder (R-format, I-format, J-format, FPU, special)
//! - C1696-C1700: CPU emulation (register file, ALU ops, condition flags, pipeline, instruction fetch)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C1676-C1680: RISC-V Decoder
// ============================================================================

#[test]
fn c1676_riscv_rtype_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t opcode;
    uint8_t rd;
    uint8_t funct3;
    uint8_t rs1;
    uint8_t rs2;
    uint8_t funct7;
} dec_riscv_rtype_t;

dec_riscv_rtype_t dec_riscv_decode_rtype(uint32_t instr) {
    dec_riscv_rtype_t r;
    r.opcode = (uint8_t)(instr & 0x7F);
    r.rd     = (uint8_t)((instr >> 7) & 0x1F);
    r.funct3 = (uint8_t)((instr >> 12) & 0x07);
    r.rs1    = (uint8_t)((instr >> 15) & 0x1F);
    r.rs2    = (uint8_t)((instr >> 20) & 0x1F);
    r.funct7 = (uint8_t)((instr >> 25) & 0x7F);
    return r;
}

int dec_riscv_rtype_is_add(dec_riscv_rtype_t r) {
    return r.opcode == 0x33 && r.funct3 == 0x00 && r.funct7 == 0x00;
}

int dec_riscv_rtype_is_sub(dec_riscv_rtype_t r) {
    return r.opcode == 0x33 && r.funct3 == 0x00 && r.funct7 == 0x20;
}

int dec_riscv_rtype_is_sll(dec_riscv_rtype_t r) {
    return r.opcode == 0x33 && r.funct3 == 0x01 && r.funct7 == 0x00;
}

int dec_riscv_rtype_is_xor(dec_riscv_rtype_t r) {
    return r.opcode == 0x33 && r.funct3 == 0x04 && r.funct7 == 0x00;
}

int dec_riscv_rtype_is_or(dec_riscv_rtype_t r) {
    return r.opcode == 0x33 && r.funct3 == 0x06 && r.funct7 == 0x00;
}

int dec_riscv_rtype_is_and(dec_riscv_rtype_t r) {
    return r.opcode == 0x33 && r.funct3 == 0x07 && r.funct7 == 0x00;
}

int dec_riscv_rtype_selftest(void) {
    uint32_t add_instr = 0x003100B3;
    dec_riscv_rtype_t decoded = dec_riscv_decode_rtype(add_instr);
    if (decoded.opcode != 0x33) return -1;
    if (!dec_riscv_rtype_is_add(decoded)) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1676: RISC-V R-type decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1676: Output should not be empty");
    assert!(
        code.contains("fn dec_riscv_decode_rtype"),
        "C1676: Should contain dec_riscv_decode_rtype function"
    );
    assert!(
        code.contains("fn dec_riscv_rtype_is_add"),
        "C1676: Should contain dec_riscv_rtype_is_add function"
    );
}

#[test]
fn c1677_riscv_itype_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef int int32_t;

typedef struct {
    uint8_t opcode;
    uint8_t rd;
    uint8_t funct3;
    uint8_t rs1;
    int32_t imm;
} dec_rv_itype_t;

dec_rv_itype_t dec_rv_decode_itype(uint32_t instr) {
    dec_rv_itype_t r;
    r.opcode = (uint8_t)(instr & 0x7F);
    r.rd     = (uint8_t)((instr >> 7) & 0x1F);
    r.funct3 = (uint8_t)((instr >> 12) & 0x07);
    r.rs1    = (uint8_t)((instr >> 15) & 0x1F);
    r.imm    = (int32_t)(instr >> 20);
    if (r.imm & 0x800) {
        r.imm |= 0xFFFFF000;
    }
    return r;
}

int dec_rv_itype_is_addi(dec_rv_itype_t r) {
    return r.opcode == 0x13 && r.funct3 == 0x00;
}

int dec_rv_itype_is_slti(dec_rv_itype_t r) {
    return r.opcode == 0x13 && r.funct3 == 0x02;
}

int dec_rv_itype_is_xori(dec_rv_itype_t r) {
    return r.opcode == 0x13 && r.funct3 == 0x04;
}

int dec_rv_itype_is_ori(dec_rv_itype_t r) {
    return r.opcode == 0x13 && r.funct3 == 0x06;
}

int dec_rv_itype_is_andi(dec_rv_itype_t r) {
    return r.opcode == 0x13 && r.funct3 == 0x07;
}

int dec_rv_itype_is_load(dec_rv_itype_t r) {
    return r.opcode == 0x03;
}

int dec_rv_itype_selftest(void) {
    uint32_t addi_instr = 0x00A00093;
    dec_rv_itype_t decoded = dec_rv_decode_itype(addi_instr);
    if (decoded.opcode != 0x13) return -1;
    if (!dec_rv_itype_is_addi(decoded)) return -2;
    if (decoded.imm != 10) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1677: RISC-V I-type decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1677: Output should not be empty");
    assert!(
        code.contains("fn dec_rv_decode_itype"),
        "C1677: Should contain dec_rv_decode_itype function"
    );
}

#[test]
fn c1678_riscv_stype_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef int int32_t;

typedef struct {
    uint8_t opcode;
    uint8_t funct3;
    uint8_t rs1;
    uint8_t rs2;
    int32_t imm;
} dec_rv_stype_t;

dec_rv_stype_t dec_rv_decode_stype(uint32_t instr) {
    dec_rv_stype_t r;
    r.opcode = (uint8_t)(instr & 0x7F);
    r.funct3 = (uint8_t)((instr >> 12) & 0x07);
    r.rs1    = (uint8_t)((instr >> 15) & 0x1F);
    r.rs2    = (uint8_t)((instr >> 20) & 0x1F);
    r.imm    = (int32_t)(((instr >> 7) & 0x1F) | (((instr >> 25) & 0x7F) << 5));
    if (r.imm & 0x800) {
        r.imm |= 0xFFFFF000;
    }
    return r;
}

int dec_rv_stype_is_sb(dec_rv_stype_t s) {
    return s.opcode == 0x23 && s.funct3 == 0x00;
}

int dec_rv_stype_is_sh(dec_rv_stype_t s) {
    return s.opcode == 0x23 && s.funct3 == 0x01;
}

int dec_rv_stype_is_sw(dec_rv_stype_t s) {
    return s.opcode == 0x23 && s.funct3 == 0x02;
}

int dec_rv_stype_get_offset(dec_rv_stype_t s) {
    return s.imm;
}

int dec_rv_stype_selftest(void) {
    uint32_t sw_instr = 0x00112023;
    dec_rv_stype_t decoded = dec_rv_decode_stype(sw_instr);
    if (decoded.opcode != 0x23) return -1;
    if (!dec_rv_stype_is_sw(decoded)) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1678: RISC-V S-type decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1678: Output should not be empty");
    assert!(
        code.contains("fn dec_rv_decode_stype"),
        "C1678: Should contain dec_rv_decode_stype function"
    );
}

#[test]
fn c1679_riscv_btype_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef int int32_t;

typedef struct {
    uint8_t opcode;
    uint8_t funct3;
    uint8_t rs1;
    uint8_t rs2;
    int32_t imm;
} dec_rv_btype_t;

dec_rv_btype_t dec_rv_decode_btype(uint32_t instr) {
    dec_rv_btype_t r;
    r.opcode = (uint8_t)(instr & 0x7F);
    r.funct3 = (uint8_t)((instr >> 12) & 0x07);
    r.rs1    = (uint8_t)((instr >> 15) & 0x1F);
    r.rs2    = (uint8_t)((instr >> 20) & 0x1F);
    r.imm = (int32_t)(
        (((instr >> 8) & 0x0F) << 1) |
        (((instr >> 25) & 0x3F) << 5) |
        (((instr >> 7) & 0x01) << 11) |
        (((instr >> 31) & 0x01) << 12)
    );
    if (r.imm & 0x1000) {
        r.imm |= 0xFFFFE000;
    }
    return r;
}

int dec_rv_btype_is_beq(dec_rv_btype_t b) {
    return b.opcode == 0x63 && b.funct3 == 0x00;
}

int dec_rv_btype_is_bne(dec_rv_btype_t b) {
    return b.opcode == 0x63 && b.funct3 == 0x01;
}

int dec_rv_btype_is_blt(dec_rv_btype_t b) {
    return b.opcode == 0x63 && b.funct3 == 0x04;
}

int dec_rv_btype_is_bge(dec_rv_btype_t b) {
    return b.opcode == 0x63 && b.funct3 == 0x05;
}

int dec_rv_btype_get_target(dec_rv_btype_t b, uint32_t pc) {
    return (int32_t)pc + b.imm;
}

int dec_rv_btype_selftest(void) {
    uint32_t beq_instr = 0x00208463;
    dec_rv_btype_t decoded = dec_rv_decode_btype(beq_instr);
    if (decoded.opcode != 0x63) return -1;
    if (!dec_rv_btype_is_beq(decoded)) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1679: RISC-V B-type decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1679: Output should not be empty");
    assert!(
        code.contains("fn dec_rv_decode_btype"),
        "C1679: Should contain dec_rv_decode_btype function"
    );
}

#[test]
fn c1680_riscv_utype_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef int int32_t;

typedef struct {
    uint8_t opcode;
    uint8_t rd;
    int32_t imm;
} dec_rv_utype_t;

dec_rv_utype_t dec_rv_decode_utype(uint32_t instr) {
    dec_rv_utype_t r;
    r.opcode = (uint8_t)(instr & 0x7F);
    r.rd     = (uint8_t)((instr >> 7) & 0x1F);
    r.imm    = (int32_t)(instr & 0xFFFFF000);
    return r;
}

int dec_rv_utype_is_lui(dec_rv_utype_t u) {
    return u.opcode == 0x37;
}

int dec_rv_utype_is_auipc(dec_rv_utype_t u) {
    return u.opcode == 0x17;
}

uint32_t dec_rv_utype_get_upper(dec_rv_utype_t u) {
    return (uint32_t)u.imm;
}

uint32_t dec_rv_utype_compose_address(dec_rv_utype_t u, int32_t lower12) {
    return (uint32_t)u.imm + (uint32_t)lower12;
}

int dec_rv_utype_selftest(void) {
    uint32_t lui_instr = 0x12345037;
    dec_rv_utype_t decoded = dec_rv_decode_utype(lui_instr);
    if (decoded.opcode != 0x37) return -1;
    if (!dec_rv_utype_is_lui(decoded)) return -2;
    if (decoded.rd != 0) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1680: RISC-V U-type decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1680: Output should not be empty");
    assert!(
        code.contains("fn dec_rv_decode_utype"),
        "C1680: Should contain dec_rv_decode_utype function"
    );
}

// ============================================================================
// C1681-C1685: ARM Decoder
// ============================================================================

#[test]
fn c1681_arm_data_processing_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t cond;
    uint8_t opcode;
    uint8_t s_bit;
    uint8_t rn;
    uint8_t rd;
    uint8_t shift_amount;
    uint8_t shift_type;
    uint8_t rm;
    uint32_t imm;
    int is_immediate;
} dec_arm_dp_t;

dec_arm_dp_t dec_arm_decode_dp(uint32_t instr) {
    dec_arm_dp_t d;
    d.cond = (uint8_t)((instr >> 28) & 0x0F);
    d.is_immediate = (instr >> 25) & 0x01;
    d.opcode = (uint8_t)((instr >> 21) & 0x0F);
    d.s_bit = (uint8_t)((instr >> 20) & 0x01);
    d.rn = (uint8_t)((instr >> 16) & 0x0F);
    d.rd = (uint8_t)((instr >> 12) & 0x0F);
    if (d.is_immediate) {
        uint8_t rotate = (uint8_t)((instr >> 8) & 0x0F);
        uint32_t val = instr & 0xFF;
        d.imm = (val >> (rotate * 2)) | (val << (32 - rotate * 2));
        d.shift_amount = 0;
        d.shift_type = 0;
        d.rm = 0;
    } else {
        d.imm = 0;
        d.shift_amount = (uint8_t)((instr >> 7) & 0x1F);
        d.shift_type = (uint8_t)((instr >> 5) & 0x03);
        d.rm = (uint8_t)(instr & 0x0F);
    }
    return d;
}

int dec_arm_dp_is_and(dec_arm_dp_t d) { return d.opcode == 0x00; }
int dec_arm_dp_is_eor(dec_arm_dp_t d) { return d.opcode == 0x01; }
int dec_arm_dp_is_sub(dec_arm_dp_t d) { return d.opcode == 0x02; }
int dec_arm_dp_is_add(dec_arm_dp_t d) { return d.opcode == 0x04; }
int dec_arm_dp_is_mov(dec_arm_dp_t d) { return d.opcode == 0x0D; }
int dec_arm_dp_is_cmp(dec_arm_dp_t d) { return d.opcode == 0x0A; }

int dec_arm_dp_selftest(void) {
    uint32_t mov_instr = 0xE3A01005;
    dec_arm_dp_t decoded = dec_arm_decode_dp(mov_instr);
    if (decoded.cond != 0x0E) return -1;
    if (!dec_arm_dp_is_mov(decoded)) return -2;
    if (decoded.rd != 1) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1681: ARM data processing decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1681: Output should not be empty");
    assert!(
        code.contains("fn dec_arm_decode_dp"),
        "C1681: Should contain dec_arm_decode_dp function"
    );
}

#[test]
fn c1682_arm_load_store_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef int int32_t;

typedef struct {
    uint8_t cond;
    uint8_t is_load;
    uint8_t is_byte;
    uint8_t is_pre_index;
    uint8_t is_add;
    uint8_t is_writeback;
    uint8_t rn;
    uint8_t rd;
    int32_t offset;
    int is_register_offset;
    uint8_t rm;
} dec_arm_ls_t;

dec_arm_ls_t dec_arm_decode_ls(uint32_t instr) {
    dec_arm_ls_t ls;
    ls.cond = (uint8_t)((instr >> 28) & 0x0F);
    ls.is_register_offset = (instr >> 25) & 0x01;
    ls.is_pre_index = (uint8_t)((instr >> 24) & 0x01);
    ls.is_add = (uint8_t)((instr >> 23) & 0x01);
    ls.is_byte = (uint8_t)((instr >> 22) & 0x01);
    ls.is_writeback = (uint8_t)((instr >> 21) & 0x01);
    ls.is_load = (uint8_t)((instr >> 20) & 0x01);
    ls.rn = (uint8_t)((instr >> 16) & 0x0F);
    ls.rd = (uint8_t)((instr >> 12) & 0x0F);
    if (ls.is_register_offset) {
        ls.rm = (uint8_t)(instr & 0x0F);
        ls.offset = 0;
    } else {
        ls.offset = (int32_t)(instr & 0xFFF);
        ls.rm = 0;
    }
    return ls;
}

uint32_t dec_arm_ls_effective_addr(dec_arm_ls_t ls, uint32_t base_val) {
    if (ls.is_add) {
        return base_val + (uint32_t)ls.offset;
    } else {
        return base_val - (uint32_t)ls.offset;
    }
}

int dec_arm_ls_is_ldr(dec_arm_ls_t ls) {
    return ls.is_load && !ls.is_byte;
}

int dec_arm_ls_is_str(dec_arm_ls_t ls) {
    return !ls.is_load && !ls.is_byte;
}

int dec_arm_ls_is_ldrb(dec_arm_ls_t ls) {
    return ls.is_load && ls.is_byte;
}

int dec_arm_ls_selftest(void) {
    uint32_t ldr_instr = 0xE5910004;
    dec_arm_ls_t decoded = dec_arm_decode_ls(ldr_instr);
    if (!dec_arm_ls_is_ldr(decoded)) return -1;
    if (decoded.rn != 1) return -2;
    if (decoded.offset != 4) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1682: ARM load/store decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1682: Output should not be empty");
    assert!(
        code.contains("fn dec_arm_decode_ls"),
        "C1682: Should contain dec_arm_decode_ls function"
    );
}

#[test]
fn c1683_arm_branch_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef int int32_t;

typedef struct {
    uint8_t cond;
    int is_link;
    int32_t offset;
} dec_arm_branch_t;

dec_arm_branch_t dec_arm_decode_branch(uint32_t instr) {
    dec_arm_branch_t b;
    b.cond = (uint8_t)((instr >> 28) & 0x0F);
    b.is_link = (instr >> 24) & 0x01;
    b.offset = (int32_t)(instr & 0x00FFFFFF);
    if (b.offset & 0x00800000) {
        b.offset |= 0xFF000000;
    }
    b.offset = b.offset << 2;
    return b;
}

uint32_t dec_arm_branch_target(dec_arm_branch_t b, uint32_t pc) {
    return (uint32_t)((int32_t)pc + 8 + b.offset);
}

int dec_arm_branch_is_unconditional(dec_arm_branch_t b) {
    return b.cond == 0x0E;
}

int dec_arm_branch_is_conditional(dec_arm_branch_t b) {
    return b.cond != 0x0E;
}

int dec_arm_branch_is_bl(dec_arm_branch_t b) {
    return b.is_link;
}

const char *dec_arm_branch_cond_name(uint8_t cond) {
    if (cond == 0x00) return "EQ";
    if (cond == 0x01) return "NE";
    if (cond == 0x02) return "CS";
    if (cond == 0x03) return "CC";
    if (cond == 0x0A) return "GE";
    if (cond == 0x0B) return "LT";
    if (cond == 0x0C) return "GT";
    if (cond == 0x0D) return "LE";
    if (cond == 0x0E) return "AL";
    return "??";
}

int dec_arm_branch_selftest(void) {
    uint32_t b_instr = 0xEA000002;
    dec_arm_branch_t decoded = dec_arm_decode_branch(b_instr);
    if (!dec_arm_branch_is_unconditional(decoded)) return -1;
    if (dec_arm_branch_is_bl(decoded)) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1683: ARM branch decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1683: Output should not be empty");
    assert!(
        code.contains("fn dec_arm_decode_branch"),
        "C1683: Should contain dec_arm_decode_branch function"
    );
}

#[test]
fn c1684_arm_multiply_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef unsigned long long uint64_t;

typedef struct {
    uint8_t cond;
    uint8_t opcode;
    uint8_t s_bit;
    uint8_t rd;
    uint8_t rn;
    uint8_t rs;
    uint8_t rm;
    int is_accumulate;
    int is_long;
} dec_arm_mul_t;

dec_arm_mul_t dec_arm_decode_mul(uint32_t instr) {
    dec_arm_mul_t m;
    m.cond = (uint8_t)((instr >> 28) & 0x0F);
    m.is_accumulate = (instr >> 21) & 0x01;
    m.s_bit = (uint8_t)((instr >> 20) & 0x01);
    m.rd = (uint8_t)((instr >> 16) & 0x0F);
    m.rn = (uint8_t)((instr >> 12) & 0x0F);
    m.rs = (uint8_t)((instr >> 8) & 0x0F);
    m.rm = (uint8_t)(instr & 0x0F);
    m.is_long = (instr >> 23) & 0x01;
    m.opcode = (uint8_t)((instr >> 21) & 0x0F);
    return m;
}

int dec_arm_mul_is_mul(dec_arm_mul_t m) {
    return !m.is_accumulate && !m.is_long;
}

int dec_arm_mul_is_mla(dec_arm_mul_t m) {
    return m.is_accumulate && !m.is_long;
}

uint32_t dec_arm_mul_execute(uint32_t a, uint32_t b) {
    return a * b;
}

uint32_t dec_arm_mla_execute(uint32_t a, uint32_t b, uint32_t acc) {
    return a * b + acc;
}

uint64_t dec_arm_umull_execute(uint32_t a, uint32_t b) {
    return (uint64_t)a * (uint64_t)b;
}

int dec_arm_mul_selftest(void) {
    uint32_t mul_instr = 0xE0010092;
    dec_arm_mul_t decoded = dec_arm_decode_mul(mul_instr);
    if (decoded.cond != 0x0E) return -1;
    if (decoded.rm != 2) return -2;
    if (dec_arm_mul_execute(3, 7) != 21) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1684: ARM multiply decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1684: Output should not be empty");
    assert!(
        code.contains("fn dec_arm_decode_mul"),
        "C1684: Should contain dec_arm_decode_mul function"
    );
}

#[test]
fn c1685_arm_coprocessor_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t cond;
    uint8_t cp_num;
    uint8_t opcode1;
    uint8_t crn;
    uint8_t crd;
    uint8_t opcode2;
    uint8_t crm;
    int is_transfer;
} dec_arm_coproc_t;

dec_arm_coproc_t dec_arm_decode_coproc(uint32_t instr) {
    dec_arm_coproc_t c;
    c.cond = (uint8_t)((instr >> 28) & 0x0F);
    c.opcode1 = (uint8_t)((instr >> 21) & 0x07);
    c.crn = (uint8_t)((instr >> 16) & 0x0F);
    c.crd = (uint8_t)((instr >> 12) & 0x0F);
    c.cp_num = (uint8_t)((instr >> 8) & 0x0F);
    c.opcode2 = (uint8_t)((instr >> 5) & 0x07);
    c.crm = (uint8_t)(instr & 0x0F);
    c.is_transfer = (instr >> 4) & 0x01;
    return c;
}

int dec_arm_coproc_is_cdp(dec_arm_coproc_t c) {
    return !c.is_transfer;
}

int dec_arm_coproc_is_mcr(dec_arm_coproc_t c) {
    return c.is_transfer && ((c.opcode1 & 0x01) == 0);
}

int dec_arm_coproc_is_mrc(dec_arm_coproc_t c) {
    return c.is_transfer && ((c.opcode1 & 0x01) == 1);
}

int dec_arm_coproc_is_cp15(dec_arm_coproc_t c) {
    return c.cp_num == 15;
}

int dec_arm_coproc_is_vfp(dec_arm_coproc_t c) {
    return c.cp_num == 10 || c.cp_num == 11;
}

int dec_arm_coproc_selftest(void) {
    uint32_t mcr_instr = 0xEE010F10;
    dec_arm_coproc_t decoded = dec_arm_decode_coproc(mcr_instr);
    if (decoded.cond != 0x0E) return -1;
    if (!dec_arm_coproc_is_cp15(decoded)) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1685: ARM coprocessor decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1685: Output should not be empty");
    assert!(
        code.contains("fn dec_arm_decode_coproc"),
        "C1685: Should contain dec_arm_decode_coproc function"
    );
}

// ============================================================================
// C1686-C1690: x86 Decoder
// ============================================================================

#[test]
fn c1686_x86_prefix_parsing() {
    let c_code = r##"
typedef unsigned char uint8_t;

typedef struct {
    int has_lock;
    int has_rep;
    int has_repne;
    int has_operand_override;
    int has_address_override;
    int segment_override;
    int prefix_count;
} dec_x86_prefix_t;

void dec_x86_prefix_init(dec_x86_prefix_t *p) {
    p->has_lock = 0;
    p->has_rep = 0;
    p->has_repne = 0;
    p->has_operand_override = 0;
    p->has_address_override = 0;
    p->segment_override = -1;
    p->prefix_count = 0;
}

int dec_x86_parse_prefixes(const uint8_t *bytes, int len, dec_x86_prefix_t *out) {
    dec_x86_prefix_init(out);
    int i = 0;
    while (i < len && i < 4) {
        uint8_t b = bytes[i];
        if (b == 0xF0) {
            out->has_lock = 1;
        } else if (b == 0xF3) {
            out->has_rep = 1;
        } else if (b == 0xF2) {
            out->has_repne = 1;
        } else if (b == 0x66) {
            out->has_operand_override = 1;
        } else if (b == 0x67) {
            out->has_address_override = 1;
        } else if (b == 0x2E) {
            out->segment_override = 1;
        } else if (b == 0x36) {
            out->segment_override = 2;
        } else if (b == 0x3E) {
            out->segment_override = 3;
        } else if (b == 0x26) {
            out->segment_override = 0;
        } else if (b == 0x64) {
            out->segment_override = 4;
        } else if (b == 0x65) {
            out->segment_override = 5;
        } else {
            break;
        }
        out->prefix_count++;
        i++;
    }
    return i;
}

int dec_x86_prefix_selftest(void) {
    uint8_t bytes[4];
    bytes[0] = 0xF0;
    bytes[1] = 0x66;
    bytes[2] = 0x89;
    bytes[3] = 0xC0;
    dec_x86_prefix_t pfx;
    int consumed = dec_x86_parse_prefixes(bytes, 4, &pfx);
    if (consumed != 2) return -1;
    if (!pfx.has_lock) return -2;
    if (!pfx.has_operand_override) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1686: x86 prefix parsing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1686: Output should not be empty");
    assert!(
        code.contains("fn dec_x86_parse_prefixes"),
        "C1686: Should contain dec_x86_parse_prefixes function"
    );
}

#[test]
fn c1687_x86_modrm_decode() {
    let c_code = r##"
typedef unsigned char uint8_t;

typedef struct {
    uint8_t mod_field;
    uint8_t reg;
    uint8_t rm;
    int has_sib;
    int has_disp8;
    int has_disp32;
} dec_x86_modrm_t;

dec_x86_modrm_t dec_x86_decode_modrm(uint8_t byte) {
    dec_x86_modrm_t m;
    m.mod_field = (byte >> 6) & 0x03;
    m.reg = (byte >> 3) & 0x07;
    m.rm = byte & 0x07;
    m.has_sib = 0;
    m.has_disp8 = 0;
    m.has_disp32 = 0;
    if (m.mod_field == 0x00) {
        if (m.rm == 0x04) m.has_sib = 1;
        if (m.rm == 0x05) m.has_disp32 = 1;
    } else if (m.mod_field == 0x01) {
        m.has_disp8 = 1;
        if (m.rm == 0x04) m.has_sib = 1;
    } else if (m.mod_field == 0x02) {
        m.has_disp32 = 1;
        if (m.rm == 0x04) m.has_sib = 1;
    }
    return m;
}

int dec_x86_modrm_is_register(dec_x86_modrm_t m) {
    return m.mod_field == 0x03;
}

int dec_x86_modrm_is_memory(dec_x86_modrm_t m) {
    return m.mod_field != 0x03;
}

int dec_x86_modrm_needs_sib(dec_x86_modrm_t m) {
    return m.has_sib;
}

int dec_x86_modrm_disp_size(dec_x86_modrm_t m) {
    if (m.has_disp32) return 4;
    if (m.has_disp8) return 1;
    return 0;
}

int dec_x86_modrm_selftest(void) {
    dec_x86_modrm_t reg = dec_x86_decode_modrm(0xC0);
    if (!dec_x86_modrm_is_register(reg)) return -1;
    if (reg.reg != 0) return -2;
    if (reg.rm != 0) return -3;
    dec_x86_modrm_t mem = dec_x86_decode_modrm(0x44);
    if (!dec_x86_modrm_is_memory(mem)) return -4;
    if (!dec_x86_modrm_needs_sib(mem)) return -5;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1687: x86 ModR/M decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1687: Output should not be empty");
    assert!(
        code.contains("fn dec_x86_decode_modrm"),
        "C1687: Should contain dec_x86_decode_modrm function"
    );
}

#[test]
fn c1688_x86_sib_byte_decode() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t scale;
    uint8_t index;
    uint8_t base;
} dec_x86_sib_t;

dec_x86_sib_t dec_x86_decode_sib(uint8_t byte) {
    dec_x86_sib_t s;
    s.scale = (byte >> 6) & 0x03;
    s.index = (byte >> 3) & 0x07;
    s.base = byte & 0x07;
    return s;
}

uint32_t dec_x86_sib_scale_factor(dec_x86_sib_t s) {
    uint32_t factors[4];
    factors[0] = 1;
    factors[1] = 2;
    factors[2] = 4;
    factors[3] = 8;
    return factors[s.scale];
}

int dec_x86_sib_has_index(dec_x86_sib_t s) {
    return s.index != 0x04;
}

int dec_x86_sib_has_base(dec_x86_sib_t s, uint8_t mod_field) {
    if (s.base == 0x05 && mod_field == 0x00) return 0;
    return 1;
}

uint32_t dec_x86_sib_compute_addr(dec_x86_sib_t s, uint32_t base_val,
                                   uint32_t index_val, uint32_t disp) {
    uint32_t scaled = index_val * dec_x86_sib_scale_factor(s);
    return base_val + scaled + disp;
}

int dec_x86_sib_selftest(void) {
    dec_x86_sib_t sib = dec_x86_decode_sib(0x8D);
    if (sib.scale != 2) return -1;
    if (dec_x86_sib_scale_factor(sib) != 4) return -2;
    uint32_t addr = dec_x86_sib_compute_addr(sib, 100, 10, 8);
    if (addr != 148) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1688: x86 SIB byte decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1688: Output should not be empty");
    assert!(
        code.contains("fn dec_x86_decode_sib"),
        "C1688: Should contain dec_x86_decode_sib function"
    );
}

#[test]
fn c1689_x86_displacement_decode() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef int int32_t;
typedef unsigned int uint32_t;
typedef short int16_t;

int32_t dec_x86_read_disp8(const uint8_t *bytes, int offset) {
    int32_t val = (int32_t)bytes[offset];
    if (val & 0x80) {
        val |= 0xFFFFFF00;
    }
    return val;
}

int32_t dec_x86_read_disp16(const uint8_t *bytes, int offset) {
    int32_t val = (int32_t)(bytes[offset] | (bytes[offset + 1] << 8));
    if (val & 0x8000) {
        val |= 0xFFFF0000;
    }
    return val;
}

int32_t dec_x86_read_disp32(const uint8_t *bytes, int offset) {
    int32_t val = (int32_t)(
        bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24)
    );
    return val;
}

uint32_t dec_x86_apply_disp(uint32_t base, int32_t disp) {
    return (uint32_t)((int32_t)base + disp);
}

int dec_x86_disp_is_positive(int32_t disp) {
    return disp >= 0;
}

int dec_x86_disp_magnitude(int32_t disp) {
    if (disp < 0) return -disp;
    return disp;
}

int dec_x86_disp_selftest(void) {
    uint8_t data[4];
    data[0] = 0xFC;
    int32_t d8 = dec_x86_read_disp8(data, 0);
    if (d8 != -4) return -1;
    data[0] = 0x04;
    data[1] = 0x00;
    int32_t d16 = dec_x86_read_disp16(data, 0);
    if (d16 != 4) return -2;
    data[0] = 0xFF;
    data[1] = 0xFF;
    data[2] = 0xFF;
    data[3] = 0xFF;
    int32_t d32 = dec_x86_read_disp32(data, 0);
    if (d32 != -1) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1689: x86 displacement decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1689: Output should not be empty");
    assert!(
        code.contains("fn dec_x86_read_disp8"),
        "C1689: Should contain dec_x86_read_disp8 function"
    );
}

#[test]
fn c1690_x86_immediate_extraction() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef int int32_t;
typedef unsigned short uint16_t;

uint32_t dec_x86_read_imm8(const uint8_t *bytes, int offset) {
    return (uint32_t)bytes[offset];
}

uint32_t dec_x86_read_imm16(const uint8_t *bytes, int offset) {
    return (uint32_t)(bytes[offset] | (bytes[offset + 1] << 8));
}

uint32_t dec_x86_read_imm32(const uint8_t *bytes, int offset) {
    return (uint32_t)(
        bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24)
    );
}

int32_t dec_x86_sign_extend8(uint32_t val) {
    if (val & 0x80) {
        return (int32_t)(val | 0xFFFFFF00);
    }
    return (int32_t)val;
}

int32_t dec_x86_sign_extend16(uint32_t val) {
    if (val & 0x8000) {
        return (int32_t)(val | 0xFFFF0000);
    }
    return (int32_t)val;
}

int dec_x86_imm_fits_byte(int32_t val) {
    return val >= -128 && val <= 127;
}

int dec_x86_imm_fits_word(int32_t val) {
    return val >= -32768 && val <= 32767;
}

int dec_x86_imm_selftest(void) {
    uint8_t data[4];
    data[0] = 0x42;
    if (dec_x86_read_imm8(data, 0) != 0x42) return -1;
    int32_t se = dec_x86_sign_extend8(0xFF);
    if (se != -1) return -2;
    if (!dec_x86_imm_fits_byte(100)) return -3;
    if (dec_x86_imm_fits_byte(200)) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1690: x86 immediate extraction should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1690: Output should not be empty");
    assert!(
        code.contains("fn dec_x86_read_imm8"),
        "C1690: Should contain dec_x86_read_imm8 function"
    );
}

// ============================================================================
// C1691-C1695: MIPS Decoder
// ============================================================================

#[test]
fn c1691_mips_rformat_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t opcode;
    uint8_t rs;
    uint8_t rt;
    uint8_t rd;
    uint8_t shamt;
    uint8_t funct;
} dec_mips_rtype_t;

dec_mips_rtype_t dec_mips_decode_rtype(uint32_t instr) {
    dec_mips_rtype_t r;
    r.opcode = (uint8_t)((instr >> 26) & 0x3F);
    r.rs     = (uint8_t)((instr >> 21) & 0x1F);
    r.rt     = (uint8_t)((instr >> 16) & 0x1F);
    r.rd     = (uint8_t)((instr >> 11) & 0x1F);
    r.shamt  = (uint8_t)((instr >> 6) & 0x1F);
    r.funct  = (uint8_t)(instr & 0x3F);
    return r;
}

int dec_mips_rtype_is_add(dec_mips_rtype_t r) {
    return r.opcode == 0x00 && r.funct == 0x20;
}

int dec_mips_rtype_is_sub(dec_mips_rtype_t r) {
    return r.opcode == 0x00 && r.funct == 0x22;
}

int dec_mips_rtype_is_and(dec_mips_rtype_t r) {
    return r.opcode == 0x00 && r.funct == 0x24;
}

int dec_mips_rtype_is_or(dec_mips_rtype_t r) {
    return r.opcode == 0x00 && r.funct == 0x25;
}

int dec_mips_rtype_is_sll(dec_mips_rtype_t r) {
    return r.opcode == 0x00 && r.funct == 0x00;
}

int dec_mips_rtype_is_srl(dec_mips_rtype_t r) {
    return r.opcode == 0x00 && r.funct == 0x02;
}

int dec_mips_rtype_is_jr(dec_mips_rtype_t r) {
    return r.opcode == 0x00 && r.funct == 0x08;
}

int dec_mips_rtype_selftest(void) {
    uint32_t add_instr = 0x01294820;
    dec_mips_rtype_t decoded = dec_mips_decode_rtype(add_instr);
    if (decoded.opcode != 0x00) return -1;
    if (!dec_mips_rtype_is_add(decoded)) return -2;
    if (decoded.rd != 9) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1691: MIPS R-format decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1691: Output should not be empty");
    assert!(
        code.contains("fn dec_mips_decode_rtype"),
        "C1691: Should contain dec_mips_decode_rtype function"
    );
}

#[test]
fn c1692_mips_iformat_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef int int32_t;
typedef short int16_t;

typedef struct {
    uint8_t opcode;
    uint8_t rs;
    uint8_t rt;
    int32_t imm;
} dec_mips_itype_t;

dec_mips_itype_t dec_mips_decode_itype(uint32_t instr) {
    dec_mips_itype_t r;
    r.opcode = (uint8_t)((instr >> 26) & 0x3F);
    r.rs     = (uint8_t)((instr >> 21) & 0x1F);
    r.rt     = (uint8_t)((instr >> 16) & 0x1F);
    r.imm    = (int32_t)(int16_t)(instr & 0xFFFF);
    return r;
}

int dec_mips_itype_is_addi(dec_mips_itype_t i) {
    return i.opcode == 0x08;
}

int dec_mips_itype_is_beq(dec_mips_itype_t i) {
    return i.opcode == 0x04;
}

int dec_mips_itype_is_bne(dec_mips_itype_t i) {
    return i.opcode == 0x05;
}

int dec_mips_itype_is_lw(dec_mips_itype_t i) {
    return i.opcode == 0x23;
}

int dec_mips_itype_is_sw(dec_mips_itype_t i) {
    return i.opcode == 0x2B;
}

int dec_mips_itype_is_andi(dec_mips_itype_t i) {
    return i.opcode == 0x0C;
}

uint32_t dec_mips_itype_branch_target(dec_mips_itype_t i, uint32_t pc) {
    return pc + 4 + (uint32_t)(i.imm << 2);
}

int dec_mips_itype_selftest(void) {
    uint32_t addi_instr = 0x2129000A;
    dec_mips_itype_t decoded = dec_mips_decode_itype(addi_instr);
    if (!dec_mips_itype_is_addi(decoded)) return -1;
    if (decoded.imm != 10) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1692: MIPS I-format decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1692: Output should not be empty");
    assert!(
        code.contains("fn dec_mips_decode_itype"),
        "C1692: Should contain dec_mips_decode_itype function"
    );
}

#[test]
fn c1693_mips_jformat_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t opcode;
    uint32_t target;
} dec_mips_jtype_t;

dec_mips_jtype_t dec_mips_decode_jtype(uint32_t instr) {
    dec_mips_jtype_t j;
    j.opcode = (uint8_t)((instr >> 26) & 0x3F);
    j.target = instr & 0x03FFFFFF;
    return j;
}

int dec_mips_jtype_is_j(dec_mips_jtype_t j) {
    return j.opcode == 0x02;
}

int dec_mips_jtype_is_jal(dec_mips_jtype_t j) {
    return j.opcode == 0x03;
}

uint32_t dec_mips_jtype_full_target(dec_mips_jtype_t j, uint32_t pc) {
    return (pc & 0xF0000000) | (j.target << 2);
}

uint32_t dec_mips_jtype_return_addr(uint32_t pc) {
    return pc + 8;
}

int dec_mips_jtype_target_aligned(dec_mips_jtype_t j) {
    return 1;
}

int dec_mips_jtype_selftest(void) {
    uint32_t j_instr = 0x08000100;
    dec_mips_jtype_t decoded = dec_mips_decode_jtype(j_instr);
    if (!dec_mips_jtype_is_j(decoded)) return -1;
    uint32_t target = dec_mips_jtype_full_target(decoded, 0x00400000);
    if (target != 0x00000400) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1693: MIPS J-format decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1693: Output should not be empty");
    assert!(
        code.contains("fn dec_mips_decode_jtype"),
        "C1693: Should contain dec_mips_decode_jtype function"
    );
}

#[test]
fn c1694_mips_fpu_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t opcode;
    uint8_t fmt;
    uint8_t ft;
    uint8_t fs;
    uint8_t fd;
    uint8_t funct;
} dec_mips_fpu_t;

dec_mips_fpu_t dec_mips_decode_fpu(uint32_t instr) {
    dec_mips_fpu_t f;
    f.opcode = (uint8_t)((instr >> 26) & 0x3F);
    f.fmt    = (uint8_t)((instr >> 21) & 0x1F);
    f.ft     = (uint8_t)((instr >> 16) & 0x1F);
    f.fs     = (uint8_t)((instr >> 11) & 0x1F);
    f.fd     = (uint8_t)((instr >> 6) & 0x1F);
    f.funct  = (uint8_t)(instr & 0x3F);
    return f;
}

int dec_mips_fpu_is_cop1(dec_mips_fpu_t f) {
    return f.opcode == 0x11;
}

int dec_mips_fpu_is_single(dec_mips_fpu_t f) {
    return f.fmt == 0x10;
}

int dec_mips_fpu_is_double(dec_mips_fpu_t f) {
    return f.fmt == 0x11;
}

int dec_mips_fpu_is_add(dec_mips_fpu_t f) {
    return f.funct == 0x00;
}

int dec_mips_fpu_is_sub(dec_mips_fpu_t f) {
    return f.funct == 0x01;
}

int dec_mips_fpu_is_mul(dec_mips_fpu_t f) {
    return f.funct == 0x02;
}

int dec_mips_fpu_is_div(dec_mips_fpu_t f) {
    return f.funct == 0x03;
}

int dec_mips_fpu_selftest(void) {
    uint32_t fadd_instr = 0x46020800;
    dec_mips_fpu_t decoded = dec_mips_decode_fpu(fadd_instr);
    if (!dec_mips_fpu_is_cop1(decoded)) return -1;
    if (!dec_mips_fpu_is_single(decoded)) return -2;
    if (!dec_mips_fpu_is_add(decoded)) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1694: MIPS FPU decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1694: Output should not be empty");
    assert!(
        code.contains("fn dec_mips_decode_fpu"),
        "C1694: Should contain dec_mips_decode_fpu function"
    );
}

#[test]
fn c1695_mips_special_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t rs;
    uint8_t rt;
    uint8_t rd;
    uint8_t funct;
    int is_syscall;
    int is_break;
    uint32_t code;
} dec_mips_special_t;

dec_mips_special_t dec_mips_decode_special(uint32_t instr) {
    dec_mips_special_t s;
    s.rs = (uint8_t)((instr >> 21) & 0x1F);
    s.rt = (uint8_t)((instr >> 16) & 0x1F);
    s.rd = (uint8_t)((instr >> 11) & 0x1F);
    s.funct = (uint8_t)(instr & 0x3F);
    s.is_syscall = (s.funct == 0x0C);
    s.is_break = (s.funct == 0x0D);
    s.code = (instr >> 6) & 0xFFFFF;
    return s;
}

int dec_mips_special_is_mult(dec_mips_special_t s) {
    return s.funct == 0x18;
}

int dec_mips_special_is_multu(dec_mips_special_t s) {
    return s.funct == 0x19;
}

int dec_mips_special_is_div(dec_mips_special_t s) {
    return s.funct == 0x1A;
}

int dec_mips_special_is_mfhi(dec_mips_special_t s) {
    return s.funct == 0x10;
}

int dec_mips_special_is_mflo(dec_mips_special_t s) {
    return s.funct == 0x12;
}

int dec_mips_special_is_movn(dec_mips_special_t s) {
    return s.funct == 0x0B;
}

int dec_mips_special_is_movz(dec_mips_special_t s) {
    return s.funct == 0x0A;
}

int dec_mips_special_selftest(void) {
    uint32_t syscall_instr = 0x0000000C;
    dec_mips_special_t decoded = dec_mips_decode_special(syscall_instr);
    if (!decoded.is_syscall) return -1;
    if (decoded.is_break) return -2;
    uint32_t mult_instr = 0x01290018;
    decoded = dec_mips_decode_special(mult_instr);
    if (!dec_mips_special_is_mult(decoded)) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1695: MIPS special decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1695: Output should not be empty");
    assert!(
        code.contains("fn dec_mips_decode_special"),
        "C1695: Should contain dec_mips_decode_special function"
    );
}

// ============================================================================
// C1696-C1700: CPU Emulation
// ============================================================================

#[test]
fn c1696_cpu_register_file() {
    let c_code = r##"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t gpr[32];
    uint32_t pc;
    uint32_t hi;
    uint32_t lo;
} dec_cpu_regfile_t;

void dec_cpu_regfile_init(dec_cpu_regfile_t *rf) {
    int i;
    for (i = 0; i < 32; i++) {
        rf->gpr[i] = 0;
    }
    rf->pc = 0;
    rf->hi = 0;
    rf->lo = 0;
}

uint32_t dec_cpu_regfile_read(const dec_cpu_regfile_t *rf, int reg) {
    if (reg == 0) return 0;
    if (reg < 0 || reg >= 32) return 0;
    return rf->gpr[reg];
}

void dec_cpu_regfile_write(dec_cpu_regfile_t *rf, int reg, uint32_t val) {
    if (reg == 0) return;
    if (reg < 0 || reg >= 32) return;
    rf->gpr[reg] = val;
}

void dec_cpu_regfile_set_pc(dec_cpu_regfile_t *rf, uint32_t addr) {
    rf->pc = addr;
}

uint32_t dec_cpu_regfile_get_pc(const dec_cpu_regfile_t *rf) {
    return rf->pc;
}

void dec_cpu_regfile_advance_pc(dec_cpu_regfile_t *rf, int offset) {
    rf->pc = rf->pc + (uint32_t)offset;
}

int dec_cpu_regfile_selftest(void) {
    dec_cpu_regfile_t rf;
    dec_cpu_regfile_init(&rf);
    if (dec_cpu_regfile_read(&rf, 0) != 0) return -1;
    dec_cpu_regfile_write(&rf, 1, 42);
    if (dec_cpu_regfile_read(&rf, 1) != 42) return -2;
    dec_cpu_regfile_write(&rf, 0, 99);
    if (dec_cpu_regfile_read(&rf, 0) != 0) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1696: CPU register file should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1696: Output should not be empty");
    assert!(
        code.contains("fn dec_cpu_regfile_init"),
        "C1696: Should contain dec_cpu_regfile_init function"
    );
    assert!(
        code.contains("fn dec_cpu_regfile_read"),
        "C1696: Should contain dec_cpu_regfile_read function"
    );
}

#[test]
fn c1697_cpu_alu_operations() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef int int32_t;

typedef struct {
    uint32_t result;
    int zero;
    int negative;
    int overflow;
    int carry;
} dec_alu_result_t;

dec_alu_result_t dec_alu_add(uint32_t a, uint32_t b) {
    dec_alu_result_t r;
    uint32_t sum = a + b;
    r.result = sum;
    r.zero = (sum == 0);
    r.negative = ((int32_t)sum < 0);
    r.carry = (sum < a);
    r.overflow = (((a ^ b) & 0x80000000) == 0) && (((a ^ sum) & 0x80000000) != 0);
    return r;
}

dec_alu_result_t dec_alu_sub(uint32_t a, uint32_t b) {
    dec_alu_result_t r;
    uint32_t diff = a - b;
    r.result = diff;
    r.zero = (diff == 0);
    r.negative = ((int32_t)diff < 0);
    r.carry = (a < b);
    r.overflow = (((a ^ b) & 0x80000000) != 0) && (((a ^ diff) & 0x80000000) != 0);
    return r;
}

dec_alu_result_t dec_alu_and(uint32_t a, uint32_t b) {
    dec_alu_result_t r;
    r.result = a & b;
    r.zero = (r.result == 0);
    r.negative = ((int32_t)r.result < 0);
    r.overflow = 0;
    r.carry = 0;
    return r;
}

dec_alu_result_t dec_alu_or(uint32_t a, uint32_t b) {
    dec_alu_result_t r;
    r.result = a | b;
    r.zero = (r.result == 0);
    r.negative = ((int32_t)r.result < 0);
    r.overflow = 0;
    r.carry = 0;
    return r;
}

dec_alu_result_t dec_alu_xor(uint32_t a, uint32_t b) {
    dec_alu_result_t r;
    r.result = a ^ b;
    r.zero = (r.result == 0);
    r.negative = ((int32_t)r.result < 0);
    r.overflow = 0;
    r.carry = 0;
    return r;
}

dec_alu_result_t dec_alu_sll(uint32_t a, uint32_t shamt) {
    dec_alu_result_t r;
    r.result = a << (shamt & 0x1F);
    r.zero = (r.result == 0);
    r.negative = ((int32_t)r.result < 0);
    r.overflow = 0;
    r.carry = 0;
    return r;
}

int dec_alu_selftest(void) {
    dec_alu_result_t r = dec_alu_add(3, 4);
    if (r.result != 7) return -1;
    if (r.zero) return -2;
    r = dec_alu_sub(5, 5);
    if (!r.zero) return -3;
    r = dec_alu_and(0xFF, 0x0F);
    if (r.result != 0x0F) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1697: CPU ALU operations should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1697: Output should not be empty");
    assert!(
        code.contains("fn dec_alu_add"),
        "C1697: Should contain dec_alu_add function"
    );
    assert!(
        code.contains("fn dec_alu_sub"),
        "C1697: Should contain dec_alu_sub function"
    );
}

#[test]
fn c1698_cpu_condition_flags() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef int int32_t;

typedef struct {
    int z;
    int n;
    int c;
    int v;
} dec_cpu_flags_t;

void dec_cpu_flags_init(dec_cpu_flags_t *f) {
    f->z = 0;
    f->n = 0;
    f->c = 0;
    f->v = 0;
}

void dec_cpu_flags_update_nz(dec_cpu_flags_t *f, uint32_t result) {
    f->z = (result == 0);
    f->n = ((int32_t)result < 0);
}

void dec_cpu_flags_update_add(dec_cpu_flags_t *f, uint32_t a, uint32_t b, uint32_t result) {
    f->z = (result == 0);
    f->n = ((int32_t)result < 0);
    f->c = (result < a);
    f->v = (((a ^ b) & 0x80000000) == 0) && (((a ^ result) & 0x80000000) != 0);
}

void dec_cpu_flags_update_sub(dec_cpu_flags_t *f, uint32_t a, uint32_t b, uint32_t result) {
    f->z = (result == 0);
    f->n = ((int32_t)result < 0);
    f->c = (a >= b);
    f->v = (((a ^ b) & 0x80000000) != 0) && (((a ^ result) & 0x80000000) != 0);
}

int dec_cpu_flags_cond_eq(const dec_cpu_flags_t *f) { return f->z; }
int dec_cpu_flags_cond_ne(const dec_cpu_flags_t *f) { return !f->z; }
int dec_cpu_flags_cond_cs(const dec_cpu_flags_t *f) { return f->c; }
int dec_cpu_flags_cond_cc(const dec_cpu_flags_t *f) { return !f->c; }
int dec_cpu_flags_cond_mi(const dec_cpu_flags_t *f) { return f->n; }
int dec_cpu_flags_cond_pl(const dec_cpu_flags_t *f) { return !f->n; }
int dec_cpu_flags_cond_vs(const dec_cpu_flags_t *f) { return f->v; }

int dec_cpu_flags_cond_ge(const dec_cpu_flags_t *f) {
    return f->n == f->v;
}

int dec_cpu_flags_cond_lt(const dec_cpu_flags_t *f) {
    return f->n != f->v;
}

int dec_cpu_flags_cond_gt(const dec_cpu_flags_t *f) {
    return !f->z && (f->n == f->v);
}

int dec_cpu_flags_cond_le(const dec_cpu_flags_t *f) {
    return f->z || (f->n != f->v);
}

int dec_cpu_flags_selftest(void) {
    dec_cpu_flags_t f;
    dec_cpu_flags_init(&f);
    dec_cpu_flags_update_add(&f, 5, 3, 8);
    if (f.z) return -1;
    if (f.n) return -2;
    dec_cpu_flags_update_sub(&f, 5, 5, 0);
    if (!f.z) return -3;
    if (!dec_cpu_flags_cond_eq(&f)) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1698: CPU condition flags should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1698: Output should not be empty");
    assert!(
        code.contains("fn dec_cpu_flags_init"),
        "C1698: Should contain dec_cpu_flags_init function"
    );
    assert!(
        code.contains("fn dec_cpu_flags_cond_eq"),
        "C1698: Should contain dec_cpu_flags_cond_eq function"
    );
}

#[test]
fn c1699_cpu_pipeline_stage() {
    let c_code = r##"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t instr;
    uint32_t pc;
    int valid;
    int stall;
} dec_pipe_reg_t;

typedef struct {
    dec_pipe_reg_t fetch;
    dec_pipe_reg_t decode;
    dec_pipe_reg_t execute;
    dec_pipe_reg_t memory;
    dec_pipe_reg_t writeback;
    int flush_pending;
    uint32_t flush_target;
} dec_pipeline_t;

void dec_pipe_reg_init(dec_pipe_reg_t *pr) {
    pr->instr = 0;
    pr->pc = 0;
    pr->valid = 0;
    pr->stall = 0;
}

void dec_pipeline_init(dec_pipeline_t *p) {
    dec_pipe_reg_init(&p->fetch);
    dec_pipe_reg_init(&p->decode);
    dec_pipe_reg_init(&p->execute);
    dec_pipe_reg_init(&p->memory);
    dec_pipe_reg_init(&p->writeback);
    p->flush_pending = 0;
    p->flush_target = 0;
}

void dec_pipeline_advance(dec_pipeline_t *p) {
    if (!p->writeback.stall) {
        p->writeback = p->memory;
    }
    if (!p->memory.stall) {
        p->memory = p->execute;
    }
    if (!p->execute.stall) {
        p->execute = p->decode;
    }
    if (!p->decode.stall) {
        p->decode = p->fetch;
    }
}

void dec_pipeline_flush(dec_pipeline_t *p, uint32_t target) {
    dec_pipe_reg_init(&p->fetch);
    dec_pipe_reg_init(&p->decode);
    dec_pipe_reg_init(&p->execute);
    dec_pipe_reg_init(&p->memory);
    p->flush_pending = 0;
    p->flush_target = target;
}

void dec_pipeline_insert_bubble(dec_pipeline_t *p) {
    dec_pipe_reg_init(&p->execute);
    p->decode.stall = 1;
    p->fetch.stall = 1;
}

int dec_pipeline_is_stalled(const dec_pipeline_t *p) {
    return p->fetch.stall || p->decode.stall;
}

int dec_pipeline_active_stages(const dec_pipeline_t *p) {
    int count = 0;
    if (p->fetch.valid) count++;
    if (p->decode.valid) count++;
    if (p->execute.valid) count++;
    if (p->memory.valid) count++;
    if (p->writeback.valid) count++;
    return count;
}

int dec_pipeline_selftest(void) {
    dec_pipeline_t p;
    dec_pipeline_init(&p);
    if (dec_pipeline_active_stages(&p) != 0) return -1;
    p.fetch.valid = 1;
    p.fetch.pc = 0x1000;
    p.fetch.instr = 0xDEADBEEF;
    dec_pipeline_advance(&p);
    if (!p.decode.valid) return -2;
    if (p.decode.pc != 0x1000) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1699: CPU pipeline stage should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1699: Output should not be empty");
    assert!(
        code.contains("fn dec_pipeline_init"),
        "C1699: Should contain dec_pipeline_init function"
    );
    assert!(
        code.contains("fn dec_pipeline_advance"),
        "C1699: Should contain dec_pipeline_advance function"
    );
}

#[test]
fn c1700_cpu_instruction_fetch() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t mem[4096];
    uint32_t base_addr;
    int size;
} dec_fetch_mem_t;

void dec_fetch_mem_init(dec_fetch_mem_t *m, uint32_t base) {
    int i;
    for (i = 0; i < 4096; i++) {
        m->mem[i] = 0;
    }
    m->base_addr = base;
    m->size = 4096;
}

void dec_fetch_store_word(dec_fetch_mem_t *m, uint32_t addr, uint32_t val) {
    uint32_t offset = addr - m->base_addr;
    if (offset + 3 < 4096) {
        m->mem[offset]     = (uint8_t)(val & 0xFF);
        m->mem[offset + 1] = (uint8_t)((val >> 8) & 0xFF);
        m->mem[offset + 2] = (uint8_t)((val >> 16) & 0xFF);
        m->mem[offset + 3] = (uint8_t)((val >> 24) & 0xFF);
    }
}

uint32_t dec_fetch_load_word(const dec_fetch_mem_t *m, uint32_t addr) {
    uint32_t offset = addr - m->base_addr;
    if (offset + 3 < 4096) {
        return (uint32_t)m->mem[offset] |
               ((uint32_t)m->mem[offset + 1] << 8) |
               ((uint32_t)m->mem[offset + 2] << 16) |
               ((uint32_t)m->mem[offset + 3] << 24);
    }
    return 0;
}

typedef struct {
    uint32_t instr;
    uint32_t addr;
    int valid;
    int fault;
} dec_fetch_result_t;

dec_fetch_result_t dec_fetch_instruction(const dec_fetch_mem_t *m, uint32_t pc) {
    dec_fetch_result_t r;
    r.addr = pc;
    r.fault = 0;
    r.valid = 0;
    if (pc < m->base_addr || pc >= m->base_addr + (uint32_t)m->size - 3) {
        r.fault = 1;
        r.instr = 0;
        return r;
    }
    r.instr = dec_fetch_load_word(m, pc);
    r.valid = 1;
    return r;
}

int dec_fetch_is_aligned(uint32_t addr) {
    return (addr & 0x03) == 0;
}

int dec_fetch_in_range(const dec_fetch_mem_t *m, uint32_t addr) {
    return addr >= m->base_addr && addr < m->base_addr + (uint32_t)m->size;
}

int dec_fetch_selftest(void) {
    dec_fetch_mem_t mem;
    dec_fetch_mem_init(&mem, 0x1000);
    dec_fetch_store_word(&mem, 0x1000, 0xDEADBEEF);
    uint32_t loaded = dec_fetch_load_word(&mem, 0x1000);
    if (loaded != 0xDEADBEEF) return -1;
    dec_fetch_result_t r = dec_fetch_instruction(&mem, 0x1000);
    if (!r.valid) return -2;
    if (r.instr != 0xDEADBEEF) return -3;
    dec_fetch_result_t bad = dec_fetch_instruction(&mem, 0x0000);
    if (!bad.fault) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1700: CPU instruction fetch should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1700: Output should not be empty");
    assert!(
        code.contains("fn dec_fetch_mem_init"),
        "C1700: Should contain dec_fetch_mem_init function"
    );
    assert!(
        code.contains("fn dec_fetch_instruction"),
        "C1700: Should contain dec_fetch_instruction function"
    );
}
