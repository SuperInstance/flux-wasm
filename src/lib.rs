//! FLUX WebAssembly VM
//! A FLUX bytecode interpreter that runs in any browser or WASM runtime

use wasm_bindgen::prelude::*;
use std::sync::{Arc, Mutex};

// Simple hex encoding/decoding functions (no external deps)
fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.trim();
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }

    let mut bytes = Vec::new();
    for i in (0..hex.len()).step_by(2) {
        let byte_str = &hex[i..i+2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|e| format!("Invalid hex: {}", e))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

mod vm;
mod assembler;
mod markdown;

use vm::{FluxVM, Opcode};
use assembler::Assembler;
use markdown::MarkdownCompiler;

// Export panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    // console_error_panic_hook::set_once();
}

/// WASM-exported FLUX Virtual Machine
#[wasm_bindgen]
pub struct FluxWasmVM {
    vm: FluxVM,
    assembler: Assembler,
    markdown_compiler: MarkdownCompiler,
    message_queue: Arc<Mutex<Vec<Message>>>,
}

/// A2A (Agent-to-Agent) message
#[derive(Debug, Clone)]
struct Message {
    from_agent: String,
    to_agent: String,
    payload: String,
}

#[wasm_bindgen]
impl FluxWasmVM {
    /// Create a new FLUX VM instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            vm: FluxVM::new(),
            assembler: Assembler::new(),
            markdown_compiler: MarkdownCompiler::new(),
            message_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Load bytecode into the VM
    pub fn load(&mut self, bytecode: &[u8]) {
        self.vm.load(bytecode);
    }

    /// Execute one instruction
    /// Returns true if execution should continue, false if halted
    pub fn step(&mut self) -> bool {
        match self.vm.step() {
            Ok(_) => !self.vm.halted,
            Err(_) => false,
        }
    }

    /// Run the program until halt
    /// Returns the number of cycles executed
    pub fn run(&mut self) -> u32 {
        self.vm.run()
    }

    /// Read a register value
    pub fn read_reg(&self, idx: u32) -> i32 {
        if idx < 16 {
            self.vm.registers[idx as usize]
        } else {
            0
        }
    }

    /// Write a register value
    pub fn write_reg(&mut self, idx: u32, val: i32) {
        if idx < 16 {
            self.vm.registers[idx as usize] = val;
        }
    }

    /// Get the program counter
    pub fn get_pc(&self) -> u32 {
        self.vm.pc as u32
    }

    /// Check if the VM is halted
    pub fn is_halted(&self) -> bool {
        self.vm.halted
    }

    /// Get the cycle count
    pub fn get_cycles(&self) -> u32 {
        self.vm.cycle_count
    }

    /// Get register state as a JavaScript array
    pub fn get_registers(&self) -> Vec<i32> {
        self.vm.registers.to_vec()
    }

    /// Reset the VM
    pub fn reset(&mut self) {
        self.vm.reset();
    }

