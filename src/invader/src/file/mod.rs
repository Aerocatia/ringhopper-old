//! File loading functions.

use std::io::Read;
use std::io::Write;
use std::path::Path;
use crate::error::*;
use strings::*;

use std::path::PathBuf;
use crate::engines::h1::{TagGroup, TagReference};
use crate::types::tag::TagGroupFn;

/// Read all bytes of a file.
///
/// Return the bytes read or an error if failed.
pub(crate) fn read_file(path: &Path) -> ErrorMessageResult<Vec<u8>> {
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
pub(crate) fn write_file(path: &Path, data: &[u8]) -> ErrorMessageResult<()> {
    let mut file = match std::fs::File::create(path.to_owned()) {
        Ok(n) => n,
        Err(error) => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_opening_file_write"), error=error, file=path.display())))
    };

    match file.write_all(&data) {
        Ok(_) => Ok(()),
        Err(e) =>  Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_writing_file"), error=e, file=path.display())))
    }
}

/// Struct used to associate a tag to a file.
pub struct TagFile {
    /// Internal tag path in the virtual tags directory.
    pub tag_path: TagReference,

    /// Filesystem path.
    pub file_path: PathBuf
}

/// Iterate a directory recursively for tags.
#[cfg(target_os = "windows")]
fn iterate_recursively(base_directory: &Path, directory: &Path, recursion_limit: usize, results: &mut Vec<TagFile>) -> ErrorMessageResult<()> {
    // This version uses the Win32 API directly. This is drastically faster than Rust's [`std::fs`] functions on Windows, although note that some unsafe code is needed to do this.

    use windows::Win32::Storage::FileSystem::*;
    use windows::Win32::Foundation::CHAR;
    use windows::core::PCSTR;

    if recursion_limit == 0 {
        return Err(ErrorMessage::StaticString(get_compiled_string!("file.error_reading_virtual_tags_directory")))
    }

    let mut find_data = WIN32_FIND_DATAA::default();
    let path_str = directory.to_str().unwrap();
    let mut path_bytes = Vec::<u8>::new();
    path_bytes.reserve_exact(path_str.len() + 3);
    path_bytes.extend_from_slice(path_str.as_bytes());
    path_bytes.extend_from_slice(b"\\*\x00");

    if let Ok(file) = unsafe { FindFirstFileA(PCSTR(path_bytes.as_ptr()), &mut find_data) } {
        // Do it!
        let result = (|| {
            loop {
                let file_name_str_slice = unsafe { &*std::ptr::slice_from_raw_parts(&find_data.cFileName as *const CHAR as *const u8, find_data.cFileName.len()) };
                let mut file_name_str_len = 0usize;
                for &c in file_name_str_slice {
                    if c == 0 {
                        break;
                    }
                    file_name_str_len += 1;
                }
                let file_name_str = unsafe { std::str::from_utf8_unchecked(&file_name_str_slice[..file_name_str_len]) };

                if file_name_str != "." && file_name_str != ".." {
                    let file_path = directory.join(file_name_str);
                    if find_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY.0 != 0 {
                        iterate_recursively(base_directory, &file_path, recursion_limit - 1, results)?;
                    }
                    else {
                        let relative_path = file_path.strip_prefix(base_directory).unwrap();
                        let path_str = match relative_path.to_str() {
                            Some(n) => n,
                            None => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_non_utf8_path"), path=relative_path.to_string_lossy())))
                        };
                        match TagReference::from_path_with_extension(path_str) {
                            Ok(tag_path) => results.push(TagFile { tag_path, file_path }),
                            Err(_) => ()
                        }
                    }
                }
                if unsafe { !FindNextFileA(file, &mut find_data) }.as_bool() {
                    break;
                }
            }
            Ok(())
        })();

        // Close the handle
        unsafe { FindClose(file); }

        // If this failed, exit the function with an error.
        result?;
    }

    Ok(())
}

/// Iterate a directory recursively for tags.
#[cfg(not(target_os = "windows"))]
fn iterate_recursively(base_directory: &Path, directory: &Path, recursion_limit: usize, results: &mut Vec<TagFile>) -> ErrorMessageResult<()> {
    // This version uses the [`std::fs`] module and is cross-platform, but there is a separate implementation just for Windows for better performance.

    if recursion_limit == 0 {
        return Err(ErrorMessage::StaticString(get_compiled_string!("file.error_reading_virtual_tags_directory")))
    }

    let iterator = match std::fs::read_dir(directory) {
        Ok(n) => n,
        Err(e) => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_iterating_directory"), path=directory.to_string_lossy(), error=e)))
    };

    for i in iterator {
        let file = match i {
            Ok(n) => n,
            Err(e) => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_iterating_directory"), path=directory.to_string_lossy(), error=e)))
        };

        let file_path = file.path();
        if file_path.is_dir() {
            iterate_recursively(base_directory, &file_path, recursion_limit - 1, results)?;
        }
        else if file_path.is_file() {
            let relative_path = file_path.strip_prefix(base_directory).unwrap();
            let path_str = match relative_path.to_str() {
                Some(n) => n,
                None => return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("file.error_non_utf8_path"), path=relative_path.to_string_lossy())))
            };
            match TagReference::from_path_with_extension(path_str) {
                Ok(tag_path) => results.push(TagFile { tag_path, file_path }),
                Err(_) => continue
            }
        }
    }
    Ok(())
}

