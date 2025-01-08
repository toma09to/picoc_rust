use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::cmp;
use crate::opcode::Opcode;
use crate::decode::*;
use crate::error::Error;

pub const VM_INST_MEMORY_SIZE: usize = 10000;
pub const VM_STACK_SIZE: usize = 10000;

/// An instance of picoc vm.
///
/// `PicocVm` runs codes written in picoc vm instruction sets.
/// See [`Opcode`] for more about picoc vm instruction sets.
///
/// # Example
///
/// ```
/// use std::io::{self, BufReader};
/// use std::fs::File;
/// use picoc_vm::{PicocVm, Error};
///
/// fn main() -> Result<(), Error> {
///     let mut input = io::stdin().lock();
///     let mut output = io::stdout();
///
///     let mut vm = PicocVm::new(&mut input, &mut output);
///
///     let f = File::open("test.out")?;
///     let code = BufReader::new(f);
///
///     vm.load(code)?;
///
///     vm.run_until_halt()?;
///
///     Ok(())
/// }
/// ```
pub struct PicocVm<'a, T: BufRead, U: Write> {
    inst_memory: Vec<Opcode>,
    stack: Vec<i32>,
    label_table: HashMap<String, usize>,
    reg: Registers,
    is_halted: bool,
    input: &'a mut T,
    output: &'a mut U,
}

/// Registers for a VM.
pub struct Registers {
    /// Program Counter
    ///
    /// PC points to a next instruction.
    pub pc: usize,
    /// Stack Pointer 
    ///
    /// SP points to the top of a stack.
    pub sp: usize,
    /// Frame Pointer
    ///
    /// FP points to the top of a stack frame.
    pub fp: usize,
}

