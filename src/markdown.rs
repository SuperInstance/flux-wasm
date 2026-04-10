//! FLUX Markdown/Natural Language to Bytecode Compiler
//! Interprets natural language patterns and generates FLUX bytecode

use crate::assembler::Assembler;

pub struct MarkdownCompiler {
    assembler: Assembler,
}

impl MarkdownCompiler {
    pub fn new() -> Self {
        Self {
            assembler: Assembler::new(),
        }
    }

    pub fn compile(&mut self, input: &str) -> Result<Vec<u8>, String> {
        let mut asm_code = String::new();

        for line in input.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Check for flux code blocks
            if line.starts_with("```flux") {
                continue;
            }
            if line == "```" {
                continue;
            }

            // Check for natural language patterns
            if let Some(code) = self.interpret_natural_language(line) {
                asm_code.push_str(&code);
                asm_code.push('\n');
            } else if line.starts_with(";") || line.starts_with("//") {
                // Comments - skip
                continue;
            } else {
                // Direct assembly
                asm_code.push_str(line);
                asm_code.push('\n');
            }
        }

        self.assembler.assemble(&asm_code)
    }

    fn interpret_natural_language(&self, input: &str) -> Option<String> {
        let lower = input.to_lowercase();

        // Pattern: "compute X + Y" or "calculate X + Y"
        if let Some(caps) = Self::regex(&lower, r"(compute|calculate)\s+(\d+)\s*\+\s*(\d+)") {
            let a: i32 = caps[2].parse().ok()?;
            let b: i32 = caps[3].parse().ok()?;
            return Some(format!(
                "MOVI R0, {}\nMOVI R1, {}\nADD R0, R1\nHALT",
                a, b
            ));
        }

        // Pattern: "compute X - Y"
        if let Some(caps) = Self::regex(&lower, r"(compute|calculate)\s+(\d+)\s*-\s*(\d+)") {
            let a: i32 = caps[2].parse().ok()?;
            let b: i32 = caps[3].parse().ok()?;
            return Some(format!(
                "MOVI R0, {}\nMOVI R1, {}\nSUB R0, R1\nHALT",
                a, b
            ));
        }

        // Pattern: "compute X * Y"
        if let Some(caps) = Self::regex(&lower, r"(compute|calculate)\s+(\d+)\s*\*\s*(\d+)") {
            let a: i32 = caps[2].parse().ok()?;
            let b: i32 = caps[3].parse().ok()?;
            return Some(format!(
                "MOVI R0, {}\nMOVI R1, {}\nMUL R0, R1\nHALT",
                a, b
            ));
        }

        // Pattern: "factorial of N"
        if let Some(caps) = Self::regex(&lower, r"factorial\s+(?:of\s+)?(\d+)") {
            let n: i32 = caps[1].parse().ok()?;
            return Some(self.generate_factorial(n));
        }

        // Pattern: "sum 1 to N" or "sum from 1 to N"
        if let Some(caps) = Self::regex(&lower, r"sum\s+(?:from\s+)?1\s+to\s+(\d+)") {
            let n: i32 = caps[1].parse().ok()?;
            return Some(self.generate_sum(n));
        }

        // Pattern: "count from A to B"
        if let Some(caps) = Self::regex(&lower, r"count\s+from\s+(\d+)\s+to\s+(\d+)") {
            let start: i32 = caps[1].parse().ok()?;
            let end: i32 = caps[2].parse().ok()?;
            return Some(self.generate_count(start, end));
        }

        // Pattern: "power X to the Y" or "X ^ Y"
        if let Some(caps) = Self::regex(&lower, r"(\d+)\s*\^\s*(\d+)") {
            let base: i32 = caps[1].parse().ok()?;
            let exp: i32 = caps[2].parse().ok()?;
            return Some(self.generate_power(base, exp));
        }

        // Pattern: "fibonacci N"
        if let Some(caps) = Self::regex(&lower, r"fibonacci\s+(\d+)") {
            let n: i32 = caps[1].parse().ok()?;
            return Some(self.generate_fibonacci(n));
        }

        // Pattern: "loop N times"
        if let Some(caps) = Self::regex(&lower, r"loop\s+(\d+)\s+times") {
            let n: i32 = caps[1].parse().ok()?;
            return Some(self.generate_simple_loop(n));
        }

        // Pattern: "store X in R0" or "set R0 to X"
        if let Some(caps) = Self::regex(&lower, r"(store|set)\s+(\d+)\s+(?:in\s+)?R(\d+)") {
            let val: i32 = caps[2].parse().ok()?;
            let reg: u8 = caps[3].parse().ok()?;
            return Some(format!("MOVI R{}, {}\nHALT", reg, val));
        }

        // Pattern: "add X to R0"
        if let Some(caps) = Self::regex(&lower, r"add\s+(\d+)\s+to\s+R(\d+)") {
            let val: i32 = caps[1].parse().ok()?;
            let reg: u8 = caps[2].parse().ok()?;
            return Some(format!("MOVI R{}, {}\nHALT", reg, val));
        }

        None
    }

    fn generate_factorial(&self, n: i32) -> String {
        format!(
            r#"
; Factorial of {}
MOVI R0, {n}       ; R0 = n
MOVI R1, 1         ; R1 = result = 1
MOVI R2, 1         ; R2 = counter = 1
loop:
CMP R0, R2         ; Compare n with counter
JL end             ; If n < counter, end
MUL R1, R2         ; result *= counter
INC R2             ; counter++
JMP loop           ; Repeat
end:
MOV R0, R1         ; Return result in R0
HALT
"#,
            n
        )
    }

    fn generate_sum(&self, n: i32) -> String {
        format!(
            r#"
; Sum from 1 to {}
MOVI R0, 0         ; R0 = sum = 0
MOVI R1, 1         ; R1 = counter = 1
MOVI R2, {n}       ; R2 = n
loop:
CMP R1, R2         ; Compare counter with n
JG end             ; If counter > n, end
ADD R0, R1         ; sum += counter
INC R1             ; counter++
JMP loop           ; Repeat
end:
HALT               ; R0 contains sum
"#,
            n
        )
    }

    fn generate_count(&self, start: i32, end: i32) -> String {
        format!(
            r#"
; Count from {} to {}
MOVI R0, {start}    ; R0 = current = start
MOVI R1, {end}      ; R1 = end
loop:
; R0 contains current value (for debugging/output)
CMP R0, R1         ; Compare current with end
JG end             ; If current > end, done
INC R0             ; current++
JMP loop           ; Repeat
end:
HALT               ; Finished
"#,
            start, end
        )
    }

    fn generate_power(&self, base: i32, exp: i32) -> String {
        format!(
            r#"
; {}^{}
MOVI R0, 1         ; R0 = result = 1
MOVI R1, {base}    ; R1 = base
MOVI R2, {exp}     ; R2 = exponent
MOVI R3, 0         ; R3 = counter = 0
loop:
CMP R2, R3         ; Compare exp with counter
JE end             ; If counter == exp, done
MUL R0, R1         ; result *= base
INC R3             ; counter++
JMP loop           ; Repeat
end:
HALT               ; R0 contains result
"#,
            base, exp
        )
    }

    fn generate_fibonacci(&self, n: i32) -> String {
        format!(
            r#"
; Fibonacci({})
MOVI R0, 0         ; F(0) = 0
MOVI R1, 1         ; F(1) = 1
MOVI R2, {n}       ; n
MOVI R3, 2         ; counter = 2
CMP R2, 0          ; if n == 0
JE result0
CMP R2, 1          ; if n == 1
JE result1
loop:
CMP R3, R2         ; Compare counter with n
JG end             ; If counter > n, done
MOV R4, R0         ; temp = F(n-2)
ADD R0, R1         ; F(n) = F(n-2) + F(n-1)
MOV R1, R4         ; F(n-1) = old F(n-2)
INC R3             ; counter++
JMP loop           ; Repeat
result0:
HALT               ; R0 = 0
result1:
MOV R0, R1         ; R0 = 1
HALT
end:
HALT               ; R0 contains result
"#,
            n
        )
    }

    fn generate_simple_loop(&self, n: i32) -> String {
        format!(
            r#"
; Loop {} times
MOVI R0, {n}       ; R0 = counter
loop:
CMP R0, 0          ; Compare counter with 0
JE end             ; If counter == 0, exit
DEC R0             ; counter--
; Loop body here
JMP loop           ; Repeat
end:
HALT
"#,
            n
        )
    }

    fn regex(text: &str, pattern: &str) -> Option<regex::Captures<'static>> {
        Regex::new(pattern).ok()?.captures(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_addition() {
        let mut compiler = MarkdownCompiler::new();
        let bytecode = compiler.compile("compute 3 + 4");
        assert!(bytecode.is_ok());
    }

    #[test]
    fn test_factorial() {
        let mut compiler = MarkdownCompiler::new();
        let bytecode = compiler.compile("factorial of 5");
        assert!(bytecode.is_ok());
    }

    #[test]
    fn test_sum() {
        let mut compiler = MarkdownCompiler::new();
        let bytecode = compiler.compile("sum 1 to 100");
        assert!(bytecode.is_ok());
    }

    #[test]
    fn test_direct_assembly() {
        let mut compiler = MarkdownCompiler::new();
        let bytecode = compiler.compile("MOVI R0, 42\nHALT");
        assert!(bytecode.is_ok());
    }
}
