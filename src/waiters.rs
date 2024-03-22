use std::{
    ffi::OsString, io::{self, Read, Write}, path::Path, process::{Child, Command}, time::{Duration, SystemTime}
};

use os_pipe::PipeWriter;


pub trait Contingency {
    fn start(&mut self);
    fn last_dose(&mut self) -> Option<Duration>;
    fn kill(&mut self);
}

pub struct Stdin {
    child: Option<Child>,
    cmd: Vec<String>,
    child_stdin: Option<PipeWriter>,
    last_byte: Option<SystemTime>,
}

impl Stdin {
    pub fn new(cmd: Vec<String>) -> Box<dyn Contingency> {
        Box::new(Self {
            child: None,
            cmd,
            child_stdin: None,
            last_byte: None,
        })
    }
}

impl Contingency for Stdin {
    
    fn start(&mut self) {
        let (reader, writer) = os_pipe::pipe().unwrap();

        self.child = match Command::new(&self.cmd[0])
            .args(&self.cmd[1..])
            .stdin(reader)
            .spawn() {
                Ok(handle) => {
                    Some(handle)
                },
                Err(e) => {
                    eprint!("Unable to start child process: {}", e);
                    std::process::exit(e.raw_os_error().unwrap());
                }
            };

        self.child_stdin = Some(writer);
    }

    fn last_dose(&mut self) -> Option<Duration> {
        let mut stdin = io::stdin();
        let mut buf = [0u8; 1024];
        let writer = self.child_stdin.as_mut()?;

        let handle = self.child.as_mut()
            .expect("Contingency never put into effect");

        match handle.try_wait() {
            Ok(None) => (),
            Ok(Some(_)) | Err(_) => {
                // The child process has ended (successfully or otherwise)
                return None;
            },
        }

        loop {
            match stdin.read(&mut buf) {
                Ok(count) => {
                    match count {
                        0 => break,
                        _ => {
                            match writer.write_all(&buf) {
                                Ok(()) => (),
                                Err(_e) => { break; },
                            }
                            self.last_byte = Some(SystemTime::now());
                        },
                    }
                },
                Err(_) => break,
            }
        }

        match self.last_byte {
            None => None,
            Some(time) => Some(time.elapsed().unwrap())
        }
    }

    fn kill(&mut self) {
        if let Some(handle) = &mut self.child {
            let _ = handle.kill();
        } else {
            eprintln!("Cannot find child process to kill off.");
        }
    }
}


pub struct File {
    child: Option<Child>,
    cmd: Vec<String>,
    file_name: OsString,
    f: std::fs::File,
}

impl File {
    pub fn new(path: &Path, cmd:Vec<String>) -> Box<dyn Contingency> {
        let file_name = path.file_name().unwrap().to_owned();
        let f = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{:?}: {}", path, e);
                std::process::exit(e.raw_os_error().unwrap());
            }
        };

        Box::new(Self {
            child: None,
            cmd,
            file_name,
            f,
        })
    }
}

impl Contingency for File {

    fn start(&mut self) {
        self.child = match Command::new(&self.cmd[0])
            .args(&self.cmd[1..])
            .spawn() {
                Ok(handle) => Some(handle),
                Err(e) => {
                    eprint!("Unable to start child process: {}", e);
                    std::process::exit(e.raw_os_error().unwrap());
                }
            };
    }


    fn last_dose(&mut self) -> Option<Duration> {
        let handle = self.child.as_mut()
            .expect("Contingency never put into effect");

        match handle.try_wait() {
            Ok(None) => (),
            Ok(Some(_)) | Err(_) => {
                // The child process has ended (successfully or otherwise)
                return None;
            },
        }

        let metadata = match self.f.metadata() {
            Ok(metadata) => metadata,
            Err(e) => {
                eprintln!("{:?}: {}", self.file_name, e);
                std::process::exit(e.raw_os_error().unwrap());
            }
        };
        let mod_time = match metadata.modified() {
            Ok(mod_time) => mod_time,
            Err(e) => {
                eprintln!("{:?}, {}", self.file_name, e);
                std::process::exit(e.raw_os_error().unwrap());
            }
        };

        let time_since_modified = match SystemTime::now().duration_since(mod_time) {
            Ok(time_since_modified) => time_since_modified,
            Err(e) => e.duration()
        };

        Some(time_since_modified)
    }

    fn kill(&mut self) {
        if let Some(handle) = &mut self.child {
            let _ = handle.kill();
        } else {
            eprintln!("Cannot find child process to kill off.");
        }
    }
}