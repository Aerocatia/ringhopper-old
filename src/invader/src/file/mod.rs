use std::io::Read;
use std::io::Write;
use std::path::Path;
use crate::error::*;
use strings::*;

/// Read all bytes of a file.
///
/// Return the bytes read or an error if failed.
pub fn read_file(path: &Path) -> ErrorMessageResult<Vec<u8>> {
    let mut file = match std::fs::File::open(path.to_owned()) {
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
    let mut file = match std::fs::File::create(path.to_owned()) {
        Ok(n) => n,
        Err(error) => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_opening_file_write"), error=error, file=path.display())))
    };

    match file.write_all(&data) {
        Ok(_) => Ok(()),
        Err(e) =>  Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_writing_file"), error=e, file=path.display())))
    }
}