impl TagFile {
    /// Get all tags located in a virtual tags directory.
    pub fn from_virtual_tags_directory(tags_directories: &[&Path]) -> ErrorMessageResult<Vec<TagFile>> {
        if tags_directories.len() == 0 {
            return Ok(Vec::new())
        }

        let mut results = Vec::<ErrorMessageResult<Vec<TagFile>>>::new();

        let tag_dir_count = tags_directories.len();
        for i in 0..tag_dir_count {
            let dir = tags_directories[i].to_owned();

            let mut vec = Vec::new();
            results.push(iterate_recursively(&dir, &dir, 128, &mut vec).map(|_| vec))
        }

        // Return what we got
        if results.len() == 1 {
            results.pop().unwrap()
        }

        // Go through everything!
        else {
            // First move the first directory since it doesn't need to check past directories
            let mut final_results = results.pop().unwrap()?;

            for i in results {
                for p in i? {
                    let contained = (|| {
                        for r in &final_results {
                            if r.tag_path == p.tag_path {
                                return true;
                            }
                        }
                        return false;
                    })();
                    if !contained {
                        final_results.push(p);
                    }
                }
            }
            Ok(final_results)
        }
    }

    /// Search `tags_directories` for a `tag_reference`.
    ///
    /// Return `Some` if one is present and `None` if not.
    pub fn from_tag_ref(tags_directories: &[&Path], tag_reference: &TagReference) -> Option<TagFile> {
        let tag_path = tag_reference.get_relative_fs_path();
        for tag_dir in tags_directories {
            let new_path = tag_dir.join(&tag_path);
            if new_path.is_file() {
                return Some(TagFile { tag_path: tag_reference.to_owned(), file_path: new_path })
            }
        }
        None
    }

    /// Search `tags_directories` for a `pattern` and optionally a `group`.
    ///
    /// If `group` is None and `pattern` does not contain wildcard characters, then `Err` will be returned if
    /// `pattern` does not resolve into a valid [`TagReference`].
    ///
    /// If `group` is not None, then its file extension will be appended to the search query.
    ///
    /// Return `Err` if the input is invalid. Otherwise returns a [`Vec`] containing all tags that match the query or
    /// empty if none were found.
    pub fn from_tag_path_batched(tags_directories: &[&Path], pattern: &str, group: Option<TagGroup>) -> ErrorMessageResult<Vec<TagFile>> {
        // Append the tag group to the query if we need to.
        let tag_path_to_search = match group {
            Some(n) => format!("{pattern}.{}", n.as_str()),
            None => pattern.to_owned()
        };

        // Check if we don't actually need to do any batching.
        if !pattern.contains(&['*', '?']) {
            let reference = TagReference::from_path_with_extension(&tag_path_to_search)?;
            return match TagFile::from_tag_ref(tags_directories, &reference) {
                Some(n) => Ok(vec![n]),
                None => Ok(vec![])
            };
        }

        let mut everything = TagFile::from_virtual_tags_directory(tags_directories)?;
        everything.retain(|i| i.tag_path.matches_pattern(pattern));
        Ok(everything)
    }
}

pub(crate) fn str_slice_to_path_vec(strs: &[String]) -> Vec<&Path> {
    let mut paths = Vec::<&Path>::new();

    for i in strs {
        paths.push(Path::new(i));
    }

    return paths;
}
