extern crate getopts;
extern crate helianto;

use std::env;
use std::path::PathBuf;
use getopts::Options;

use helianto::{Settings, Generator};


fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "display this help and exit");
    opts.optflag("V", "version", "output version information and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => {
            m
        }
        Err(f) => {
            panic!(f.to_string())
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    if matches.free.len() > 2 {
        panic!("Invalid number of arguments");
    }

    let mut settings = Settings::default();

    if matches.free.len() > 0 {
        settings.source_dir = PathBuf::from(matches.free[0].clone())
    }

    if matches.free.len() > 1 {
        settings.output_dir = PathBuf::from(matches.free[1].clone());
    }

    Generator::new(&settings).run().unwrap_or_else(|err| {
        println!("Compilation failed: {}", err);
    });
}
