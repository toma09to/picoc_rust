use std::string::ToString;
use crate::error::Error;

/// Opcode of picoc vm instruction sets.
///
/// picoc vm is a stack machine. Thus, it has internal operations: `push` and `pop`.
///
/// `push` and `pop` are performed like the below C code:
/// ```c
/// void push(int d) {
///     sp--;
///     *sp = d;
/// }
///
/// int pop(void) {
///     int t;
///     t = *sp;
///     sp++;
///     return t;
/// }
/// ```
#[derive(Debug, PartialEq)]
pub enum Opcode {
    /// Pushes a value of a local variable
    /// # Assembly
    /// ```asm
    /// pushl n
    /// ```
    /// # Actions
    /// ```c
    /// push(*(fp + n));
    /// ```
    Pushl(i32),
    /// Stores a value peeked from a stack on a local variable.
    /// # Assembly
    /// ```asm
    /// storel n
    /// ```
    /// # Actions
    /// ```c
    /// *(fp + n) = *sp;
    /// ```
    Storel(i32),
    /// Stores a value peeked from a stack on a temporary area.
    /// # Assembly
    /// ```asm
    /// storet n
    /// ```
    /// # Actions
    /// ```c
    /// *(sp + n) = *sp;
    /// ```
    Storet(i32),
    /// Pushes a immediate value.
    /// # Assembly
    /// ```asm
    /// pushi d
    /// ```
    /// # Actions
    /// ```c
    /// push(d);
    /// ```
    Pushi(i32),
    /// Calls a function.
    /// # Assembly
    /// ```asm
    /// call label
    /// ```
    /// # Actions
    /// ```c
    /// push(pc + 1);
    /// pc = label;
    /// ```
    Call(String),
    /// Returns from a function called.
    /// # Assembly
    /// ```asm
    /// ret
    /// ```
    /// # Actions
    /// ```c
    /// pc = pop();
    /// ```
    Ret,
    /// Generates a stack frame.
    /// # Assembly
    /// ```asm
    /// enter
    /// ```
    /// # Actions
    /// ```c
    /// push(fp);
    /// fp = sp;
    /// ```
    Enter,
    /// Releases a stack frame.
    /// # Assembly
    /// ```asm
    /// leave
    /// ```
    /// # Actions
    /// ```c
    /// sp = fp;
    /// fp = pop();
    /// ```
    Leave,
    /// Moves a value of SP.
    /// # Assembly
    /// ```asm
    /// mvsp n
    /// ```
    /// # Actions
    /// ```c
    /// sp = sp + n;
    /// ```
    Mvsp(i32),
    /// Jumps to an instruction where the label is located on unconditionally.
    /// # Assembly
    /// ```asm
    /// jp label
    /// ```
    /// # Actions
    /// ```c
    /// pc = label;
    /// ```
    Jp(String),
    /// Jumps to an instruction where the label is located on if a value popped is true.
    /// # Assembly
    /// ```asm
    /// jt label
    /// ```
    /// # Actions
    /// ```c
    /// if (pop() != 0) {
    ///     pc = label;
    /// }
    /// ```
    Jt(String),
    /// Jumps to an instruction where the label is located on if a value popped is false.
    /// # Assembly
    /// ```asm
    /// jf label
    /// ```
    /// # Actions
    /// ```c
    /// if (pop() == 0) {
    ///     pc = label;
    /// }
    /// ```
    Jf(String),
    /// Performs addition.
    /// # Assembly
    /// ```asm
    /// add
    /// ```
    /// # Actions
    /// ```c
    /// t1 = pop();
    /// t2 = pop();
    /// push(t2 + t1);
    /// ```
    Add,
    /// Performs subtraction.
    /// # Assembly
    /// ```asm
    /// sub
    /// ```
    /// # Actions
    /// ```c
    /// t1 = pop();
    /// t2 = pop();
    /// push(t2 - t1);
    /// ```
    Sub,
    /// Performs multiplication.
    /// # Assembly
    /// ```asm
    /// mul
    /// ```
    /// # Actions
    /// ```c
    /// t1 = pop();
    /// t2 = pop();
    /// push(t2 * t1);
    /// ```
    Mul,
    /// Performs division.
    /// # Assembly
    /// ```asm
    /// div
    /// ```
    /// # Actions
    /// ```c
    /// t1 = pop();
    /// t2 = pop();
    /// push(t2 / t1);
    /// ```
    Div,
    /// Performs modulo.
    /// # Assembly
    /// ```asm
    /// mod
    /// ```
    /// # Actions
    /// ```c
    /// t1 = pop();
    /// t2 = pop();
    /// push(t2 % t1);
    /// ```
    Mod,
    /// Returns whether two popped values are the same.
    /// # Assembly
    /// ```asm
    /// eq
    /// ```
    /// # Actions
    /// ```c
    /// t1 = pop();
    /// t2 = pop();
    /// if (t2 == t1) {
    ///     push(1);
    /// } else {
    ///     push(0);
    /// }
    /// ```
    Eq,
    /// Returns whether two popped values are not the same.
    /// # Assembly
    /// ```asm
    /// ne
    /// ```
    /// # Actions
    /// ```c
    /// t1 = pop();
    /// t2 = pop();
    /// if (t2 != t1) {
    ///     push(1);
    /// } else {
    ///     push(0);
    /// }
    /// ```
    Ne,
    /// Returns whether the first pushed value is greater than the second pushed value.
    /// # Assembly
    /// ```asm
    /// gt
    /// ```
    /// # Actions
    /// ```c
    /// t1 = pop();
    /// t2 = pop();
    /// if (t2 > t1) {
    ///     push(1);
    /// } else {
    ///     push(0);
    /// }
    /// ```
    Gt,
    /// Returns whether the first pushed value is greater than or equal to the second pushed value.
    /// # Assembly
    /// ```asm
    /// ge
    /// ```
    /// # Actions
    /// ```c
    /// t1 = pop();
    /// t2 = pop();
    /// if (t2 >= t1) {
    ///     push(1);
    /// } else {
    ///     push(0);
    /// }
    /// ```
    Ge,
    /// Returns whether the first pushed value is less than the second pushed value.
    /// # Assembly
    /// ```asm
    /// lt
    /// ```
    /// # Actions
    /// ```c
    /// t1 = pop();
    /// t2 = pop();
    /// if (t2 < t1) {
    ///     push(1);
    /// } else {
    ///     push(0);
    /// }
    /// ```
    Lt,
    /// Returns whether the first pushed value is less than or equal to the second pushed value.
    /// # Assembly
    /// ```asm
    /// le
    /// ```
    /// # Actions
    /// ```c
    /// t1 = pop();
    /// t2 = pop();
    /// if (t2 <= t1) {
    ///     push(1);
    /// } else {
    ///     push(0);
    /// }
    /// ```
    Le,
    /// Reads a value from an input.
    /// # Assembly
    /// ```asm
    /// rd
    /// ```
    /// # Actions
    /// ```c
    /// scanf("%d", &t);
    /// push(t);
    /// ```
    Rd,
    /// Writes a value popped to an output.
    /// # Assembly
    /// ```asm
    /// wr
    /// ```
    /// # Actions
    /// ```c
    /// t = pop();
    /// printf("%d ", t);
    /// ```
    Wr,
    /// Writes `LF`(`0xA`) to an output.
    /// # Assembly
    /// ```asm
    /// wrln
    /// ```
    /// # Actions
    /// ```c
    /// printf("\n");
    /// ```
    Wrln,
    /// Halts a VM.
    /// # Assembly
    /// ```asm
    /// halt
    /// ```
    Halt,
}

