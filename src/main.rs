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
use std::{env, fs, process};

use helianto::{Compiler, Result, Settings};

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

fn main() -> Result<()> {
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

    let matches = opts.parse(&args[1..])?;

    if matches.opt_present("help") {
        print_usage(&program, opts);
        return Ok(());
    } else if matches.opt_present("version") {
        print_version();
        return Ok(());
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

    let working_directory = matches.free.get(0).map_or(".", |s| s.as_str());

    let settings_file = matches.opt_str("settings").map_or_else(
        || (working_directory.as_ref() as &Path).join(SETTINGS_FILE),
        |s| PathBuf::from(&s),
    );

    let mut settings = Settings::from_file(&settings_file)?;

    if let Some(ref path) = matches.free.get(1) {
        settings.output_dir = PathBuf::from(path);
    }

    if matches.opt_present("init") {
        if matches.opt_present("settings") {
            error!("Option \"--settings\" can't be used with \"--init\".");
            process::exit(1);
        }
        return init_content(working_directory);
    }

    Compiler::new(&settings).run()
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] [SRC [DEST]]", program);
    print!("{}", opts.usage(&brief));
}

fn print_version() {
    println!("helianto v{}", env!("CARGO_PKG_VERSION"));
}

fn init_content<P: AsRef<Path>>(dest_dir: P) -> Result<()> {
    let dest_dir = dest_dir.as_ref();
    for (dest_path_relative, file_content) in DEFAULT_FILES {
        let dest_path = dest_dir.join(dest_path_relative);

        if dest_path.exists() {
            info!("Skipping {} : the path already exists", dest_path.display());
            continue;
        }

        if let Some((parent_dir, false)) = dest_path.parent().map(|p| (p, p.exists())) {
            debug!("Creating directory {} …", parent_dir.display());
            fs::create_dir_all(&parent_dir)?;
        }

        info!("Creating {} …", dest_path.display());
        let mut fh = fs::File::create(&dest_path)?;
        fh.write_all(file_content)?;
    }

    Ok(())
}
