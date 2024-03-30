use std::{io::Result, path::PathBuf, thread, time::Duration};
use clap::{Parser, ValueHint};
use waiters::{File, Stdin};

mod waiters;

const SLEEP_DURATION: Duration = Duration::from_millis(500);

/// Prevents the spread of commands if they ever get off the island
#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Input {
    /// Maximum age of command in seconds
    #[clap(short, long, default_value_t = 60)]
    max_age: u64,

    /// Use file modified timestamp for lysine check
    #[clap(value_hint = ValueHint::FilePath)]
    file: PathBuf,

    /// Initialization grace time in seconds
    #[clap(short, long, default_value_t = 0)]
    grace_time: u64,

    /// Command (and args) to run
    #[clap(required = true)]
    command: Vec<String>,
}

fn main() -> Result<()> {
    let args: Input = Input::parse();
    let expir_duration = Duration::from_secs(args.max_age);

    let mut contingency = if args.file.as_os_str() == "-" {
        Stdin::new(args.command)
    } else {
        File::new(&args.file.as_path(), args.command)
    };

    contingency.start();

    if args.grace_time > 0 {
        thread::sleep(Duration::from_secs(args.grace_time));
    }
    
    loop {
        match contingency.last_dose() {
            Some(dur) => if dur > expir_duration { break; },
            None => break,
        };
        
        thread::sleep(SLEEP_DURATION);
    }

    let _ = contingency.kill();

    Ok(())
}