impl Opcode {
    /// Converts strings (e.g. `["pushi", "123"]`) into an instruction.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if an opcode and an operand required cannot be parsed.
    ///
    /// # Example
    ///
    /// ```
    /// use picoc_vm::Opcode;
    ///
    /// fn main() {
    ///     let line = vec!["pushi".to_string(), "123".to_string()];
    ///
    ///     let opcode = Opcode::from_line(&line).unwrap();
    ///
    ///     assert_eq!(opcode, Opcode::Pushi(123));
    /// }
    /// ```
    pub fn from_line(line: &Vec<String>) -> Result<Opcode, Error> {
        if line.len() < 1 {
            return Err(Error::OpcodeNotFound);
        }

        match line[0].to_lowercase().as_str() {
            "pushl" => {
                if let Some(num) = line.get(1) {
                    inst_with_i32("pushl", num.parse()?)
                } else {
                    Err(Error::OperandNotFound)
                }
            },
            "storel" => {
                if let Some(num) = line.get(1) {
                    inst_with_i32("storel", num.parse()?)
                } else {
                    Err(Error::OperandNotFound)
                }
            },
            "storet" => {
                if let Some(num) = line.get(1) {
                    inst_with_i32("storet", num.parse()?)
                } else {
                    Err(Error::OperandNotFound)
                }
            },
            "pushi" => {
                if let Some(num) = line.get(1) {
                    inst_with_i32("pushi", num.parse()?)
                } else {
                    Err(Error::OperandNotFound)
                }
            },
            "call" => {
                if let Some(label) = line.get(1) {
                    inst_with_string("call", label.to_string())
                } else {
                    Err(Error::OperandNotFound)
                }
            },
            "ret" => {
                Ok(Opcode::Ret)
            },
            "enter" => {
                Ok(Opcode::Enter)
            },
            "leave" => {
                Ok(Opcode::Leave)
            },
            "mvsp" => {
                if let Some(num) = line.get(1) {
                    inst_with_i32("mvsp", num.parse()?)
                } else {
                    Err(Error::OperandNotFound)
                }
            },
            "jp" => {
                if let Some(label) = line.get(1) {
                    inst_with_string("jp", label.to_string())
                } else {
                    Err(Error::OperandNotFound)
                }
            },
            "jt" => {
                if let Some(label) = line.get(1) {
                    inst_with_string("jt", label.to_string())
                } else {
                    Err(Error::OperandNotFound)
                }
            },
            "jf" => {
                if let Some(label) = line.get(1) {
                    inst_with_string("jf", label.to_string())
                } else {
                    Err(Error::OperandNotFound)
                }
            },
            "add" => {
                Ok(Opcode::Add)
            },
            "sub" => {
                Ok(Opcode::Sub)
            },
            "mul" => {
                Ok(Opcode::Mul)
            },
            "div" => {
                Ok(Opcode::Div)
            },
            "mod" => {
                Ok(Opcode::Mod)
            },
            "eq" => {
                Ok(Opcode::Eq)
            },
            "ne" => {
                Ok(Opcode::Ne)
            },
            "gt" => {
                Ok(Opcode::Gt)
            },
            "ge" => {
                Ok(Opcode::Ge)
            },
            "lt" => {
                Ok(Opcode::Lt)
            },
            "le" => {
                Ok(Opcode::Le)
            },
            "rd" => {
                Ok(Opcode::Rd)
            },
            "wr" => {
                Ok(Opcode::Wr)
            },
            "wrln" => {
                Ok(Opcode::Wrln)
            },
            "halt" => {
                Ok(Opcode::Halt)
            },
            other => Err(Error::UnknownOpcode(other.to_string())),
        }
    }
}

