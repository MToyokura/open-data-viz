use std::fs;
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

    let mut builder = tempfile::Builder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(fs::Permissions::from_mode(0o666));
    }
    let mut temporary_file = builder.tempfile_in(file_path.parent().unwrap_or(&args.directory))?;
    temporary_file.write_all(args.content.as_bytes())?;
    temporary_file.persist(file_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_existing_file_with_complete_contents() -> io::Result<()> {
        let directory = tempfile::tempdir()?;
        let destination = directory.path().join("data.json");
        fs::write(&destination, "old contents")?;

        write_to_directory(WriteArgs {
            directory: directory.path().into(),
            file_name: "data.json",
            extension: "json",
            content: "{}",
        })?;

        assert_eq!(fs::read_to_string(destination)?, "{}");
        assert_eq!(fs::read_dir(directory.path())?.count(), 1);
        Ok(())
    }

    #[test]
    fn removes_temporary_file_when_rename_fails() -> io::Result<()> {
        let directory = tempfile::tempdir()?;
        let destination = directory.path().join("data.json");
        fs::create_dir(&destination)?;

        let result = write_to_directory(WriteArgs {
            directory: directory.path().into(),
            file_name: "data",
            extension: "json",
            content: "{}",
        });

        assert!(result.is_err());
        assert!(destination.is_dir());
        assert_eq!(fs::read_dir(directory.path())?.count(), 1);
        Ok(())
    }
}
