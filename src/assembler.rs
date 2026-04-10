//! FLUX Assembler - Text to bytecode conversion
//! Two-pass assembler with label support

use crate::vm::Opcode;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum TokenKind {
    Identifier(String),
    Register(u8),
    Number(i32),
    LabelDef(String),
    LabelRef(String),
    Comma,
    Colon,
    LeftBracket,
    RightBracket,
    Plus,
    Minus,
    Star,
    Slash,
    Newline,
    Comment,
}

pub struct Assembler {
    source: String,
    tokens: Vec<Token>,
    labels: HashMap<String, usize>,
    bytecode: Vec<u8>,
    current_addr: usize,
    errors: Vec<AssemblerError>,
}

#[derive(Debug, Clone)]
pub enum AssemblerError {
    SyntaxError { line: usize, msg: String },
    UndefinedLabel { name: String },
    InvalidRegister { name: String },
}

impl std::fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblerError::SyntaxError { line, msg } => {
                write!(f, "Syntax error at line {}: {}", line, msg)
            }
            AssemblerError::UndefinedLabel { name } => {
                write!(f, "Undefined label: {}", name)
            }
            AssemblerError::InvalidRegister { name } => {
                write!(f, "Invalid register: {}", name)
            }
        }
    }
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            source: String::new(),
            tokens: Vec::new(),
            labels: HashMap::new(),
            bytecode: Vec::new(),
            current_addr: 0,
            errors: Vec::new(),
        }
    }

    pub fn assemble(&mut self, source: &str) -> Result<Vec<u8>, String> {
        self.source = source.to_string();
        self.tokens.clear();
        self.labels.clear();
        self.bytecode.clear();
        self.current_addr = 0;
        self.errors.clear();

        // Tokenize
        self.tokenize()?;

        // First pass: collect labels
        self.first_pass()?;

        // Second pass: generate bytecode
        self.second_pass()?;

        if !self.errors.is_empty() {
            return Err(self.errors[0].to_string());
        }

        Ok(self.bytecode.clone())
    }

    fn tokenize(&mut self) -> Result<(), String> {
        let chars: Vec<char> = self.source.chars().collect();
        let mut pos = 0;
        let mut line = 1;
        let mut col = 1;

        while pos < chars.len() {
            let ch = chars[pos];

            match ch {
                ' ' | '\t' | '\r' => {
                    col += 1;
                    pos += 1;
                }
                '\n' => {
                    self.tokens.push(Token {
                        kind: TokenKind::Newline,
                        line,
                        col,
                    });
                    line += 1;
                    col = 1;
                    pos += 1;
                }
                ';' => {
                    // Comment until end of line
                    while pos < chars.len() && chars[pos] != '\n' {
                        pos += 1;
                    }
                    self.tokens.push(Token {
                        kind: TokenKind::Comment,
                        line,
                        col,
                    });
                }
                ',' => {
                    self.tokens.push(Token {
                        kind: TokenKind::Comma,
                        line,
                        col,
                    });
                    col += 1;
                    pos += 1;
                }
                ':' => {
                    self.tokens.push(Token {
                        kind: TokenKind::Colon,
                        line,
                        col,
                    });
                    col += 1;
                    pos += 1;
                }
                '[' => {
                    self.tokens.push(Token {
                        kind: TokenKind::LeftBracket,
                        line,
                        col,
                    });
                    col += 1;
                    pos += 1;
                }
                ']' => {
                    self.tokens.push(Token {
                        kind: TokenKind::RightBracket,
                        line,
                        col,
                    });
                    col += 1;
                    pos += 1;
                }
                '+' => {
                    self.tokens.push(Token {
                        kind: TokenKind::Plus,
                        line,
                        col,
                    });
                    col += 1;
                    pos += 1;
                }
                '-' => {
                    // Check if this is a negative number
                    if chars.get(pos + 1).map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        let start = pos;
                        pos += 1; // consume '-'
                        while pos < chars.len() && chars[pos].is_ascii_digit() {
                            pos += 1;
                        }
                        let num_str: String = chars[start..pos].iter().collect();
                        let num = num_str.parse::<i32>().map_err(|e| e.to_string())?;
                        self.tokens.push(Token {
                            kind: TokenKind::Number(num),
                            line,
                            col: start as usize + 1,
                        });
                        col = (pos - start) as usize + 1;
                    } else {
                        self.tokens.push(Token {
                            kind: TokenKind::Minus,
                            line,
                            col,
                        });
                        col += 1;
                        pos += 1;
                    }
                }
                '*' => {
                    self.tokens.push(Token {
                        kind: TokenKind::Star,
                        line,
                        col,
                    });
                    col += 1;
                    pos += 1;
                }
                '/' => {
                    self.tokens.push(Token {
                        kind: TokenKind::Slash,
                        line,
                        col,
                    });
                    col += 1;
                    pos += 1;
                }
                '0'..='9' => {
                    let start = pos;
                    while pos < chars.len() && chars[pos].is_ascii_digit() {
                        pos += 1;
                    }
                    let num_str: String = chars[start..pos].iter().collect();
                    let num = num_str.parse::<i32>().map_err(|e| e.to_string())?;
                    self.tokens.push(Token {
                        kind: TokenKind::Number(num),
                        line,
                        col: start as usize + 1,
                    });
                    col = (pos - start) as usize + 1;
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let start = pos;
                    while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                        pos += 1;
                    }
                    let ident: String = chars[start..pos].iter().collect();
                    self.tokens.push(Token {
                        kind: TokenKind::Identifier(ident),
                        line,
                        col: start as usize + 1,
                    });
                    col = (pos - start) as usize + 1;
                }
                _ => {
                    return Err(format!("Unexpected character '{}' at line {} col {}", ch, line, col));
                }
            }
        }

        Ok(())
    }

    fn first_pass(&mut self) -> Result<(), String> {
        let mut i = 0;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::Identifier(name) => {
                    // Check if it's a label definition (followed by colon)
                    if i + 1 < self.tokens.len() {
                        if let TokenKind::Colon = &self.tokens[i + 1].kind {
                            self.labels.insert(name.clone(), self.current_addr);
                            i += 2;
                            continue;
                        }
                    }

                    // Check if it's an instruction
                    if let Some(_) = Self::opcode_from_name(name) {
                        i += 1;
                        self.current_addr += 1; // Opcode byte
                        continue;
                    }

                    // Check if it's a register (R0-R15)
                    if let Some(_) = Self::parse_register(name) {
                        i += 1;
                        continue;
                    }

                    // It's a label reference
                    self.current_addr += 4; // Address size
                    i += 1;
                }
                TokenKind::Register(_) => {
                    i += 1;
                }
                TokenKind::Number(_) => {
                    self.current_addr += 4; // Immediate value size
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        Ok(())
    }

    fn second_pass(&mut self) -> Result<(), String> {
        let mut i = 0;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::Identifier(name) => {
                    // Check if it's a label definition (skip)
                    if i + 1 < self.tokens.len() {
                        if let TokenKind::Colon = &self.tokens[i + 1].kind {
                            i += 2;
                            continue;
                        }
                    }

                    // Check if it's an instruction
                    if let Some(opcode) = Self::opcode_from_name(name) {
                        self.bytecode.push(opcode as u8);
                        i += 1;

                        // Parse operands based on instruction
                        self.parse_instruction_operands(opcode, &mut i)?;
                        continue;
                    }

                    // Check if it's a label reference (for jumps/calls)
                    if self.labels.contains_key(name) {
                        let addr = self.labels[name] as u32;
                        self.bytecode.extend_from_slice(&addr.to_le_bytes());
                        i += 1;
                        continue;
                    }

                    // Try parsing as register
                    if let Some(reg) = Self::parse_register(name) {
                        self.bytecode.push(reg);
                        i += 1;
                        continue;
                    }

                    return Err(format!("Unknown identifier: {}", name));
                }
                TokenKind::Register(reg) => {
                    self.bytecode.push(*reg);
                    i += 1;
                }
                TokenKind::Number(num) => {
                    self.bytecode.extend_from_slice(&num.to_le_bytes());
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        Ok(())
    }

    fn parse_instruction_operands(&mut self, opcode: Opcode, i: &mut usize) -> Result<(), String> {
        match opcode {
            Opcode::MOVI => {
                // MOVI Rd, imm
                let rd = self.expect_register(i)?;
                self.bytecode.push(rd);
                self.expect_comma(i)?;
                let imm = self.expect_number(i)?;
                self.bytecode.extend_from_slice(&imm.to_le_bytes());
            }
            Opcode::JMP | Opcode::JZ | Opcode::JNZ | Opcode::JE | Opcode::JNE |
            Opcode::JL | Opcode::JG | Opcode::CALL => {
                // Jump/call instructions: addr or label
                if *i < self.tokens.len() {
                    if let TokenKind::Identifier(ref name) = self.tokens[*i].kind {
                        if let Some(&addr) = self.labels.get(name) {
                            self.bytecode.extend_from_slice(&(addr as u32).to_le_bytes());
                            *i += 1;
                            return Ok(());
                        }
                    }
                    if let TokenKind::Number(addr) = self.tokens[*i].kind {
                        self.bytecode.extend_from_slice(&(addr as u32).to_le_bytes());
                        *i += 1;
                        return Ok(());
                    }
                }
                return Err("Expected address or label".to_string());
            }
            Opcode::MOV | Opcode::ADD | Opcode::SUB | Opcode::MUL | Opcode::DIV |
            Opcode::MOD | Opcode::AND | Opcode::OR | Opcode::XOR |
            Opcode::SHL | Opcode::SHR | Opcode::EQ | Opcode::NE | Opcode::LT |
            Opcode::GT | Opcode::LE | Opcode::GE => {
                // Two-register instructions: OP Rd, Rs
                let rd = self.expect_register(i)?;
                self.bytecode.push(rd);
                self.expect_comma(i)?;
                let rs = self.expect_register(i)?;
                self.bytecode.push(rs);
            }
            Opcode::NEG | Opcode::INC | Opcode::DEC | Opcode::NOT | Opcode::TEST |
            Opcode::PUSH | Opcode::POP => {
                // Single-register instructions: OP Rd
                let rd = self.expect_register(i)?;
                self.bytecode.push(rd);
            }
            Opcode::HALT | Opcode::RET | Opcode::NOP | Opcode::PUSHA | Opcode::POPA => {
                // No operands
            }
            _ => {
                // Default: try to parse as two registers
                if *i < self.tokens.len() {
                    if let TokenKind::Register(reg) = self.tokens[*i].kind {
                        self.bytecode.push(reg);
                        *i += 1;
                        if *i < self.tokens.len() {
                            if let TokenKind::Comma = self.tokens[*i].kind {
                                *i += 1;
                                if *i < self.tokens.len() {
                                    if let TokenKind::Register(reg2) = self.tokens[*i].kind {
                                        self.bytecode.push(reg2);
                                        *i += 1;
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn expect_register(&self, i: &mut usize) -> Result<u8, String> {
        if *i >= self.tokens.len() {
            return Err("Expected register".to_string());
        }

        match &self.tokens[*i].kind {
            TokenKind::Register(reg) => {
                *i += 1;
                Ok(*reg)
            }
            TokenKind::Identifier(name) => {
                if let Some(reg) = Self::parse_register(name) {
                    *i += 1;
                    Ok(reg)
                } else {
                    Err(format!("Expected register, got {}", name))
                }
            }
            _ => Err("Expected register".to_string()),
        }
    }

    fn expect_number(&self, i: &mut usize) -> Result<i32, String> {
        if *i >= self.tokens.len() {
            return Err("Expected number".to_string());
        }

        match &self.tokens[*i].kind {
            TokenKind::Number(num) => {
                *i += 1;
                Ok(*num)
            }
            _ => Err("Expected number".to_string()),
        }
    }

    fn expect_comma(&self, i: &mut usize) -> Result<(), String> {
        if *i < self.tokens.len() {
            if let TokenKind::Comma = self.tokens[*i].kind {
                *i += 1;
                return Ok(());
            }
        }
        Err("Expected comma".to_string())
    }

    fn opcode_from_name(name: &str) -> Option<Opcode> {
        match name.to_uppercase().as_str() {
            "ADD" => Some(Opcode::ADD),
            "SUB" => Some(Opcode::SUB),
            "MUL" => Some(Opcode::MUL),
            "DIV" => Some(Opcode::DIV),
            "MOD" => Some(Opcode::MOD),
            "NEG" => Some(Opcode::NEG),
            "INC" => Some(Opcode::INC),
            "DEC" => Some(Opcode::DEC),
            "AND" => Some(Opcode::AND),
            "OR" => Some(Opcode::OR),
            "XOR" => Some(Opcode::XOR),
            "NOT" => Some(Opcode::NOT),
            "SHL" => Some(Opcode::SHL),
            "SHR" => Some(Opcode::SHR),
            "USHR" => Some(Opcode::USHR),
            "ROL" => Some(Opcode::ROL),
            "CMP" => Some(Opcode::CMP),
            "EQ" => Some(Opcode::EQ),
            "NE" => Some(Opcode::NE),
            "LT" => Some(Opcode::LT),
            "GT" => Some(Opcode::GT),
            "LE" => Some(Opcode::LE),
            "GE" => Some(Opcode::GE),
            "TEST" => Some(Opcode::TEST),
            "MOV" => Some(Opcode::MOV),
            "MOVI" => Some(Opcode::MOVI),
            "LOAD" => Some(Opcode::LOAD),
            "STORE" => Some(Opcode::STORE),
            "PUSH" => Some(Opcode::PUSH),
            "POP" => Some(Opcode::POP),
            "PUSHA" => Some(Opcode::PUSHA),
            "POPA" => Some(Opcode::POPA),
            "JMP" => Some(Opcode::JMP),
            "JZ" => Some(Opcode::JZ),
            "JNZ" => Some(Opcode::JNZ),
            "JE" => Some(Opcode::JE),
            "JNE" => Some(Opcode::JNE),
            "JL" => Some(Opcode::JL),
            "JG" => Some(Opcode::JG),
            "CALL" => Some(Opcode::CALL),
            "RET" => Some(Opcode::RET),
            "HALT" => Some(Opcode::HALT),
            "NOP" => Some(Opcode::NOP),
            _ => None,
        }
    }

    fn parse_register(name: &str) -> Option<u8> {
        let upper = name.to_uppercase();
        if upper.starts_with('R') {
            upper[1..].parse::<u8>().ok()
        } else {
            None
        }
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        let mut asm = Assembler::new();
        let source = r#"
            MOVI R0, 42
            MOVI R1, 10
            ADD R0, R1
            HALT
        "#;

        let bytecode = asm.assemble(source).unwrap();
        assert!(!bytecode.is_empty());
    }

    #[test]
    fn test_labels() {
        let mut asm = Assembler::new();
        let source = r#"
            MOVI R0, 0
        loop:
            INC R0
            MOVI R1, 10
            CMP R0, R1
            JNE loop
            HALT
        "#;

        let bytecode = asm.assemble(source).unwrap();
        assert!(!bytecode.is_empty());
    }
}