fn inst_with_i32(op: &str, num: i32) -> Result<Opcode, Error> {
    match op {
        "pushl" => Ok(Opcode::Pushl(num)),
        "storel" => Ok(Opcode::Storel(num)),
        "storet" => Ok(Opcode::Storet(num)),
        "pushi" => Ok(Opcode::Pushi(num)),
        "mvsp" => Ok(Opcode::Mvsp(num)),
        other => Err(Error::UnknownOpcode(other.to_string())),
    }
}

fn inst_with_string(op: &str, str: String) -> Result<Opcode, Error> {
    match op {
        "call" => Ok(Opcode::Call(str)),
        "jp" => Ok(Opcode::Jp(str)),
        "jt" => Ok(Opcode::Jt(str)),
        "jf" => Ok(Opcode::Jf(str)),
        other => Err(Error::UnknownOpcode(other.to_string())),
    }
}

impl ToString for Opcode {
    fn to_string(&self) -> String {
        match self {
            Opcode::Pushl(n) => format!("pushl {}", n).to_string(),
            Opcode::Storel(n) => format!("storel {}", n).to_string(),
            Opcode::Storet(n) => format!("storet {}", n).to_string(),
            Opcode::Pushi(d) => format!("pushi {}", d).to_string(),
            Opcode::Call(label) => format!("call {}", label).to_string(),
            Opcode::Ret => format!("ret").to_string(),
            Opcode::Enter => format!("enter").to_string(),
            Opcode::Leave => format!("leave").to_string(),
            Opcode::Mvsp(n) => format!("mvsp {}", n).to_string(),
            Opcode::Jp(label) => format!("jp {}", label).to_string(),
            Opcode::Jt(label) => format!("jt {}", label).to_string(),
            Opcode::Jf(label) => format!("jf {}", label).to_string(),
            Opcode::Add => format!("add").to_string(),
            Opcode::Sub => format!("sub").to_string(),
            Opcode::Mul => format!("mul").to_string(),
            Opcode::Div => format!("div").to_string(),
            Opcode::Mod => format!("mod").to_string(),
            Opcode::Eq => format!("eq").to_string(),
            Opcode::Ne => format!("ne").to_string(),
            Opcode::Gt => format!("gt").to_string(),
            Opcode::Ge => format!("ge").to_string(),
            Opcode::Lt => format!("lt").to_string(),
            Opcode::Le => format!("le").to_string(),
            Opcode::Rd => format!("rd").to_string(),
            Opcode::Wr => format!("wr").to_string(),
            Opcode::Wrln => format!("wrln").to_string(),
            Opcode::Halt => format!("halt").to_string(),
        }
    }
}
