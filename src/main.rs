use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::env;
use std::fs;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

// Watches Pictures directory and renames any file found starting with
// "Screenshot from x" to "ss-x", also removing spaces.
fn main() {
    let dir = format!("{}/Pictures", env::var("HOME").unwrap());
    let dir = Path::new(&dir);
    scan(&dir);
    watch(&dir);
}

fn check(path: &Path) {
    let name = path.file_name().unwrap().to_string_lossy();
    if name.starts_with("Screenshot from ") {
        let new_name = format!(
            "{}/ss-{}",
            path.parent().unwrap().to_string_lossy(),
            name.chars()
                .skip(16)
                .filter(|&c| c != ' ')
                .collect::<String>()
        );
        let new_path = Path::new(&new_name);
        fs::rename(path, Path::new(new_path)).unwrap();
    }
}

fn scan(dir: &Path) {
    for entry in fs::read_dir(dir).unwrap() {
        check(&entry.unwrap().path());
    }
}

fn watch(dir: &Path) {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    watcher.watch(dir, RecursiveMode::NonRecursive).unwrap();
    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Create(path)) => check(&path),
            Ok(_) => {}
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }
}