impl<'a, T, U> PicocVm<'a, T, U>
where 
    T: BufRead,
    U: Write,
{
    /// Creates a new VM.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::Cursor;
    /// use picoc_vm::PicocVm;
    ///
    /// fn main() {
    ///     let mut input = Cursor::new(b"10\n");
    ///     let mut output = Cursor::new(Vec::new());
    ///
    ///     let mut vm = PicocVm::new(&mut input, &mut output);
    /// }
    /// ```
    pub fn new(input: &'a mut T, output: &'a mut U) -> Self {
        let stack = vec![0; VM_STACK_SIZE];
        let reg = Registers {
            pc: 0,
            sp: VM_STACK_SIZE,
            fp: VM_STACK_SIZE,
        };

        Self {
            inst_memory: Vec::with_capacity(VM_INST_MEMORY_SIZE),
            stack,
            label_table: HashMap::new(),
            reg,
            is_halted: false,
            input,
            output,
        }
    }

    fn push(&mut self, data: i32) -> Result<(), Error> {
        if self.is_halted {
            return Err(Error::VmHalted);
        }

        self.reg.sp -= 1;

        if self.reg.sp >= VM_STACK_SIZE {
            return Err(Error::StackOverflow)
        }
        self.stack[self.reg.sp] = data;

        Ok(())
    }

    fn pop(&mut self) -> Result<i32, Error> {
        if self.is_halted {
            return Err(Error::VmHalted);
        }

        if self.reg.sp >= VM_STACK_SIZE {
            return Err(Error::StackUnderflow);
        }

        let ret = self.stack[self.reg.sp];
        self.reg.sp += 1;

        Ok(ret)
    }

    /// Loads a code into the VM from a stream.
    ///
    /// This method also initializes the VM's registers, which are PC, SP, and FP.
    ///
    /// # Errors
    ///
    /// This method returns [`Err`] if an invalid opcode or operand is found,
    /// of any I/O error occurs.
    /// See [`Error`] for details.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::{self, Cursor};
    /// use picoc_vm::{PicocVm, Error};
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut input = io::stdin().lock();
    ///     let mut output = io::stdout();
    ///
    ///     let mut vm = PicocVm::new(&mut input, &mut output);
    ///
    ///     let code = Cursor::new(b"
    ///         pushi 5
    ///         pushi 6
    ///         add
    ///         wr
    ///         wrln
    ///         halt");
    ///
    ///     vm.load(code)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn load<V: BufRead>(&mut self, inst: V) -> Result<(), Error> {
        let lines = split_code(inst)?;

        load_label(&lines, &mut self.label_table); // 1st pass
        load_inst(&lines, &mut self.inst_memory)?; // 2nd pass

        self.reg.pc = 0;
        self.reg.sp = VM_STACK_SIZE;
        self.reg.fp = VM_STACK_SIZE;
        self.is_halted = false;

        Ok(())
    }

    /// Executes once the instruction that PC points to and (mostly) increments PC.
    ///
    /// # Errors
    ///
    /// This method returns [`Err`] if a value of PC or SP is out of bounds,
    /// or an unknown label is found.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::{self, Cursor};
    /// use picoc_vm::{PicocVm, Error};
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut input = io::stdin().lock();
    ///     let mut output = io::stdout();
    ///
    ///     let mut vm = PicocVm::new(&mut input, &mut output);
    ///
    ///     let code = Cursor::new(b"
    ///         pushi 5
    ///         pushi 3
    ///         add
    ///         pushi 4
    ///         pushi 2
    ///         sub
    ///         div
    ///         wr
    ///         wrln
    ///         halt");
    ///
    ///     vm.load(code)?;
    ///
    ///     let mut count = 0;
    ///
    ///     while let Ok(()) = vm.step() {
    ///         count += 1;
    ///     }
    ///
    ///     println!("execution count: {count}");
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn step(&mut self) -> Result<(), Error> {
        if self.is_halted {
            return Err(Error::VmHalted);
        }

        if self.reg.pc >= self.inst_memory.len() {
            return Err(Error::MemoryOutOfBound);
        }

        match &self.inst_memory[self.reg.pc] {
            Opcode::Pushl(n) => {
                let target = self.reg.fp as i32 + n;
                if target < 0 || target >= VM_STACK_SIZE.try_into().unwrap() {
                    return Err(Error::StackOutOfBound);
                }

                let elem = self.stack[target as usize];
                self.push(elem)?;

                self.reg.pc += 1;
            },
            Opcode::Storel(n) => {
                let target = self.reg.fp as i32 + n;
                if target < 0 || target >= VM_STACK_SIZE.try_into().unwrap() {
                    return Err(Error::StackOutOfBound);
                }

                self.stack[target as usize] = self.stack[self.reg.sp];

                self.reg.pc += 1;
            },
            Opcode::Storet(n) => {
                let target = self.reg.sp as i32 + n;
                if target < 0 || target >= VM_STACK_SIZE.try_into().unwrap() {
                    return Err(Error::StackOutOfBound);
                }

                self.stack[target as usize] = self.stack[self.reg.sp];

                self.reg.pc += 1;
            },
            Opcode::Pushi(d) => {
                self.push(*d)?;

                self.reg.pc += 1;
            },
            Opcode::Call(label) => {
                let previous_pc = self.reg.pc as i32;
                if let Some(target) = self.label_table.get(label) {
                    self.reg.pc = *target;
                }
                self.push(previous_pc + 1)?;
            },
            Opcode::Ret => {
                self.reg.pc = self.pop()? as usize;
            },
            Opcode::Enter => {
                self.push(self.reg.fp as i32)?;
                self.reg.fp = self.reg.sp;

                self.reg.pc += 1;
            },
            Opcode::Leave => {
                self.reg.sp = self.reg.fp;
                self.reg.fp = self.pop()? as usize;

                self.reg.pc += 1;
            },
            Opcode::Mvsp(n) => {
                self.reg.sp = (self.reg.sp as i32 + n) as usize;

                self.reg.pc += 1;
            },
            Opcode::Jp(label) => {
                if let Some(target) = self.label_table.get(label) {
                    self.reg.pc = *target;
                } else {
                    return Err(Error::LabelNotFound(label.clone()));
                }
            },
            Opcode::Jt(label) => {
                if let Some(target) = self.label_table.get(label) {
                    let num = *target;

                    if self.pop()? != 0 {
                        self.reg.pc = num;
                    } else {
                        self.reg.pc += 1;
                    }
                } else {
                    return Err(Error::LabelNotFound(label.clone()));
                }
            },
            Opcode::Jf(label) => {
                if let Some(target) = self.label_table.get(label) {
                    let num = *target;

                    if self.pop()? == 0 {
                        self.reg.pc = num;
                    } else {
                        self.reg.pc += 1;
                    }
                } else {
                    return Err(Error::LabelNotFound(label.clone()));
                }
            },
            Opcode::Add => {
                let t1 = self.pop()?;
                let t2 = self.pop()?;

                self.push(t2 + t1)?;

                self.reg.pc += 1;
            },
            Opcode::Sub => {
                let t1 = self.pop()?;
                let t2 = self.pop()?;

                self.push(t2 - t1)?;

                self.reg.pc += 1;
            },
            Opcode::Mul => {
                let t1 = self.pop()?;
                let t2 = self.pop()?;

                self.push(t2 * t1)?;

                self.reg.pc += 1;
            },
            Opcode::Div => {
                let t1 = self.pop()?;
                let t2 = self.pop()?;

                self.push(t2 / t1)?;

                self.reg.pc += 1;
            },
            Opcode::Mod => {
                let t1 = self.pop()?;
                let t2 = self.pop()?;

                self.push(t2 % t1)?;

                self.reg.pc += 1;
            },
            Opcode::Eq => {
                let t1 = self.pop()?;
                let t2 = self.pop()?;

                self.push(if t2 == t1 { 1 } else { 0 })?;

                self.reg.pc += 1;
            },
            Opcode::Ne => {
                let t1 = self.pop()?;
                let t2 = self.pop()?;

                self.push(if t2 != t1 { 1 } else { 0 })?;

                self.reg.pc += 1;
            },
            Opcode::Gt => {
                let t1 = self.pop()?;
                let t2 = self.pop()?;

                self.push(if t2 > t1 { 1 } else { 0 })?;

                self.reg.pc += 1;
            },
            Opcode::Ge => {
                let t1 = self.pop()?;
                let t2 = self.pop()?;

                self.push(if t2 >= t1 { 1 } else { 0 })?;

                self.reg.pc += 1;
            },
            Opcode::Lt => {
                let t1 = self.pop()?;
                let t2 = self.pop()?;

                self.push(if t2 < t1 { 1 } else { 0 })?;

                self.reg.pc += 1;
            },
            Opcode::Le => {
                let t1 = self.pop()?;
                let t2 = self.pop()?;

                self.push(if t2 <= t1 { 1 } else { 0 })?;

                self.reg.pc += 1;
            },
            Opcode::Rd => {
                let mut line = String::new();

                self.output.write(b"? ")?;
                self.output.flush()?;
                self.input.read_line(&mut line)?;
                self.push(line.trim().parse()?)?;

                self.reg.pc += 1;
            },
            Opcode::Wr => {
                let content = self.pop()?.to_string() + " ";

                self.output.write(content.as_bytes())?;

                self.reg.pc += 1;
            },
            Opcode::Wrln => {
                self.output.write(b"\n")?;

                self.reg.pc += 1;
            },
            Opcode::Halt => {
                self.is_halted = true;
            },
        }

        self.reg.pc %= VM_INST_MEMORY_SIZE;

        Ok(())
    }

    /// Runs the code until VM halts or PC exceeds the length of the instruction memory.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] under the same situations as [`step`](PicocVm::step()).
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::{self, BufReader};
    /// use std::fs::File;
    /// use picoc_vm::{PicocVm, Error};
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut input = io::stdin().lock();
    ///     let mut output = io::stdout();
    ///
    ///     let mut vm = PicocVm::new(&mut input, &mut output);
    ///
    ///     let f = File::open("test.out")?;
    ///     let reader = BufReader::new(f);
    ///
    ///     vm.load(reader)?;
    ///
    ///     vm.run_until_halt()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn run_until_halt(&mut self) -> Result<(), Error> {
        loop {
            match self.step() {
                Ok(()) => (),
                Err(Error::VmHalted) => break,
                Err(Error::MemoryOutOfBound) => break,
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }

    /// Gets a reference to the instruction memory of the VM.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::Cursor;
    /// use picoc_vm::{PicocVm, Error, Opcode};
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut input = Cursor::new(b"");
    ///     let mut output = Cursor::new(Vec::new());
    ///
    ///     let mut vm = PicocVm::new(&mut input, &mut output);
    ///
    ///     let code = Cursor::new(b"pushi 5\nhalt\n");
    ///
    ///     vm.load(code)?;
    ///
    ///     assert_eq!(vm.inst_memory(), &[Opcode::Pushi(5), Opcode::Halt]);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn inst_memory(&self) -> &[Opcode] {
        &self.inst_memory[..]
    }

    /// Gets a reference to the label table of the VM.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::Cursor;
    /// use picoc_vm::{PicocVm, Error};
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut input = Cursor::new(b"10\n20\n");
    ///     let mut output = Cursor::new(Vec::new());
    ///
    ///     let mut vm = PicocVm::new(&mut input, &mut output);
    ///
    ///     let code = Cursor::new(b"
    ///             pushi 2
    ///         .L0:
    ///             rd
    ///             wr
    ///             pushl -1
    ///             pushi 1
    ///             sub
    ///             pushi 0
    ///             gt
    ///             jt .L0");
    ///
    ///     vm.load(code)?;
    ///
    ///     assert_eq!(vm.label_table().get(".L0"), Some(&1));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn label_table(&self) -> &HashMap<String, usize> {
        &self.label_table
    }

    /// Gets a reference to the stack of the VM.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::Cursor;
    /// use picoc_vm::{PicocVm, Error};
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut input = Cursor::new(b"");
    ///     let mut output = Cursor::new(Vec::new());
    ///
    ///     let mut vm = PicocVm::new(&mut input, &mut output);
    ///
    ///     let code = Cursor::new(b"
    ///         pushi 4
    ///         pushi 5
    ///         pushi 6
    ///         pushi 7
    ///         add");
    ///
    ///     vm.load(code)?;
    ///     vm.run_until_halt()?;
    ///
    ///     assert_eq!(vm.stack(), &[13, 5, 4]);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn stack(&self) -> &[i32] {
        let stack_bottom = cmp::min(self.reg.sp, self.reg.fp);
        return &self.stack[stack_bottom..VM_STACK_SIZE];
    }

    /// Gets a reference to the registers of the VM.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::Cursor;
    /// use picoc_vm::{PicocVm, Error};
    /// use picoc_vm::VM_STACK_SIZE;
    ///
    /// fn main() -> Result<(), Error> {
    ///     let mut input = Cursor::new(b"");
    ///     let mut output = Cursor::new(Vec::new());
    ///
    ///     let mut vm = PicocVm::new(&mut input, &mut output);
    ///
    ///     let code = Cursor::new(b"
    ///         __start__:
    ///             call main
    ///             halt
    ///         main:
    ///             enter
    ///             pushi 2147483647
    ///             leave
    ///             ret");
    ///
    ///     vm.load(code)?;
    ///
    ///     for _ in 0..3 {
    ///         println!("a");
    ///         vm.step()?;
    ///     }
    ///
    ///     let reg = vm.registers();
    ///     assert_eq!(reg.pc, 4);
    ///     assert_eq!(reg.sp, VM_STACK_SIZE - 3);
    ///     assert_eq!(reg.fp, VM_STACK_SIZE - 2);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn registers(&self) -> &Registers {
        &self.reg 
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{self, BufReader};

    #[test]
    fn load_assembly_code() -> Result<(), Error> {
        let mut input = io::stdin().lock();
        let mut output = io::stdout();

        let mut vm = PicocVm::new(&mut input, &mut output);

        let code = File::open("test.out")?;
        let code = BufReader::new(code);

        vm.load(code)?;

        assert_eq!(
            vm.inst_memory,
            vec![
                // __start__
                Opcode::Call("main".to_string()),
                Opcode::Halt,
                // read()
                Opcode::Enter,
                Opcode::Rd,
                Opcode::Storel(2),
                Opcode::Leave,
                Opcode::Ret,
                // write()
                Opcode::Enter,
                Opcode::Pushl(3),
                Opcode::Wr,
                Opcode::Leave,
                Opcode::Ret,
                // writeln()
                Opcode::Enter,
                Opcode::Wrln,
                Opcode::Leave,
                Opcode::Ret,
                // main()
                Opcode::Enter,
                Opcode::Pushi(1),
                Opcode::Mvsp(-1),
                Opcode::Call("write".to_string()),
                Opcode::Storet(1),
                Opcode::Mvsp(1),
                Opcode::Mvsp(1),
                Opcode::Mvsp(-1),
                Opcode::Call("writeln".to_string()),
                Opcode::Mvsp(1),
                Opcode::Leave,
                Opcode::Ret,
            ]
        );

        assert_eq!(
            vm.label_table,
            HashMap::from([
                ("__start__".to_string(), 0),
                ("read".to_string(), 2),
                ("write".to_string(), 7),
                ("writeln".to_string(), 12),
                ("main".to_string(), 16),
            ])
        );

        Ok(())
    }

    #[test]
    fn arithmetic_operations() -> Result<(), Error> {
        let mut input = io::stdin().lock();
        let mut output = io::stdout();

        let mut vm = PicocVm::new(&mut input, &mut output);

        // (3+2) * (8-2) / 5 = 6
        let code = io::Cursor::new(b"
            pushi 3
            pushi 2
            add
            pushi 8
            pushi 2
            sub
            mul
            pushi 5
            div
        ");

        vm.load(code)?;

        while let Ok(()) = vm.step() {}

        assert_eq!(vm.stack[VM_STACK_SIZE - 1], 6);

        Ok(())
    }

    #[test]
    fn comparison_operations() -> Result<(), Error> {
        let mut input = io::stdin().lock();
        let mut output = io::stdout();

        let mut vm = PicocVm::new(&mut input, &mut output);

        let code_eq = io::Cursor::new(b"
            pushi 10
            pushi -1
            eq
        ");
        let code_ne = io::Cursor::new(b"
            pushi 1
            pushi -1
            ne
        ");
        let code_gt = io::Cursor::new(b"
            pushi 4
            pushi 4
            gt
        ");
        let code_ge = io::Cursor::new(b"
            pushi 4
            pushi 4
            ge
        ");
        let code_lt = io::Cursor::new(b"
            pushi 10
            pushi -1
            lt
        ");
        let code_le = io::Cursor::new(b"
            pushi -1
            pushi 10
            le
        ");

        vm.load(code_eq)?;
        for _ in 0..3 {
            vm.step()?;
        }
        assert_eq!(vm.stack[VM_STACK_SIZE - 1], 0); // 10 == -1 is false
        vm.load(code_ne)?;
        for _ in 0..3 {
            vm.step()?;
        }
        assert_eq!(vm.stack[VM_STACK_SIZE - 1], 1); // 1 != -1 is true
        vm.load(code_gt)?;
        for _ in 0..3 {
            vm.step()?;
        }
        assert_eq!(vm.stack[VM_STACK_SIZE - 1], 0); // 4 > 4 is false
        vm.load(code_ge)?;
        for _ in 0..3 {
            vm.step()?;
        }
        assert_eq!(vm.stack[VM_STACK_SIZE - 1], 1); // 4 >= 4 is true
        vm.load(code_lt)?;
        for _ in 0..3 {
            vm.step()?;
        }
        assert_eq!(vm.stack[VM_STACK_SIZE - 1], 0); // 10 < -1 is false
        vm.load(code_le)?;
        for _ in 0..3 {
            vm.step()?;
        }
        assert_eq!(vm.stack[VM_STACK_SIZE - 1], 1); // -1 <= 10 is true

        Ok(())
    }

    #[test]
    fn io_operations() -> Result<(), Error> {
        let mut input = io::Cursor::new(b"-123\n");
        let mut output = io::Cursor::new(Vec::new());

        let mut vm = PicocVm::new(&mut input, &mut output);

        let code = io::Cursor::new(b"
            rd
            wr
            wrln
        ");

        vm.load(code)?;

        for _ in 0..3 {
            vm.step()?;
        }

        assert_eq!(output.get_ref(), b"? -123 \n");

        Ok(())
    }

    #[test]
    fn call_function() -> Result<(), Error> {
        let mut input = io::Cursor::new(b"10\n20\n");
        let mut output = io::Cursor::new(Vec::new());

        let mut vm = PicocVm::new(&mut input, &mut output);

        let code = io::Cursor::new(b"
            __start__:
                call main
                halt
            main:
                enter
                rd
                rd
                mvsp -1
                call add
                storet 2
                mvsp 2
                wr
                wrln
                leave
                ret
            add:
                enter
                pushl 4
                pushl 3
                add
                storel 2
                leave
                ret
        ");

        vm.load(code)?;

        while let Ok(()) = vm.step() {}

        assert_eq!(output.get_ref(), b"? ? 30 \n");

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Unknown opcode 'hoge' is found")]
    fn unknown_operation() {
        let mut input = io::stdin().lock();
        let mut output = io::stdout();

        let mut vm = PicocVm::new(&mut input, &mut output);

        let code = io::Cursor::new(b"hoge");

        vm.load(code).unwrap_or_else(|err| {
            panic!("{}", err.to_string());
        });
    }

    #[test]
    #[should_panic(expected = "Operand is not found")]
    fn operand_not_found() {
        let mut input = io::stdin().lock();
        let mut output = io::stdout();

        let mut vm = PicocVm::new(&mut input, &mut output);

        let code = io::Cursor::new(b"pushi");

        vm.load(code).unwrap_or_else(|err| {
            panic!("{}", err.to_string());
        });
    }
}
