use rayon::prelude::*;
use std::{env, io::Result, path::Path};

fn dir_size(path: &Path) -> Result<u64> {
    // We don't count symlinks in disk usage calculations
    if path.is_symlink() {
        return Ok(0);
    };

    // Process the input path if it's a file.
    if path.is_file() {
        return Ok(path.metadata()?.len());
    }

    // Process the input if it's a directory
    let total = path
        .read_dir()?
        .par_bridge()
        .map(|entry| {
            let entry = entry.expect("Cannot get directory entry!");
            let ft = entry.file_type().expect("Cannot get file type!"); // cheap, no extra syscall on OS

            if ft.is_symlink() {
                return 0;
            }
            if ft.is_file() {
                return entry.metadata().expect("Cannot get file size!").len();
            } else {
                return dir_size(&entry.path()).expect(&format!(
                    "Cannot get size of directory: {:?}",
                    &entry.path().as_os_str()
                ));
            }
        })
        .sum();

    // Add default directory size to total size
    return Ok(total);
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    assert!(
        args.len() == 2,
        "Expected 2 arguments, but {} provided",
        args.iter().len()
    );

    let current_dir = env::current_dir().expect("Cannot get the current dir location!");

    let abs_path = match current_dir.join(&args[1]).canonicalize() {
        Ok(x) => x,
        Err(e) => return Err(e),
    };

    match (abs_path.is_dir(), abs_path.is_file()) {
        (true, false) => {
            for entry in abs_path.read_dir().expect("read_dir() call failed") {
                let entry = match entry {
                    Ok(x) => x,
                    Err(e) => return Err(e),
                };

                let name = entry.file_name().into_string().expect(&format!(
                    "Cannot get string of file name: {:?}",
                    entry.file_name()
                ));
                let size = match dir_size(&entry.path()) {
                    Err(e) => return Err(e),
                    Ok(x) => x,
                };

                println!("{}\t| {}", name, size);
            }
        }
        (false, true) => {
            let name = abs_path
                .file_name()
                .expect("Cannot get name of given file!")
                .to_string_lossy()
                .into_owned();
            let size = match dir_size(&abs_path) {
                Err(e) => return Err(e),
                Ok(x) => x,
            };

            println!("{}\t| {}", name, size);
        }
        _ => panic!("Entry can only be a directory or file"),
    }

    Ok(())
}
