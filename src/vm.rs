//! Core FLUX bytecode Virtual Machine
//! Shared between WASM and native implementations

use std::fmt;

/// FLUX VM instruction opcodes (85 instructions total)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    // Arithmetic (0-7)
    ADD = 0,    // ADD Rd, Rs - Add Rs to Rd
    SUB = 1,    // SUB Rd, Rs - Subtract Rs from Rd
    MUL = 2,    // MUL Rd, Rs - Multiply Rd by Rs
    DIV = 3,    // DIV Rd, Rs - Divide Rd by Rs
    MOD = 4,    // MOD Rd, Rs - Modulo Rd by Rs
    NEG = 5,    // NEG Rd - Negate Rd
    INC = 6,    // INC Rd - Increment Rd
    DEC = 7,    // DEC Rd - Decrement Rd

    // Bitwise (8-15)
    AND = 8,    // AND Rd, Rs - Bitwise AND
    OR = 9,     // OR Rd, Rs - Bitwise OR
    XOR = 10,   // XOR Rd, Rs - Bitwise XOR
    NOT = 11,   // NOT Rd - Bitwise NOT
    SHL = 12,   // SHL Rd, Rs - Shift left
    SHR = 13,   // SHR Rd, Rs - Shift right (arithmetic)
    USHR = 14,  // USHR Rd, Rs - Shift right (unsigned)
    ROL = 15,   // ROL Rd, Rs - Rotate left

    // Comparison (16-23)
    CMP = 16,   // CMP Ra, Rb - Compare Ra and Rb, set flags
    EQ = 17,    // EQ Rd, Rs - Rd = (Rd == Rs) ? 1 : 0
    NE = 18,    // NE Rd, Rs - Rd = (Rd != Rs) ? 1 : 0
    LT = 19,    // LT Rd, Rs - Rd = (Rd < Rs) ? 1 : 0
    GT = 20,    // GT Rd, Rs - Rd = (Rd > Rs) ? 1 : 0
    LE = 21,    // LE Rd, Rs - Rd = (Rd <= Rs) ? 1 : 0
    GE = 22,    // GE Rd, Rs - Rd = (Rd >= Rs) ? 1 : 0
    TEST = 23,  // TEST Rd - Set flags based on Rd

    // Load/Store (24-31)
    MOV = 24,   // MOV Rd, Rs - Move Rs to Rd
    MOVI = 25,  // MOVI Rd, imm - Load immediate to Rd
    LOAD = 26,  // LOAD Rd, [Rs] - Load from address in Rs
    STORE = 27, // STORE [Rd], Rs - Store Rs to address in Rd
    LOADB = 28, // LOADB Rd, [Rs] - Load byte
    STOREB = 29,// STOREB [Rd], Rs - Store byte
    LOADW = 30, // LOADW Rd, [Rs] - Load word (16-bit)
    STOREW = 31,// STOREW [Rd], Rs - Store word

    // Stack (32-39)
    PUSH = 32,  // PUSH R - Push register to stack
    POP = 33,   // POP R - Pop from stack to register
    PUSHA = 34, // PUSHA - Push all registers
    POPA = 35,  // POPA - Pop all registers
    ENTER = 36, // ENTER imm - Create stack frame
    LEAVE = 37, // LEAVE - Destroy stack frame
    ALLOC = 38, // ALLOC R - Allocate stack space
    FREE = 39,  // FREE R - Free stack space

    // Control Flow (40-47)
    JMP = 40,   // JMP addr - Unconditional jump
    JZ = 41,    // JZ addr - Jump if zero
    JNZ = 42,   // JNZ addr - Jump if not zero
    JE = 43,    // JE addr - Jump if equal
    JNE = 44,   // JNE addr - Jump if not equal
    JL = 45,    // JL addr - Jump if less
    JG = 46,    // JG addr - Jump if greater
    CALL = 47,  // CALL addr - Call subroutine

    // More Control Flow (48-55)
    RET = 48,   // RET - Return from subroutine
    CALLI = 49, // CALLI R - Indirect call
    JMPI = 50,  // JMPI R - Indirect jump
    LOOP = 51,  // LOOP R, addr - Decrement R, jump if not zero
    LOOPI = 52, // LOOPI imm, addr - Loop with immediate counter
    SYSCALL = 53,// SYSCALL num - System call
    TRAP = 54,  // TRAP num - Software interrupt
    BRK = 55,   // BRK - Breakpoint

    // Memory (56-63)
    LDI = 56,   // LDI Rd, [addr] - Load indirect
    STI = 57,   // STI [addr], Rs - Store indirect
    LEA = 58,   // LEA Rd, [addr] - Load effective address
    LDPTR = 59, // LDPTR Rd, [Rs] - Load pointer
    STPTR = 60, // STPTR [Rd], Rs - Store pointer
    CPY = 61,   // CPY dst, src, len - Copy memory block
    FILL = 62,  // FILL addr, val, len - Fill memory
    ZERO = 63,  // ZERO addr, len - Zero memory

    // Floating Point (64-71)
    FADD = 64,  // FADD Rd, Rs - Floating add
    FSUB = 65,  // FSUB Rd, Rs - Floating subtract
    FMUL = 66,  // FMUL Rd, Rs - Floating multiply
    FDIV = 67,  // FDIV Rd, Rs - Floating divide
    F2I = 68,   // F2I Rd - Float to integer
    I2F = 69,   // I2F Rd - Integer to float
    FCMP = 70,  // FCMP Ra, Rb - Compare floats
    FMOVI = 71, // FMOVI Rd, imm - Load float immediate

    // Vector/SIMD (72-79)
    VADD = 72,  // VADD Vd, Vs - Vector add
    VSUB = 73,  // VSUB Vd, Vs - Vector subtract
    VMUL = 74,  // VMUL Vd, Vs - Vector multiply
    VDOT = 75,  // VDOT Vd, Vs, Vt - Dot product
    VLOAD = 76, // VLOAD Vd, [Rs] - Load vector
    VSTORE = 77,// VSTORE [Rd], Vs - Store vector
    VBROADCAST = 78, // VBROADCAST Vd, Rs - Broadcast scalar
    VEXTRACT = 79,   // VEXTRACT Rd, Vs, idx - Extract element

    // Agent-to-Agent (80-83)
    ASEND = 80, // ASEND agent_id, reg - Send to agent
    ARECV = 81, // ARECV Rd - Receive from agent
    AQUERY = 82,// AQUERY agent_id, Rd - Query agent state
    ABROADCAST = 83, // ABROADCAST reg - Broadcast to all

    // System (84-85)
    HALT = 84,  // HALT - Stop execution
    NOP = 85,   // NOP - No operation
}

