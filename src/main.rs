use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::sync::mpsc::channel;
use std::time::Duration;

// Watches Pictures directory and renames any file found starting with
// "Screenshot from x" to "ss-x", also removing spaces.
fn main() {
    let dir = format!(
        "{}/Pictures",
        env::var("HOME").expect("HOME environment variable missing")
    );
    let dir = Path::new(&dir);

    let mut lock = Lock::new(&dir.join(".screenshot-renamer.lock"));
    if let Err(e) = lock.lock() {
        eprintln!("{:?}: {}", lock.path, e);
        std::process::exit(1);
    };
    match scan(&dir) {
        Ok(()) => match watch(&dir) {
            Ok(()) => unreachable!(),
            Err(e) => eprintln!("watch(): {}", e),
        },
        Err(e) => eprintln!("scan(): {}", e),
    }
    drop(lock);
}

fn check(path: &Path) -> Result<(), io::Error> {
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(ErrorKind::Other, format!("{:?}: No file name", path)))?
        .to_string_lossy();
    if name.starts_with("Screenshot from ") {
        let new_name = format!(
            "{}/ss-{}",
            path.parent()
                .ok_or_else(|| io::Error::new(ErrorKind::Other, format!("{:?}: No parent", path)))?
                .to_string_lossy(),
            name.chars()
                .skip(16)
                .filter(|&c| c != ' ')
                .collect::<String>()
        );
        let new_path = Path::new(&new_name);
        fs::rename(path, Path::new(new_path))?;
    }
    Ok(())
}

fn scan(dir: &Path) -> Result<(), io::Error> {
    for entry in fs::read_dir(dir)? {
        check(&entry?.path())?;
    }
    Ok(())
}

fn watch(dir: &Path) -> Result<(), Box<Error>> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1))?;
    watcher.watch(dir, RecursiveMode::NonRecursive)?;
    loop {
        match rx.recv()? {
            DebouncedEvent::Create(path) => check(&path)?,
            _ => {}
        }
    }
}

struct Lock {
    path: PathBuf,
    lock: Option<File>,
}

impl Lock {
    fn new(path: &Path) -> Lock {
        dbg!("Lock::new()");
        Lock {
            path: PathBuf::from(path),
            lock: None,
        }
    }

    fn lock(&mut self) -> Result<(), io::Error> {
        dbg!("Lock::lock()");
        let lock = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.path)
        {
            Ok(mut lock) => {
                write!(lock, "{}\n", process::id())?;
                lock
            }
            Err(e) => return Err(e),
        };
        self.lock = Some(lock);
        Ok(())
    }

    fn unlock(&mut self) -> Result<(), io::Error> {
        dbg!("Lock::unlock()");
        if self.lock.is_some() {
            drop(self.lock.take());
            fs::remove_file(&self.path)?
        }
        Ok(())
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        dbg!("Lock::drop()");
        match self.unlock() {
            Ok(()) => (),
            Err(e) => eprintln!("Error dropping lock: {}", e),
        }
    }
}
