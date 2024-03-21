use std::{fs, io::Result, process::{Command, Stdio}, thread, time::{Duration, SystemTime}};
use clap::{Parser, ValueHint};

/// Prevents the spread of commands if they ever get off the island
#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Input {
    /// Maximum age of command in seconds
    #[clap(short, long, default_value_t = 60)]
    max_age: u64,

    /// Use file modified timestamp for lysine check
    #[clap(value_hint = ValueHint::FilePath)]
    file: String,

    /// Command (and args) to run
    #[clap(required = true)]
    command: Vec<String>,
}

fn main() -> Result<()> {
    let args: Input = Input::parse();
    let expir_duration = Duration::from_secs(args.max_age);
    let sleep_duration = Duration::from_millis(std::cmp::max((expir_duration.as_millis() / 10).try_into().unwrap(), 100));
    let mut watchers: Vec<_> = Vec::new();
    let path = &args.file;

    let f = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}: {}", path, e);
            std::process::exit(e.raw_os_error().unwrap());
        }
    };

    watchers.push(move || {
        let metadata = match f.metadata() {
            Ok(metadata) => metadata,
            Err(e) => {
                eprintln!("{}: {}", path, e);
                std::process::exit(e.raw_os_error().unwrap());
            }
        };
        let mod_time = match metadata.modified() {
            Ok(mod_time) => mod_time,
            Err(e) => {
                eprintln!("{}: {}", path, e);
                std::process::exit(e.raw_os_error().unwrap());
            }
        };

        let time_since_modified = match SystemTime::now().duration_since(mod_time) {
            Ok(time_since_modified) => time_since_modified,
            Err(e) => e.duration()
        };

        time_since_modified < expir_duration
    });

    let mut child = Command::new(args.command.first().unwrap())
        .args(&args.command[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    'outer: loop {

        for watcher in &watchers {
            if watcher() == false {
                break 'outer;
            }
        }

        thread::sleep(sleep_duration)
    }

    let _ = child.kill();

    Ok(())
}
