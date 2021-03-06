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
#[macro_use]
extern crate log;

use getopts::Options;
use log::{info, LevelFilter};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::{env, fs, io, process};

use helianto::{Compiler, Error, Result, Settings};

const SETTINGS_FILE: &str = "helianto.toml";

const DEFAULT_FILES: &[(&str, &[u8])] = &[
    (
        "css/normalize.css",
        include_bytes!["../example_data/css/normalize.css"] as &'static [u8],
    ),
    (
        "css/skeleton.css",
        include_bytes!["../example_data/css/skeleton.css"],
    ),
    (
        "css/custom.css",
        include_bytes!["../example_data/css/custom.css"],
    ),
    (
        "_layouts/head.html.hbs",
        include_bytes!["templates/head.html.hbs"],
    ),
    (
        "_layouts/foot.html.hbs",
        include_bytes!["templates/foot.html.hbs"],
    ),
    (
        "_layouts/page.html.hbs",
        include_bytes!["templates/page.html.hbs"],
    ),
    (
        "welcome.markdown",
        include_bytes!["../example_data/example.markdown"],
    ),
    (
        "helianto.toml",
        include_bytes!["../example_data/helianto.toml"],
    ),
];

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] [SRC [DEST]]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("s", "settings", "use an alternate config file", "FILE");
    opts.optflag(
        "i",
        "init",
        "populate the source directory with default content",
    );
    opts.optflag("h", "help", "display this help and exit");
    opts.optflag("V", "version", "output version information and exit");
    opts.optflag("q", "quiet", "only display error messages");

    if !cfg!(ndebug) {
        opts.optflag("D", "debug", "display debug information");
    }

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            let _ = writeln!(&mut io::stderr(), "{}", f.to_string());
            process::exit(1);
        }
    };

    if matches.opt_present("help") {
        return print_usage(&program, opts);
    } else if matches.opt_present("version") {
        return print_version();
    }

    pretty_env_logger::formatted_builder()
        .filter(
            Some("helianto"),
            if matches.opt_present("quiet") {
                LevelFilter::Error
            } else if matches.opt_present("debug") {
                LevelFilter::Trace
            } else {
                LevelFilter::Info
            },
        )
        .init();

    if matches.free.len() > 2 || (matches.opt_present("init") && matches.free.len() > 1) {
        error!("Invalid number of arguments");
        process::exit(1);
    }

    let source_dir = if !matches.free.is_empty() {
        Some(PathBuf::from(matches.free[0].clone()))
    } else {
        None
    };

    let output_dir = if matches.free.len() > 1 {
        Some(PathBuf::from(matches.free[1].clone()))
    } else {
        None
    };

    let settings_file = matches.opt_str("settings").map(PathBuf::from);
    let working_directory = source_dir.clone().unwrap_or_else(|| PathBuf::from("."));

    let mut settings = match read_settings(&working_directory, settings_file.as_ref()) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    debug!("settings {:?}", settings);

    if let Some(ref path) = source_dir {
        settings.source_dir = path.clone();
    }

    if let Some(ref path) = output_dir {
        settings.output_dir = path.clone();
    }

    if matches.opt_present("init") {
        if matches.opt_present("settings") {
            error!("Option \"--settings\" can't be used with \"--init\".");
            process::exit(1);
        }
        return init_content(source_dir.as_ref());
    }

    Compiler::new(&settings).run().unwrap_or_else(|err| {
        error!("Compilation failed: {}", err);
        process::exit(2)
    });
}

fn read_settings<P: AsRef<Path>>(
    cwd: &P,
    alternate_file: Option<&P>,
) -> helianto::Result<Settings> {
    let default_file = cwd.as_ref().join(SETTINGS_FILE);
    let settings_file = alternate_file.map(|p| p.as_ref()).unwrap_or(&default_file);

    if is_file(&settings_file) {
        info!("Loading settings from {}.", settings_file.display());
        Settings::from_file(&settings_file)
    } else {
        Ok(Settings::with_working_directory(cwd.as_ref()))
    }
}

fn init_content<P: AsRef<Path>>(source_dir: Option<&P>) {
    let source_dir: PathBuf = if let Some(path) = source_dir {
        path.as_ref().into()
    } else {
        PathBuf::from(".")
    };

    match unpack_files(DEFAULT_FILES, &source_dir) {
        Ok(_) => process::exit(0),
        Err(e) => {
            error!("{}", e);
            process::exit(2);
        }
    }
}

fn print_version() {
    println!("helianto v{}", env!("CARGO_PKG_VERSION"));
}

fn unpack_files<P: AsRef<Path>>(files: &[(&str, &[u8])], dest: &P) -> Result<()> {
    let dest: &Path = dest.as_ref();
    if files.is_empty() {
        Ok(())
    } else {
        let current_file = files[0];
        let dest_file = dest.join(current_file.0);

        if is_file(&dest_file) {
            info!("Skipping {} : the file already exists", dest_file.display());
        } else {
            let parent_dir = dest_file.parent().ok_or(Error::Settings {
                    message: format!("\"{}\" is not a valid directory", dest.display()),
                })?;

            debug!("Creating directory {} …", parent_dir.display());
            fs::create_dir_all(&parent_dir)?;

            info!("Creating {} …", current_file.0);
            let mut fh = fs::File::create(&dest_file)?;
            fh.write(current_file.1)?;
        }

        unpack_files(&files[1..], &dest)
    }
}

fn is_file(path: &Path) -> bool {
    fs::metadata(path).map(|m| m.is_file()).unwrap_or(false)
}
