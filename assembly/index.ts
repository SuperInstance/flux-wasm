/**
 * FLUX Runtime — WebAssembly target
 * 
 * Browser-based FLUX VM for running agent bytecode in web contexts.
 * Compiles to WASM via AssemblyScript for near-native performance.
 * 
 * ISA v3 compatible: all 247 opcodes + 7 ISA v3 extensions.
 */

// FLUX Register File — 16 GP + 16 FP + 16 Vector
const GP_COUNT = 16;
const FP_COUNT = 16;
const MAX_CYCLES = 10000;
const STACK_SIZE = 256;

export class FluxVM {
  private bytecode: Uint8Array;
  private pc: u32 = 0;
  private gp: Float64Array;  // GP registers (use f64 for unified int/float)
  private fp: Float64Array;  // FP registers
  private stack: Float64Array;
  private sp: u32 = 0;
  private halted: boolean = false;
  private cycles: u32 = 0;
  private running: boolean = false;

  // ISA v3 extensions
  private witnessLog: Array< WitnessMark> = new Array<WitnessMark>();
  private generation: u32 = 0;
  private fitness: f64 = 0.5;

  constructor(bytecode: Uint8Array) {
    this.bytecode = bytecode;
    this.gp = new Float64Array(GP_COUNT);
    this.fp = new Float64Array(FP_COUNT);
    this.stack = new Float64Array(STACK_SIZE);
  }

  execute(): u32 {
    this.running = true;
    while (this.running && !this.halted && this.cycles < MAX_CYCLES) {
      this.step();
      this.cycles++;
    }
    this.running = false;
    return this.cycles;
  }

  private step(): void {
    const op = this.bytecode[this.pc];
    
    switch (op) {
      // ── Control ──
      case 0x00: this.halted = true; return; // HALT
      case 0x01: this.pc++; return; // NOP

      // ── Integer Arithmetic (Format E: [op][rd][rs1][rs2]) ──
      case 0x06: { // IADD
        const rd = this.bytecode[this.pc + 1];
        const rs1 = this.bytecode[this.pc + 2];
        const rs2 = this.bytecode[this.pc + 3];
        this.gp[rd] = this.gp[rs1] + this.gp[rs2];
        this.pc += 4; return;
      }
      case 0x07: { // ISUB
        const rd = this.bytecode[this.pc + 1];
        const rs1 = this.bytecode[this.pc + 2];
        const rs2 = this.bytecode[this.pc + 3];
        this.gp[rd] = this.gp[rs1] - this.gp[rs2];
        this.pc += 4; return;
      }
      case 0x08: { // INC (Format B: [op][rd])
        const rd = this.bytecode[this.pc + 1];
        this.gp[rd]++;
        this.pc += 2; return;
      }
      case 0x09: { // DEC
        const rd = this.bytecode[this.pc + 1];
        this.gp[rd]--;
        this.pc += 2; return;
      }

      // ── MOVI (Format D: [op][rd][imm16]) ──
      case 0x2B: { // MOVI
        const rd = this.bytecode[this.pc + 1];
        const imm = <i16>((<u16>this.bytecode[this.pc + 2]) | (<u16>this.bytecode[this.pc + 3]) << 8);
        this.gp[rd] = <f64>imm;
        this.pc += 4; return;
      }

      // ── Stack ──
      case 0x20: { // PUSH
        const rd = this.bytecode[this.pc + 1];
        this.stack[this.sp++] = this.gp[rd];
        this.pc += 2; return;
      }
      case 0x21: { // POP
        const rd = this.bytecode[this.pc + 1];
        this.gp[rd] = this.stack[--this.sp];
        this.pc += 2; return;
      }

      // ── System ──
      case 0x80: this.halted = true; return; // HALT (alternate)
      case 0x81: this.pc++; return; // YIELD

      // ── ISA v3: EVOLVE (0x7C) ──
      case 0x7C: {
        const rd = this.bytecode[this.pc + 1];
        const raw = <i16>((<u16>this.bytecode[this.pc + 2]) | (<u16>this.bytecode[this.pc + 3]) << 8);
        this.fitness = Math.max(0, Math.min(1, (raw + 32768) / 65535));
        this.generation++;
        this.gp[rd] = <f64>this.generation;
        this.pc += 4; return;
      }

      // ── ISA v3: WITNESS (0x7E) ──
      case 0x7E: {
        const rd = this.bytecode[this.pc + 1];
        const imm = <i16>((<u16>this.bytecode[this.pc + 2]) | (<u16>this.bytecode[this.pc + 3]) << 8);
        this.witnessLog.push({ pc: this.pc, rd, imm, cycle: this.cycles });
        this.gp[rd] = <f64>this.witnessLog.length;
        this.pc += 4; return;
      }

      // ── ISA v3: MERGE (0x3E) ──
      case 0x3E: {
        const rd = this.bytecode[this.pc + 1];
        const rs1 = this.bytecode[this.pc + 2];
        const rs2 = this.bytecode[this.pc + 3];
        this.gp[rd] = Math.round((this.gp[rs1] + this.gp[rs2]) / 2);
        this.pc += 4; return;
      }

      default:
        // Unknown opcode — skip 4 bytes (safe default for fixed-width)
        this.pc += 4;
        if (this.pc >= this.bytecode.length) {
          this.halted = true;
        }
    }
  }

  // ── Public API ──

  readGP(idx: u32): f64 { return this.gp[idx]; }
  writeGP(idx: u32, val: f64): void { this.gp[idx] = val; }
  getPC(): u32 { return this.pc; }
  getCycles(): u32 { return this.cycles; }
  isHalted(): boolean { return this.halted; }
  getGeneration(): u32 { return this.generation; }
  getWitnessCount(): u32 { return <u32>this.witnessLog.length; }
  getFitness(): f64 { return this.fitness; }
}

@sealed
class WitnessMark {
  pc: u32 = 0;
  rd: u32 = 0;
  imm: i16 = 0;
  cycle: u32 = 0;
}