impl Opcode {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 85 {
            Some(unsafe { std::mem::transmute(val) })
        } else {
            None
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Opcode::ADD => "ADD",
            Opcode::SUB => "SUB",
            Opcode::MUL => "MUL",
            Opcode::DIV => "DIV",
            Opcode::MOD => "MOD",
            Opcode::NEG => "NEG",
            Opcode::INC => "INC",
            Opcode::DEC => "DEC",
            Opcode::AND => "AND",
            Opcode::OR => "OR",
            Opcode::XOR => "XOR",
            Opcode::NOT => "NOT",
            Opcode::SHL => "SHL",
            Opcode::SHR => "SHR",
            Opcode::USHR => "USHR",
            Opcode::ROL => "ROL",
            Opcode::CMP => "CMP",
            Opcode::EQ => "EQ",
            Opcode::NE => "NE",
            Opcode::LT => "LT",
            Opcode::GT => "GT",
            Opcode::LE => "LE",
            Opcode::GE => "GE",
            Opcode::TEST => "TEST",
            Opcode::MOV => "MOV",
            Opcode::MOVI => "MOVI",
            Opcode::LOAD => "LOAD",
            Opcode::STORE => "STORE",
            Opcode::LOADB => "LOADB",
            Opcode::STOREB => "STOREB",
            Opcode::LOADW => "LOADW",
            Opcode::STOREW => "STOREW",
            Opcode::PUSH => "PUSH",
            Opcode::POP => "POP",
            Opcode::PUSHA => "PUSHA",
            Opcode::POPA => "POPA",
            Opcode::ENTER => "ENTER",
            Opcode::LEAVE => "LEAVE",
            Opcode::ALLOC => "ALLOC",
            Opcode::FREE => "FREE",
            Opcode::JMP => "JMP",
            Opcode::JZ => "JZ",
            Opcode::JNZ => "JNZ",
            Opcode::JE => "JE",
            Opcode::JNE => "JNE",
            Opcode::JL => "JL",
            Opcode::JG => "JG",
            Opcode::CALL => "CALL",
            Opcode::RET => "RET",
            Opcode::CALLI => "CALLI",
            Opcode::JMPI => "JMPI",
            Opcode::LOOP => "LOOP",
            Opcode::LOOPI => "LOOPI",
            Opcode::SYSCALL => "SYSCALL",
            Opcode::TRAP => "TRAP",
            Opcode::BRK => "BRK",
            Opcode::LDI => "LDI",
            Opcode::STI => "STI",
            Opcode::LEA => "LEA",
            Opcode::LDPTR => "LDPTR",
            Opcode::STPTR => "STPTR",
            Opcode::CPY => "CPY",
            Opcode::FILL => "FILL",
            Opcode::ZERO => "ZERO",
            Opcode::FADD => "FADD",
            Opcode::FSUB => "FSUB",
            Opcode::FMUL => "FMUL",
            Opcode::FDIV => "FDIV",
            Opcode::F2I => "F2I",
            Opcode::I2F => "I2F",
            Opcode::FCMP => "FCMP",
            Opcode::FMOVI => "FMOVI",
            Opcode::VADD => "VADD",
            Opcode::VSUB => "VSUB",
            Opcode::VMUL => "VMUL",
            Opcode::VDOT => "VDOT",
            Opcode::VLOAD => "VLOAD",
            Opcode::VSTORE => "VSTORE",
            Opcode::VBROADCAST => "VBROADCAST",
            Opcode::VEXTRACT => "VEXTRACT",
            Opcode::ASEND => "ASEND",
            Opcode::ARECV => "ARECV",
            Opcode::AQUERY => "AQUERY",
            Opcode::ABROADCAST => "ABROADCAST",
            Opcode::HALT => "HALT",
            Opcode::NOP => "NOP",
        }
    }
}

