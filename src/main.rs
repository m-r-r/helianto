extern crate getopts;
extern crate helianto;
extern crate stdio_logger;
#[macro_use]
extern crate log;

use std::{env, fs, process};
use std::path::PathBuf;
use getopts::Options;
use std::path::Path;
use log::{LogLevelFilter};

use helianto::{Compiler, Settings};

const SETTINGS_FILE: &'static str = "helianto.toml";


fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("s", "settings", "FILE", "use an alternate config file");
    opts.optflag("h", "help", "display this help and exit");
    opts.optflag("V", "version", "output version information and exit");
    opts.optflag("q", "quiet", "only display error messages");

    if ! cfg!(ndebug) {
        opts.optflag("D", "debug", "display debug information");
    }

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => {
            m
        }
        Err(f) => {
            panic!(f.to_string())
        }
    };

    if matches.opt_present("help") {
        print_usage(&program, opts);
        return;
    }

    if matches.free.len() > 2 {
        error!("Invalid number of arguments");
        return process::exit(1);
    }

    let source_dir = if matches.free.len() > 0 {
        Some(PathBuf::from(matches.free[0].clone()))
    } else {
        None
    };

    let output_dir = if matches.free.len() > 1 {
        Some(PathBuf::from(matches.free[1].clone()))
    } else {
        None
    };


    stdio_logger::init(
        if matches.opt_present("quiet") {
            LogLevelFilter::Error
        } else if matches.opt_present("debug") {
            LogLevelFilter::Trace
        } else {
            LogLevelFilter::Info
        }
    ).expect("Could not initialize logging");

    let settings_file = matches.opt_str("settings").map(PathBuf::from);

    let mut settings = match read_settings(source_dir.as_ref(), settings_file.as_ref()) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };

    if let Some(path) = source_dir {
        settings.source_dir = path;
    }

    if let Some(path) = output_dir {
        settings.output_dir = path;
    }

    Compiler::new(&settings).run().unwrap_or_else(|err| {
        error!("Compilation failed: {}", err);
        return process::exit(2);
    });
}

fn read_settings<P: AsRef<Path>>(cwd: Option<&P>,
                                 settings_file: Option<&P>)
                                 -> helianto::Result<Settings> {
    if let Some(ref path) = settings_file {
        info!("Loading settings from {}.", path.as_ref().display());
        return Settings::from_file(path);
    }
    let settings_file = cwd.map(|p| PathBuf::from(p.as_ref()))
                           .unwrap_or(PathBuf::from("."))
                           .join(SETTINGS_FILE);
    if is_file(&settings_file) {
        info!("Loading settings from {}.", settings_file.display());
        Settings::from_file(&settings_file)
    } else {
        Ok(Settings::default())
    }
}


fn is_file(path: &Path) -> bool {
    fs::metadata(path).map(|m| m.is_file()).unwrap_or(false)
}
