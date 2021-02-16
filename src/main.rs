use saturn_patch::{SaturnDisc, DESIRED_SATURN_DISC};
use std::env;
use std::ffi::OsString;

use anyhow::Result;

fn main() {
    let mut args = env::args_os().peekable();

    // let's go by name of executable to decide what operation to run
    let mut patch = args.next().map_or(true, |exe| !exe.to_string_lossy().to_ascii_lowercase().contains("unpatch"));

    // but also support a single arg in the first position, if people wish
    if args.peek().map_or(false, |arg| arg.eq("-u") || arg.eq("--unpatch")) {
        args.next(); // just skip the next one
        patch = false;
    }

    let saturn_disc = if patch {
        match SaturnDisc::from_env_args() {
            Ok(sat_p) => sat_p,
            Err(err) => {
                eprintln!("ERROR: {}", err);
                std::process::exit(2);
            }
        }
    } else {
        DESIRED_SATURN_DISC
    };

    let patch: Box<dyn Fn(&OsString) -> Result<()>> = if patch { Box::new(|f| saturn_disc.patch(f)) } else { Box::new(|f| SaturnDisc::unpatch(f)) };

    let mut exit_code = 0;

    for file_name in args {
        if let Err(err) = patch(&file_name) {
            eprintln!("ERROR: {}", err);
            exit_code = 1;
        }
    }
    std::process::exit(exit_code);
}
