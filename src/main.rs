use std::env;

enum Entry {
    DirEntry(std::fs::DirEntry),
    PathBuf(std::path::PathBuf),
}

fn dir_size(entry: Entry) -> std::io::Result<i64> {
    let path = match entry {
        Entry::DirEntry(d) => d.path(),
        Entry::PathBuf(d) => d,
    };

    // Process the input path if it's a file.
    if path.is_file() {
        return match path.metadata() {
            Err(e) => Err(e),
            Ok(x) => match x.is_symlink() {
                true => Ok(0),
                false => Ok(x.len() as i64),
            },
        };
    }

    // Process the input if it's a directory
    let output = path
        .read_dir()
        .expect("read_dir() call failed!")
        .map(|x| match x {
            Err(e) => return Err(e),
            Ok(x) => dir_size(Entry::DirEntry(x)),
        })
        .map(|x| x.expect("Cannot extract directory size!"))
        .sum::<i64>();

    // Add default directory size to total size
    Ok(output)
}

fn main() -> std::io::Result<()> {
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

                let name = entry.file_name().into_string().unwrap();
                let size = match dir_size(Entry::DirEntry(entry)) {
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
            let size = match dir_size(Entry::PathBuf(abs_path.clone())) {
                Err(e) => return Err(e),
                Ok(x) => x,
            };

            println!("{:?}\t| {:#?}", name, size);
        }
        _ => panic!("Entry can only be a directory or file"),
    }

    Ok(())
}