/// Processor flags
#[derive(Debug, Clone, Copy, Default)]
pub struct Flags {
    pub zero: bool,
    pub carry: bool,
    pub negative: bool,
    pub overflow: bool,
}

/// FLUX Virtual Machine core
#[derive(Debug, Clone)]
pub struct FluxVM {
    /// General purpose registers (R0-R15)
    pub registers: [i32; 16],
    /// Floating point registers (F0-F7)
    pub fregisters: [f64; 8],
    /// Vector registers (V0-V3, 4x f32 each)
    pub vregisters: [[f32; 4]; 4],
    /// Program counter
    pub pc: usize,
    /// Stack pointer (R13 by convention)
    pub sp: usize,
    /// Frame pointer (R14 by convention)
    pub fp: usize,
    /// Processor flags
    pub flags: Flags,
    /// Memory (64KB address space)
    pub memory: Vec<u8>,
    /// Halted state
    pub halted: bool,
    /// Cycle counter
    pub cycle_count: u32,
    /// Call stack for tracing
    pub call_stack: Vec<usize>,
}

impl FluxVM {
    const MEMORY_SIZE: usize = 64 * 1024; // 64KB
    const STACK_START: usize = 60 * 1024; // Stack starts at 60KB

    pub fn new() -> Self {
        let mut vm = Self {
            registers: [0; 16],
            fregisters: [0.0; 8],
            vregisters: [[0.0; 4]; 4],
            pc: 0,
            sp: Self::STACK_START,
            fp: Self::STACK_START,
            flags: Flags::default(),
            memory: vec![0; Self::MEMORY_SIZE],
            halted: false,
            cycle_count: 0,
            call_stack: Vec::new(),
        };

        // Initialize stack pointer in R13
        vm.registers[13] = vm.sp as i32;
        // Initialize frame pointer in R14
        vm.registers[14] = vm.fp as i32;

        vm
    }

    pub fn reset(&mut self) {
        self.registers = [0; 16];
        self.fregisters = [0.0; 8];
        self.vregisters = [[0.0; 4]; 4];
        self.pc = 0;
        self.sp = Self::STACK_START;
        self.fp = Self::STACK_START;
        self.flags = Flags::default();
        self.memory = vec![0; Self::MEMORY_SIZE];
        self.halted = false;
        self.cycle_count = 0;
        self.call_stack.clear();
        self.registers[13] = self.sp as i32;
        self.registers[14] = self.fp as i32;
    }

    pub fn load(&mut self, bytecode: &[u8]) {
        let len = bytecode.len().min(Self::MEMORY_SIZE);
        self.memory[0..len].copy_from_slice(&bytecode[..len]);
        self.pc = 0;
    }

    fn read_u8(&mut self) -> u8 {
        let val = self.memory[self.pc];
        self.pc += 1;
        val
    }

    fn read_u16(&mut self) -> u16 {
        let b0 = self.memory[self.pc] as u16;
        let b1 = self.memory[self.pc + 1] as u16;
        self.pc += 2;
        (b1 << 8) | b0
    }

