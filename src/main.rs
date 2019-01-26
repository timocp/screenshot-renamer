use std::env;
use std::fs;
use std::path::Path;

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
        println!("rename({:?}, {:?})", path, new_path);
        fs::rename(path, Path::new(new_path)).unwrap();
    }
}

fn scan(dir: &Path) {
    println!("scan({:?})", dir);
    for entry in fs::read_dir(dir).unwrap() {
        check(&entry.unwrap().path());
    }
}

fn watch(dir: &Path) {
    println!("watch({:?})", dir);
}
