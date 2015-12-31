// Helianto -- static website generator
// Copyright © 2015-2016 Mickaël RAYBAUD-ROIG
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.


extern crate getopts;
extern crate helianto;
extern crate stdio_logger;
#[macro_use]
extern crate log;

use std::{env, fs, process, io};
use std::path::PathBuf;
use getopts::Options;
use std::path::Path;
use std::io::Write;
use log::{LogLevel};

use helianto::{Error, Result, Compiler, Settings};

const SETTINGS_FILE: &'static str = "helianto.toml";

const DEFAULT_FILES: &'static [(&'static str, &'static [u8])] = &[
    ("css/normalize.css",      include_bytes!["../example_data/css/normalize.css"] as &'static [u8]),
    ("css/skeleton.css",       include_bytes!["../example_data/css/skeleton.css"]),
    ("css/custom.css",         include_bytes!["../example_data/css/custom.css"]),
    ("_layouts/head.html.hbs", include_bytes!["templates/head.html.hbs"]),
    ("_layouts/foot.html.hbs", include_bytes!["templates/foot.html.hbs"]),
    ("_layouts/page.html.hbs", include_bytes!["templates/page.html.hbs"]),
    ("welcome.markdown",       include_bytes!["../example_data/example.markdown"]),
];


fn print_usage(program: &str, opts: Options) -> ! {
    let brief = format!("Usage: {} [options] [SRC [DEST]]", program);
    print!("{}", opts.usage(&brief));
    process::exit(0)
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("s",  "settings", "use an alternate config file", "FILE");
    opts.optflag("i", "init", "populate the source directory with default content");
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
            writeln!(&mut io::stderr(), "{}", f.to_string());
            return process::exit(1);
        }
    };

    if matches.opt_present("help") {
        return print_usage(&program, opts);
    } else if matches.opt_present("version") {
        return print_version();
    }

    if matches.free.len() > 2 || (matches.opt_present("init") && matches.free.len() > 1) {
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
            LogLevel::Error
        } else if matches.opt_present("debug") {
            LogLevel::Trace
        } else {
            LogLevel::Info
        }
    ).expect("Could not initialize logging");

    let settings_file = matches.opt_str("settings").map(PathBuf::from);

    let mut settings = match read_settings(source_dir.as_ref(), settings_file.as_ref()) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };

    if let Some(ref path) = source_dir {
        settings.source_dir = path.clone();
    }

    if let Some(ref path) = output_dir {
        settings.output_dir = path.clone();
    }

    
    if matches.opt_present("init") {
        if matches.opt_present("settings") {
            error!("Option \"--settings\" can't be used with \"--init\".");
            return process::exit(1);
        }
        return init_content(source_dir.as_ref());
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


fn init_content<P: AsRef<Path>>(source_dir: Option<&P>) -> ! {
    let source_dir: PathBuf = if let Some(path) = source_dir {
        path.as_ref().into()
    } else {
        PathBuf::from(".")
    };
    let settings_file = source_dir.join(SETTINGS_FILE);


    match unpack_files(DEFAULT_FILES, &source_dir) {
        Ok(_) => process::exit(0),
        Err(e) => {
            error!("{}", e);
            process::exit(2);
        }
    }
}


fn print_version() -> ! {
    println!("helianto v{}", env!("CARGO_PKG_VERSION"));
    process::exit(0);
}


fn unpack_files<P: AsRef<Path>>(files: &[(&str, &[u8])], dest: &P) -> Result<()> {
    let dest: &Path = dest.as_ref();
    if files.len() == 0 {
        Ok(())
    } else {
        let current_file = files[0];
        let dest_file = dest.join(current_file.0);
        let parent_dir = try! {
            dest_file.parent().ok_or(Error::Settings {
                message: format!("\"{}\" is not a valid directory", dest.display()),
            })
        };

        debug!("Creating directory {} …", parent_dir.display());
        try! { fs::create_dir_all(&parent_dir) }

        info!("Creating {} …", current_file.0);
        let mut fh = try! { fs::File::create(&dest_file) };
        try! { fh.write(current_file.1).map(void) }

        unpack_files(&files[1..], &dest)
    }
}


fn is_file(path: &Path) -> bool {
    fs::metadata(path).map(|m| m.is_file()).unwrap_or(false)
}

#[inline]
fn void<T: 'static>(_arg: T) -> () {
    ()
}