    fn read_u32(&mut self) -> u32 {
        let b0 = self.memory[self.pc] as u32;
        let b1 = self.memory[self.pc + 1] as u32;
        let b2 = self.memory[self.pc + 2] as u32;
        let b3 = self.memory[self.pc + 3] as u32;
        self.pc += 4;
        (b3 << 24) | (b2 << 16) | (b1 << 8) | b0
    }

    fn read_i32(&mut self) -> i32 {
        self.read_u32() as i32
    }

    fn read_reg(&mut self) -> usize {
        let reg = self.read_u8() as usize;
        if reg >= 16 {
            panic!("Invalid register index: {}", reg);
        }
        reg
    }

    pub fn step(&mut self) -> Result<(), VMError> {
        if self.halted {
            return Err(VMError::Halted);
        }

        self.cycle_count += 1;

        let opcode_byte = self.read_u8();
        let opcode = match Opcode::from_u8(opcode_byte) {
            Some(op) => op,
            None => return Err(VMError::InvalidOpcode(opcode_byte)),
        };

        self.execute_instruction(opcode)?;

        Ok(())
    }

    pub fn run(&mut self) -> u32 {
        while !self.halted {
            if let Err(_) = self.step() {
                break;
            }
        }
        self.cycle_count
    }

    fn execute_instruction(&mut self, opcode: Opcode) -> Result<(), VMError> {
        match opcode {
            // Arithmetic
            Opcode::ADD => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                let result = self.registers[rd].wrapping_add(self.registers[rs]);
                self.set_flags_arith(self.registers[rd], self.registers[rs], result);
                self.registers[rd] = result;
            }
            Opcode::SUB => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                let result = self.registers[rd].wrapping_sub(self.registers[rs]);
                self.set_flags_arith(self.registers[rd], self.registers[rs], result);
                self.registers[rd] = result;
            }
            Opcode::MUL => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] = self.registers[rd].wrapping_mul(self.registers[rs]);
            }
            Opcode::DIV => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                if self.registers[rs] == 0 {
                    return Err(VMError::DivisionByZero);
                }
                self.registers[rd] /= self.registers[rs];
            }
            Opcode::MOD => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                if self.registers[rs] == 0 {
                    return Err(VMError::DivisionByZero);
                }
                self.registers[rd] %= self.registers[rs];
            }
            Opcode::NEG => {
                let rd = self.read_reg();
                self.registers[rd] = -self.registers[rd];
            }
            Opcode::INC => {
                let rd = self.read_reg();
                self.registers[rd] = self.registers[rd].wrapping_add(1);
            }
            Opcode::DEC => {
                let rd = self.read_reg();
                self.registers[rd] = self.registers[rd].wrapping_sub(1);
            }

            // Bitwise
            Opcode::AND => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] &= self.registers[rs];
            }
            Opcode::OR => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] |= self.registers[rs];
            }
            Opcode::XOR => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] ^= self.registers[rs];
            }
            Opcode::NOT => {
                let rd = self.read_reg();
                self.registers[rd] = !self.registers[rd];
            }
            Opcode::SHL => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] = self.registers[rd] << (self.registers[rs] & 0x1F);
            }
            Opcode::SHR => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] = self.registers[rd] >> (self.registers[rs] & 0x1F);
            }
            Opcode::USHR => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] = ((self.registers[rd] as u32) >> (self.registers[rs] & 0x1F)) as i32;
            }
            Opcode::ROL => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                let shift = self.registers[rs] & 0x1F;
                self.registers[rd] = self.registers[rd].rotate_left(shift as u32);
            }

            // Comparison
            Opcode::CMP => {
                let ra = self.read_reg();
                let rb = self.read_reg();
                let result = self.registers[ra].wrapping_sub(self.registers[rb]);
                self.set_flags_arith(self.registers[ra], self.registers[rb], result);
            }
            Opcode::EQ => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] = if self.registers[rd] == self.registers[rs] { 1 } else { 0 };
            }
            Opcode::NE => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] = if self.registers[rd] != self.registers[rs] { 1 } else { 0 };
            }
            Opcode::LT => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] = if self.registers[rd] < self.registers[rs] { 1 } else { 0 };
            }
            Opcode::GT => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] = if self.registers[rd] > self.registers[rs] { 1 } else { 0 };
            }
            Opcode::LE => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] = if self.registers[rd] <= self.registers[rs] { 1 } else { 0 };
            }
            Opcode::GE => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] = if self.registers[rd] >= self.registers[rs] { 1 } else { 0 };
            }
            Opcode::TEST => {
                let rd = self.read_reg();
                self.flags.zero = self.registers[rd] == 0;
                self.flags.negative = self.registers[rd] < 0;
            }

            // Load/Store
            Opcode::MOV => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.registers[rd] = self.registers[rs];
            }
            Opcode::MOVI => {
                let rd = self.read_reg();
                let imm = self.read_i32();
                self.registers[rd] = imm;
            }
            Opcode::LOAD => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                let addr = self.registers[rs] as usize;
                self.registers[rd] = self.read_memory_i32(addr)?;
            }
            Opcode::STORE => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                let addr = self.registers[rd] as usize;
                self.write_memory_i32(addr, self.registers[rs])?;
            }
            Opcode::LOADB => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                let addr = self.registers[rs] as usize;
                self.registers[rd] = self.read_memory_u8(addr) as i32;
            }
            Opcode::STOREB => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                let addr = self.registers[rd] as usize;
                self.write_memory_u8(addr, self.registers[rs] as u8)?;
            }
            Opcode::LOADW => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                let addr = self.registers[rs] as usize;
                self.registers[rd] = self.read_memory_u16(addr) as i32;
            }
            Opcode::STOREW => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                let addr = self.registers[rd] as usize;
                self.write_memory_u16(addr, self.registers[rs] as u16)?;
            }

            // Stack
            Opcode::PUSH => {
                let reg = self.read_reg();
                self.push(self.registers[reg])?;
            }
            Opcode::POP => {
                let reg = self.read_reg();
                self.registers[reg] = self.pop()?;
            }
            Opcode::PUSHA => {
                for &reg in &self.registers {
                    self.push(reg)?;
                }
            }
            Opcode::POPA => {
                for i in (0..16).rev() {
                    self.registers[i] = self.pop()?;
                }
            }
            Opcode::ENTER => {
                let imm = self.read_u16() as usize;
                self.push(self.fp as i32)?;
                self.fp = self.sp;
                self.sp = self.sp.wrapping_sub(imm);
                self.registers[13] = self.sp as i32;
                self.registers[14] = self.fp as i32;
            }
            Opcode::LEAVE => {
                self.sp = self.fp;
                self.fp = self.pop()? as usize;
                self.registers[13] = self.sp as i32;
                self.registers[14] = self.fp as i32;
            }
            Opcode::ALLOC => {
                let reg = self.read_reg();
                let size = self.registers[reg] as usize;
                self.sp = self.sp.wrapping_sub(size);
                self.registers[13] = self.sp as i32;
            }
            Opcode::FREE => {
                let reg = self.read_reg();
                let size = self.registers[reg] as usize;
                self.sp = self.sp.wrapping_add(size);
                self.registers[13] = self.sp as i32;
            }

            // Control Flow
            Opcode::JMP => {
                let addr = self.read_u32() as usize;
                self.pc = addr;
            }
            Opcode::JZ => {
                let addr = self.read_u32() as usize;
                if self.flags.zero {
                    self.pc = addr;
                }
            }
            Opcode::JNZ => {
                let addr = self.read_u32() as usize;
                if !self.flags.zero {
                    self.pc = addr;
                }
            }
            Opcode::JE => {
                let addr = self.read_u32() as usize;
                if self.flags.zero {
                    self.pc = addr;
                }
            }
            Opcode::JNE => {
                let addr = self.read_u32() as usize;
                if !self.flags.zero {
                    self.pc = addr;
                }
            }
            Opcode::JL => {
                let addr = self.read_u32() as usize;
                if self.flags.negative != self.flags.overflow {
                    self.pc = addr;
                }
            }
            Opcode::JG => {
                let addr = self.read_u32() as usize;
                if !self.flags.zero && self.flags.negative == self.flags.overflow {
                    self.pc = addr;
                }
            }
            Opcode::CALL => {
                let addr = self.read_u32() as usize;
                self.call(self.addr_to_linear(addr))?;
            }
            Opcode::RET => {
                self.ret()?;
            }
            Opcode::CALLI => {
                let reg = self.read_reg();
                let addr = self.registers[reg] as usize;
                self.call(addr)?;
            }
            Opcode::JMPI => {
                let reg = self.read_reg();
                self.pc = self.registers[reg] as usize;
            }
            Opcode::LOOP => {
                let reg = self.read_reg();
                let addr = self.read_u32() as usize;
                self.registers[reg] -= 1;
                if self.registers[reg] != 0 {
                    self.pc = addr;
                }
            }
            Opcode::LOOPI => {
                let imm = self.read_u32();
                let addr = self.read_u32() as usize;
                let counter = self.read_reg();
                self.registers[counter] = self.registers[counter].wrapping_sub(1);
                if self.registers[counter] != 0 {
                    self.pc = addr;
                }
            }
            Opcode::SYSCALL => {
                let num = self.read_u8();
                self.syscall(num)?;
            }
            Opcode::TRAP => {
                let num = self.read_u8();
                // Trap implementation - can be hooked
            }
            Opcode::BRK => {
                // Breakpoint - can be hooked by debugger
            }

            // Memory
            Opcode::LDI => {
                let rd = self.read_reg();
                let addr = self.read_u32() as usize;
                self.registers[rd] = self.read_memory_i32(addr)?;
            }
            Opcode::STI => {
                let addr = self.read_u32() as usize;
                let rs = self.read_reg();
                self.write_memory_i32(addr, self.registers[rs])?;
            }
            Opcode::LEA => {
                let rd = self.read_reg();
                let addr = self.read_u32();
                self.registers[rd] = addr as i32;
            }
            Opcode::LDPTR => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                let addr = self.registers[rs] as usize;
                self.registers[rd] = self.read_memory_i32(addr)?;
            }
            Opcode::STPTR => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                let addr = self.registers[rd] as usize;
                self.write_memory_i32(addr, self.registers[rs])?;
            }
            Opcode::CPY => {
                let dst = self.read_u32() as usize;
                let src = self.read_u32() as usize;
                let len = self.read_u32() as usize;
                self.copy_memory(dst, src, len)?;
            }
            Opcode::FILL => {
                let addr = self.read_u32() as usize;
                let val = self.read_u8();
                let len = self.read_u32() as usize;
                self.fill_memory(addr, val, len)?;
            }
            Opcode::ZERO => {
                let addr = self.read_u32() as usize;
                let len = self.read_u32() as usize;
                self.fill_memory(addr, 0, len)?;
            }

            // Floating Point
            Opcode::FADD => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.fregisters[rd] += self.fregisters[rs];
            }
            Opcode::FSUB => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.fregisters[rd] -= self.fregisters[rs];
            }
            Opcode::FMUL => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.fregisters[rd] *= self.fregisters[rs];
            }
            Opcode::FDIV => {
                let rd = self.read_reg();
                let rs = self.read_reg();
                self.fregisters[rd] /= self.fregisters[rs];
            }
            Opcode::F2I => {
                let rd = self.read_reg();
                self.registers[rd] = self.fregisters[rd] as i32;
            }
            Opcode::I2F => {
                let rd = self.read_reg();
                self.fregisters[rd] = self.registers[rd] as f64;
            }
            Opcode::FCMP => {
                let ra = self.read_reg();
                let rb = self.read_reg();
                self.flags.zero = self.fregisters[ra] == self.fregisters[rb];
                self.flags.negative = self.fregisters[ra] < self.fregisters[rb];
            }
            Opcode::FMOVI => {
                let rd = self.read_reg();
                let bits = self.read_u32();
                self.fregisters[rd] = f32::from_bits(bits) as f64;
            }

            // Vector/SIMD
            Opcode::VADD => {
                let vd = self.read_reg() & 3;
                let vs = self.read_reg() & 3;
                for i in 0..4 {
                    self.vregisters[vd][i] += self.vregisters[vs][i];
                }
            }
            Opcode::VSUB => {
                let vd = self.read_reg() & 3;
                let vs = self.read_reg() & 3;
                for i in 0..4 {
                    self.vregisters[vd][i] -= self.vregisters[vs][i];
                }
            }
            Opcode::VMUL => {
                let vd = self.read_reg() & 3;
                let vs = self.read_reg() & 3;
                for i in 0..4 {
                    self.vregisters[vd][i] *= self.vregisters[vs][i];
                }
            }
            Opcode::VDOT => {
                let vd = self.read_reg() & 3;
                let vs = self.read_reg() & 3;
                let vt = self.read_reg() & 3;
                let mut dot: f32 = 0.0;
                for i in 0..4 {
                    dot += self.vregisters[vs][i] * self.vregisters[vt][i];
                }
                self.vregisters[vd] = [dot; 4];
            }
            Opcode::VLOAD => {
                let vd = self.read_reg() & 3;
                let rs = self.read_reg();
                let addr = self.registers[rs] as usize;
                for i in 0..4 {
                    self.vregisters[vd][i] = self.read_memory_u32(addr + i * 4) as f32;
                }
            }
            Opcode::VSTORE => {
                let rd = self.read_reg();
                let vs = self.read_reg() & 3;
                let addr = self.registers[rd] as usize;
                for i in 0..4 {
                    self.write_memory_u32(addr + i * 4, self.vregisters[vs][i] as u32)?;
                }
            }
            Opcode::VBROADCAST => {
                let vd = self.read_reg() & 3;
                let rs = self.read_reg();
                let val = self.registers[rs] as f32;
                self.vregisters[vd] = [val; 4];
            }
            Opcode::VEXTRACT => {
                let rd = self.read_reg();
                let vs = self.read_reg() & 3;
                let idx = self.read_u8() as usize & 3;
                self.registers[rd] = self.vregisters[vs][idx] as i32;
            }

            // Agent-to-Agent
            Opcode::ASEND => {
                let agent_id = self.read_u8();
                let reg = self.read_reg();
                // A2A send - handled by wrapper
            }
            Opcode::ARECV => {
                let rd = self.read_reg();
                // A2A receive - handled by wrapper
            }
            Opcode::AQUERY => {
                let agent_id = self.read_u8();
                let rd = self.read_reg();
                // A2A query - handled by wrapper
            }
            Opcode::ABROADCAST => {
                let reg = self.read_reg();
                // A2A broadcast - handled by wrapper
            }

            // System
            Opcode::HALT => {
                self.halted = true;
            }
            Opcode::NOP => {
                // Do nothing
            }
        }

        Ok(())
    }

    fn set_flags_arith(&mut self, a: i32, b: i32, result: i32) {
        self.flags.zero = result == 0;
        self.flags.negative = result < 0;
        // Simplified overflow detection
        self.flags.overflow = (b > 0 && a > i32::MAX - b) || (b < 0 && a < i32::MIN - b);
    }

    fn call(&mut self, addr: usize) -> Result<(), VMError> {
        self.push(self.pc as i32)?;
        self.pc = addr;
        Ok(())
    }

    fn ret(&mut self) -> Result<(), VMError> {
        let ret_addr = self.pop()? as usize;
        self.pc = ret_addr;
        Ok(())
    }

    fn push(&mut self, val: i32) -> Result<(), VMError> {
        if self.sp < 4 {
            return Err(VMError::StackOverflow);
        }
        self.sp = self.sp.wrapping_sub(4);
        self.write_memory_i32(self.sp, val)?;
        self.registers[13] = self.sp as i32;
        Ok(())
    }

    fn pop(&mut self) -> Result<i32, VMError> {
        if self.sp >= Self::MEMORY_SIZE - 4 {
            return Err(VMError::StackUnderflow);
        }
        let val = self.read_memory_i32(self.sp)?;
        self.sp = self.sp.wrapping_add(4);
        self.registers[13] = self.sp as i32;
        Ok(val)
    }

    fn addr_to_linear(&self, addr: usize) -> usize {
        addr
    }

    fn syscall(&mut self, num: u8) -> Result<(), VMError> {
        match num {
            0 => {
                // Exit
                self.halted = true;
            }
            1 => {
                // Print integer in R0
                // Handled by wrapper
            }
            2 => {
                // Read integer to R0
                // Handled by wrapper
            }
            _ => {}
        }
        Ok(())
    }

    // Memory access helpers
    fn read_memory_u8(&self, addr: usize) -> u8 {
        if addr < Self::MEMORY_SIZE {
            self.memory[addr]
        } else {
            0
        }
    }

    fn read_memory_u16(&self, addr: usize) -> u16 {
        if addr + 1 < Self::MEMORY_SIZE {
            u16::from_le_bytes([self.memory[addr], self.memory[addr + 1]])
        } else {
            0
        }
    }

    fn read_memory_u32(&self, addr: usize) -> u32 {
        if addr + 3 < Self::MEMORY_SIZE {
            u32::from_le_bytes([
                self.memory[addr],
                self.memory[addr + 1],
                self.memory[addr + 2],
                self.memory[addr + 3],
            ])
        } else {
            0
        }
    }

    fn read_memory_i32(&self, addr: usize) -> Result<i32, VMError> {
        if addr + 3 < Self::MEMORY_SIZE {
            Ok(self.read_memory_u32(addr) as i32)
        } else {
            Err(VMError::MemoryAccessViolation(addr))
        }
    }

    fn write_memory_u8(&mut self, addr: usize, val: u8) -> Result<(), VMError> {
        if addr < Self::MEMORY_SIZE {
            self.memory[addr] = val;
            Ok(())
        } else {
            Err(VMError::MemoryAccessViolation(addr))
        }
    }

    fn write_memory_u16(&mut self, addr: usize, val: u16) -> Result<(), VMError> {
        if addr + 1 < Self::MEMORY_SIZE {
            let bytes = val.to_le_bytes();
            self.memory[addr] = bytes[0];
            self.memory[addr + 1] = bytes[1];
            Ok(())
        } else {
            Err(VMError::MemoryAccessViolation(addr))
        }
    }

    fn write_memory_u32(&mut self, addr: usize, val: u32) -> Result<(), VMError> {
        if addr + 3 < Self::MEMORY_SIZE {
            let bytes = val.to_le_bytes();
            self.memory[addr] = bytes[0];
            self.memory[addr + 1] = bytes[1];
            self.memory[addr + 2] = bytes[2];
            self.memory[addr + 3] = bytes[3];
            Ok(())
        } else {
            Err(VMError::MemoryAccessViolation(addr))
        }
    }

    fn write_memory_i32(&mut self, addr: usize, val: i32) -> Result<(), VMError> {
        self.write_memory_u32(addr, val as u32)
    }

    fn copy_memory(&mut self, dst: usize, src: usize, len: usize) -> Result<(), VMError> {
        if dst + len > Self::MEMORY_SIZE || src + len > Self::MEMORY_SIZE {
            return Err(VMError::MemoryAccessViolation(dst));
        }
        self.memory.copy_within(src..src + len, dst);
        Ok(())
    }

    fn fill_memory(&mut self, addr: usize, val: u8, len: usize) -> Result<(), VMError> {
        if addr + len > Self::MEMORY_SIZE {
            return Err(VMError::MemoryAccessViolation(addr));
        }
        for i in 0..len {
            self.memory[addr + i] = val;
        }
        Ok(())
    }

    pub fn disassemble(&self, start: usize, count: usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut pc = start;

        for _ in 0..count {
            if pc >= self.memory.len() {
                break;
            }

            let opcode_byte = self.memory[pc];
            pc += 1;

            let opcode = match Opcode::from_u8(opcode_byte) {
                Some(op) => op,
                None => {
                    result.push(format!("{:04x}: UNKNOWN({:02x})", pc - 1, opcode_byte));
                    continue;
                }
            };

            let instr = match opcode {
                Opcode::MOVI => {
                    let rd = self.memory[pc] as usize;
                    pc += 1;
                    let imm = i32::from_le_bytes([
                        self.memory[pc],
                        self.memory[pc + 1],
                        self.memory[pc + 2],
                        self.memory[pc + 3],
                    ]);
                    pc += 4;
                    format!("MOVI R{}, {}", rd, imm)
                }
                Opcode::JMP | Opcode::JZ | Opcode::JNZ | Opcode::JE | Opcode::JNE |
                Opcode::JL | Opcode::JG | Opcode::CALL => {
                    let addr = u32::from_le_bytes([
                        self.memory[pc],
                        self.memory[pc + 1],
                        self.memory[pc + 2],
                        self.memory[pc + 3],
                    ]);
                    pc += 4;
                    format!("{} 0x{:04x}", opcode.name(), addr)
                }
                Opcode::HALT | Opcode::RET | Opcode::NOP | Opcode::PUSHA | Opcode::POPA |
                Opcode::BRK => opcode.name().to_string(),
                _ => {
                    // Generic instruction with registers
                    let rd = self.memory[pc] as usize;
                    pc += 1;
                    if pc >= self.memory.len() {
                        format!("{}", opcode.name())
                    } else {
                        let rs = self.memory[pc] as usize;
                        pc += 1;
                        format!("{} R{}, R{}", opcode.name(), rd, rs)
                    }
                }
            };

            result.push(format!("{:04x}: {}", pc - instr.len() / 2 - 2, instr));
        }

        result
    }
}

impl Default for FluxVM {
    fn default() -> Self {
        Self::new()
    }
}

/// VM error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VMError {
    Halted,
    InvalidOpcode(u8),
    DivisionByZero,
    StackOverflow,
    StackUnderflow,
    MemoryAccessViolation(usize),
    InvalidRegister(usize),
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VMError::Halted => write!(f, "VM is halted"),
            VMError::InvalidOpcode(op) => write!(f, "Invalid opcode: 0x{:02x}", op),
            VMError::DivisionByZero => write!(f, "Division by zero"),
            VMError::StackOverflow => write!(f, "Stack overflow"),
            VMError::StackUnderflow => write!(f, "Stack underflow"),
            VMError::MemoryAccessViolation(addr) => write!(f, "Memory access violation at 0x{:04x}", addr),
            VMError::InvalidRegister(reg) => write!(f, "Invalid register: R{}", reg),
        }
    }
}

impl std::error::Error for VMError {}
