use std::path::Path;
use std::fs::File;
use ringhopper::error::*;
use ringhopper_proc::*;
use std::io::*;

/// Helper function for converting a string slice into a [`Path`] vector.
pub fn str_slice_to_path_vec(strs: &[String]) -> Vec<&Path> {
    let mut paths = Vec::<&Path>::new();

    for i in strs {
        paths.push(Path::new(i));
    }

    return paths;
}

/// Read all bytes of a file.
///
/// Return the bytes read or an error if failed.
pub fn read_file(path: &Path) -> ErrorMessageResult<Vec<u8>> {
    let mut file = match File::open(path.to_owned()) {
        Ok(n) => n,
        Err(error) => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_opening_file_read"), error=error, file=path.display())))
    };

    let mut data = Vec::<u8>::new();
    match file.read_to_end(&mut data) {
        Ok(_) => Ok(data),
        Err(error) => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_reading_file"), error=error, file=path.display())))
    }
}

/// Write all bytes to a file, overwriting it.
///
/// Return the bytes read or an error if failed.
pub fn write_file(path: &Path, data: &[u8]) -> ErrorMessageResult<()> {
    let mut file = match File::create(path.to_owned()) {
        Ok(n) => n,
        Err(error) => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_opening_file_write"), error=error, file=path.display())))
    };

    match file.write_all(&data) {
        Ok(_) => Ok(()),
        Err(e) =>  Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_writing_file"), error=e, file=path.display())))
    }
}

/// Make directories for the file if necessary.
pub fn make_directories(path: &Path) -> ErrorMessageResult<()> {
    match std::fs::create_dir_all(path) {
        Ok(_) => Ok(()),
        Err(error) => Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.verbs.unicode-strings.error_creating_directories"), error=error, dirs=path.display())))
    }
}

/// Make parent directories for the file if necessary.
pub fn make_parent_directories(path: &Path) -> ErrorMessageResult<()> {
    make_directories(&path.parent().unwrap())
}

/// Format the size to a human-readable size
pub fn format_size(length: usize) -> String {
    // Convert to 64-bit float
    let length = length as f64;

    let suffix = (|| {
        let suffixes = &[
            (1.0, "B"),
            (1024.0, "KiB"),
            (1024.0 * 1024.0, "MiB"),
            (1024.0 * 1024.0 * 1024.0, "GiB"),
            (1024.0 * 1024.0 * 1024.0 * 1024.0, "TiB")
        ];
        for i in 0..suffixes.len() - 1 {
            if length < suffixes[i + 1].0 {
                return suffixes[i];
            }
        }
        return *suffixes.last().unwrap();
    })();

    format!("{length:0.3} {suffix}", length=length / suffix.0, suffix=suffix.1)
}