    /// Assemble FLUX assembly source code to bytecode
    pub fn assemble(&mut self, source: &str) -> Result<Vec<u8>, JsValue> {
        self.assembler
            .assemble(source)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Interpret markdown/natural language and compile to bytecode
    pub fn interpret_markdown(&mut self, md: &str) -> Result<Vec<u8>, JsValue> {
        self.markdown_compiler
            .compile(md)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Disassemble bytecode at the current PC
    pub fn disassemble(&self, count: usize) -> Vec<JsValue> {
        let lines = self.vm.disassemble(self.vm.pc, count);
        lines
            .into_iter()
            .map(|l| JsValue::from_str(&l))
            .collect()
    }

    /// Get memory as a byte array
    pub fn get_memory(&self) -> Vec<u8> {
        self.vm.memory.clone()
    }

    /// Read a byte from memory
    pub fn read_memory(&self, addr: u32) -> u8 {
        if (addr as usize) < self.vm.memory.len() {
            self.vm.memory[addr as usize]
        } else {
            0
        }
    }

    /// Write a byte to memory
    pub fn write_memory(&mut self, addr: u32, val: u8) -> bool {
        if (addr as usize) < self.vm.memory.len() {
            self.vm.memory[addr as usize] = val;
            true
        } else {
            false
        }
    }

    /// Get stack pointer
    pub fn get_sp(&self) -> u32 {
        self.vm.sp as u32
    }

    /// Get frame pointer
    pub fn get_fp(&self) -> u32 {
        self.vm.fp as u32
    }

    /// Get flag state as a comma-separated string: zero,carry,negative,overflow
    pub fn get_flags(&self) -> String {
        format!(
            "{},{},{},{}",
            self.vm.flags.zero as i32,
            self.vm.flags.carry as i32,
            self.vm.flags.negative as i32,
            self.vm.flags.overflow as i32
        )
    }

    /// Send a message to another agent (A2A)
    pub fn send_tell(&mut self, agent_id: &str, payload: &str) {
        let msg = Message {
            from_agent: "self".to_string(),
            to_agent: agent_id.to_string(),
            payload: payload.to_string(),
        };
        if let Ok(mut queue) = self.message_queue.lock() {
            queue.push(msg);
        }
    }

    /// Poll for incoming messages (A2A)
    /// Returns messages as JSON strings: from|to|payload
    pub fn poll_messages(&mut self) -> Vec<String> {
        let messages = if let Ok(mut queue) = self.message_queue.lock() {
            let msgs = queue.drain(..).collect::<Vec<_>>();
            msgs
        } else {
            Vec::new()
        };

        messages
            .into_iter()
            .map(|m| format!("{}|{}|{}", m.from_agent, m.to_agent, m.payload))
            .collect()
    }

    /// Broadcast a message to all agents (A2A)
    pub fn broadcast(&mut self, payload: &str) {
        let msg = Message {
            from_agent: "self".to_string(),
            to_agent: "*".to_string(),
            payload: payload.to_string(),
        };
        if let Ok(mut queue) = self.message_queue.lock() {
            queue.push(msg);
        }
    }

    /// Get VM statistics as a comma-separated string: cycles,pc,sp,fp,halted
    pub fn get_stats(&self) -> String {
        format!(
            "{},{},{},{},{}",
            self.vm.cycle_count,
            self.vm.pc,
            self.vm.sp,
            self.vm.fp,
            if self.vm.halted { 1 } else { 0 }
        )
    }

    /// Convenience: compile and run in one step
    pub fn compile_and_run(&mut self, source: &str) -> Result<u32, JsValue> {
        let bytecode = self.interpret_markdown(source)?;
        self.load(&bytecode);
        Ok(self.run())
    }

    /// Convenience: assemble and run in one step
    pub fn assemble_and_run(&mut self, source: &str) -> Result<u32, JsValue> {
        let bytecode = self.assemble(source)?;
        self.load(&bytecode);
        Ok(self.run())
    }

    /// Create a VM from a hex string
    pub fn from_hex(&mut self, hex: &str) -> Result<(), JsValue> {
        let bytecode = hex_decode(hex).map_err(|e| JsValue::from_str(&e))?;
        self.load(&bytecode);
        Ok(())
    }

    /// Get current bytecode as hex string
    pub fn to_hex(&self) -> String {
        let len = self.vm.memory.iter().rev().position(|&x| x != 0).unwrap_or(0);
        let actual_len = self.vm.memory.len() - len;
        hex_encode(&self.vm.memory[..actual_len])
    }

    /// Step with debug info
    /// Returns a JSON string with pcBefore,pcAfter,success,error
    pub fn step_debug(&mut self) -> Result<String, JsValue> {
        let pc_before = self.vm.pc;
        let result = self.vm.step();

        let mut debug_info = format!(
            "pcBefore={},pcAfter={},success={}",
            pc_before,
            self.vm.pc,
            if result.is_ok() { 1 } else { 0 }
        );

        if let Err(e) = result {
            debug_info.push_str(&format!(",error={}", e));
        }

        Ok(debug_info)
    }

    /// Breakpoint execution at a specific address
    pub fn set_breakpoint(&mut self, _addr: u32) {
        // TODO: Implement breakpoint support
    }

    /// Clear all breakpoints
    pub fn clear_breakpoints(&mut self) {
        // TODO: Implement breakpoint support
    }

    /// Get supported opcodes as an array of strings: "code:name"
    pub fn get_opcode_list(&self) -> Vec<String> {
        (0..=85)
            .filter_map(|i| Opcode::from_u8(i))
            .map(|op| format!("{}:{}", op as u32, op.name()))
            .collect()
    }
}

impl Default for FluxWasmVM {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for working with FLUX bytecode
#[wasm_bindgen]
pub struct FluxUtils;

#[wasm_bindgen]
impl FluxUtils {
    /// Validate bytecode
    pub fn validate_bytecode(bytecode: &[u8]) -> bool {
        !bytecode.is_empty()
    }

    /// Get bytecode info as a JSON string: length,isValid
    pub fn bytecode_info(bytecode: &[u8]) -> String {
        let is_valid = !bytecode.is_empty();
        let mut info = format!("length={},isValid={}", bytecode.len(), if is_valid { 1 } else { 0 });

        // Count instruction types
        let mut opcode_counts = std::collections::HashMap::new();
        for &byte in bytecode {
            if let Some(op) = Opcode::from_u8(byte) {
                *opcode_counts.entry(op.name()).or_insert(0) += 1;
            }
        }

        // Add top 5 opcodes
        let mut count_vec: Vec<_> = opcode_counts.into_iter().collect();
        count_vec.sort_by(|a, b| b.1.cmp(&a.1));
        for (op, count) in count_vec.iter().take(5) {
            info.push_str(&format!(",{}={}", op, count));
        }

        info
    }

    /// Convert bytecode to disassembly
    pub fn disassemble_static(bytecode: &[u8]) -> Vec<JsValue> {
        let mut vm = FluxVM::new();
        vm.load(bytecode);
        let lines = vm.disassemble(0, bytecode.len());
        lines.into_iter().map(|l| JsValue::from_str(&l)).collect()
    }

    /// Format bytecode as hex with offsets
    pub fn format_hex(bytecode: &[u8]) -> String {
        let mut result = String::new();
        for (i, chunk) in bytecode.chunks(16).enumerate() {
            result.push_str(&format!("{:04x}: ", i * 16));
            for (j, byte) in chunk.iter().enumerate() {
                result.push_str(&format!("{:02x} ", byte));
                if (j + 1) % 4 == 0 {
                    result.push(' ');
                }
            }
            result.push('\n');
        }
        result
    }
}

// WASM console logging utilities
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {
        $crate::log(&format_args!($($t)*).to_string())
    };
}

#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => {
        $crate::error(&format_args!($($t)*).to_string())
    };
}

// Add missing dependency for panic hook
// use console_error_panic_hook as _;
