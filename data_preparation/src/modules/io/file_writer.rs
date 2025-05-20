use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

pub struct WriteArgs<'a> {
    pub directory: PathBuf,
    pub file_name: &'a str,
    pub extension: &'a str,
    pub content: &'a str,
}

pub fn write_to_directory(args: WriteArgs) -> io::Result<()> {
    // Ensure the directory exists
    fs::create_dir_all(&args.directory)?;

    // Construct the full file path with the given extension
    let file_path = args
        .directory
        .join(args.file_name)
        .with_extension(args.extension);

    // Create and write to the file
    let mut file = File::create(file_path)?;
    file.write_all(args.content.as_bytes())?;

    Ok(())
}
