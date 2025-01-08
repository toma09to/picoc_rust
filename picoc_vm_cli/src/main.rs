use std::env;
use std::process;
use getopts::Options;

mod run;

use run::run_vm;

fn print_usage(program: &str, opts: Options, exit_code: i32) -> ! {
    let brief = format!("Usage: {} [OPTION] FILE...", program);
    print!("{}", opts.usage(&brief));
    process::exit(exit_code);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("d", "", "dump instruction memory");
    opts.optflag("r", "", "trace registers");
    opts.optflag("s", "", "trace stack");
    opts.optflag("h", "help", "print help and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(f) => { panic!("{}", f.to_string()) },
    };

    if matches.opt_present("h") {
        print_usage(&args[0], opts, 0);
    }

    if matches.free.len() == 0 {
        print_usage(&args[0], opts, 1);
    }

    match run_vm(matches) {
        Ok(()) => (),
        Err(err) => { panic!("{}", err.to_string()) },
    }
}
