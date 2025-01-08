use std::io::{self, BufReader, BufRead, Write};
use std::fs::File;
use std::iter;
use getopts::Matches;
use picoc_vm::{PicocVm, Opcode};
use picoc_vm::VM_STACK_SIZE;

fn dump_inst_memory<T, U>(vm: &PicocVm<T, U>)
where
    T: BufRead,
    U: Write
{
    let imem = vm.inst_memory();
    let label_table = vm.label_table();

    for (i, inst) in iter::zip(0..imem.len(), imem) {
        eprint!("{:4}: {}", i, inst.to_string());
        match inst {
            Opcode::Call(l)
                | Opcode::Jp(l)
                | Opcode::Jt(l)
                | Opcode::Jf(l) => {
                    if let Some(num) = label_table.get(l) {
                        eprint!("({})", num);
                    }
                },
            _ => (),
        }
        eprintln!("");
    }
}

fn trace_stack<T, U>(vm: &PicocVm<T, U>)
where
    T: BufRead,
    U: Write,
{
    let stack = vm.stack();
    let reg = vm.registers();
    let bottom = VM_STACK_SIZE - stack.len();

    for (i, data) in iter::zip(bottom..VM_STACK_SIZE, stack).rev() {
        eprintln!(
            "{:04} {:11}{}{}",
            i,
            data,
            if i == reg.fp { " <-- FP" } else { "" },
            if i == reg.sp { " <-- SP" } else { "" },
        );
    }
    eprintln!("");
}

fn trace_registers<T, U>(vm: &PicocVm<T, U>)
where
    T: BufRead,
    U: Write,
{
    let reg = vm.registers();

    eprintln!("PC = {:05}, SP = {:05}, FP = {:05}", reg.pc, reg.sp, reg.fp);
}

pub fn run_vm(matches: Matches) -> Result<(), picoc_vm::Error> {
    let dump_imem = matches.opt_present("d");
    let trace_regs = matches.opt_present("r");
    let trace_stk = matches.opt_present("s");

    for file in matches.free {
        let mut input = io::stdin().lock();
        let mut output = io::stdout();

        let mut vm = PicocVm::new(&mut input, &mut output);

        let file = File::open(file)?;
        let code = BufReader::new(file);

        vm.load(code)?;

        if dump_imem {
            dump_inst_memory(&vm);
        }

        let mut result = Ok(());
        while result.is_ok() {
            if trace_stk {
                trace_stack(&vm);
            }
            if trace_regs {
                trace_registers(&vm);
            }
            result = vm.step();
        }

        match result {
            Ok(()) | Err(picoc_vm::Error::VmHalted) => (),
            Err(err) => return Err(err),
        }
    }

    Ok(())
}
