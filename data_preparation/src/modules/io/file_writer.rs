use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct WriteArgs<'a> {
    pub directory: &'a Path,
    pub file_name: &'a str,
    pub extension: &'a str,
    pub content: &'a str,
}

pub fn write_to_directory(args: WriteArgs) -> io::Result<()> {
    if Path::new(args.file_name).file_name() != Some(args.file_name.as_ref())
        || args.file_name.contains(['/', '\\', ':', '\0'])
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "file_name must be a single filename",
        ));
    }
    if args.extension.contains(['/', '\\', ':', '\0']) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "extension must not contain path separators, colons, or NUL bytes",
        ));
    }

    // Ensure the directory exists
    fs::create_dir_all(args.directory)?;

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
    let mut temporary_file = builder.tempfile_in(args.directory)?;
    temporary_file.write_all(args.content.as_bytes())?;
    temporary_file.persist(file_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_paths_before_creating_output_directory() -> io::Result<()> {
        let root = tempfile::tempdir()?;
        let output = root.path().join("output");
        let absolute = root.path().join("outside.json");
        let cases = [
            ("../outside", "json"),
            (absolute.to_str().unwrap(), "json"),
            ("nested/data", "json"),
            ("nested\\data", "json"),
            ("", "json"),
            (".", "json"),
            ("..", "json"),
            ("data", "nested/json"),
            ("data", "nested\\json"),
            ("data", "json:stream"),
        ];

        for (file_name, extension) in cases {
            let error = write_to_directory(WriteArgs {
                directory: &output,
                file_name,
                extension,
                content: "{}",
            })
            .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        }
        assert_eq!(fs::read_dir(root.path())?.count(), 0);
        Ok(())
    }

    #[test]
    fn replaces_existing_file_with_complete_contents() -> io::Result<()> {
        let directory = tempfile::tempdir()?;
        let destination = directory.path().join("data.json");
        fs::write(&destination, "old contents")?;

        write_to_directory(WriteArgs {
            directory: directory.path(),
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
            directory: directory.path(),
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
