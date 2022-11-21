use std::num::NonZeroUsize;
use std::process::ExitCode;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use std::path::Path;

use ringhopper::engines::h1::TagReference;
use ringhopper_proc::*;
use macros::terminal::*;
use ringhopper::engines::h1::TagGroup;
use ringhopper::error::*;
use ringhopper::file::TagFile;

pub mod bitmap;
pub mod collection;
pub mod convert;
pub mod list_engines;
pub mod recover;
pub mod recover_processed;
pub mod script;
pub mod sound;
pub mod strip;
pub mod unicode_strings;

#[derive(Clone, Copy)]
pub struct BatchResult {
    pub success: usize,
    pub errors: usize,
    pub skipped: usize
}

impl BatchResult {
    fn exit_code(self) -> ExitCode {
        match self.errors {
            0 => ExitCode::SUCCESS,
            _ => ExitCode::FAILURE
        }
    }
}

pub type LogMutex = Arc<Mutex<bool>>;

pub fn do_with_batching_threaded<T: Sized + Clone + Send + 'static, F: Fn(&TagFile, LogMutex, NonZeroUsize, &T) -> ErrorMessageResult<bool> + Send + Copy + 'static>(function: F, pattern: &str, group: Option<TagGroup>, tags_directories: &[&Path], thread_count: NonZeroUsize, data: T) -> ErrorMessageResult<BatchResult> {
    let success = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(AtomicUsize::new(0));
    let skipped = Arc::new(AtomicUsize::new(0));
    let current_tag_index = Arc::new(AtomicUsize::new(0));
    let thread_count = thread_count.get();
    let batching = TagFile::uses_batching(pattern);

    let batch_result = TagFile::from_tag_path_batched(tags_directories, pattern, group)?;
    let all_tags = match batching {
        // If we do batching, use the batching result
        true => batch_result,

        // If it is not batching but we have something, use the batching result
        false if !batch_result.is_empty() => batch_result,

        // If it doesn't exist, use the first directory
        _ => {
            let reference = match group {
                Some(n) => TagReference::from_path_and_group(pattern, n)?,
                None => TagReference::from_full_path(pattern)?
            };
            vec![TagFile { file_path: tags_directories[0].join(reference.to_string()), tag_path: reference }]
        }
    };

    let tag_count = all_tags.len();
    let all_tags = Arc::new(Mutex::new(all_tags));

    let log_mutex = LogMutex::new(Mutex::new(false));

    let available_threads = NonZeroUsize::new((thread_count / tag_count.max(1)).max(1)).unwrap();

    let mut threads = Vec::with_capacity(thread_count);
    for _ in 0..thread_count {
        let tag_count = tag_count.clone();
        let current_tag_index = current_tag_index.clone();
        let success = success.clone();
        let errors = errors.clone();
        let skipped = skipped.clone();
        let data = data.clone();
        let all_tags = all_tags.clone();
        let log_mutex = log_mutex.clone();

        threads.push(std::thread::spawn(move || -> () {
            loop {
                let tag_index = current_tag_index.fetch_add(1, Ordering::Relaxed);
                if tag_index >= tag_count {
                    current_tag_index.fetch_sub(1, Ordering::Relaxed);
                    return;
                }

                let tag_file = all_tags.lock().unwrap().get(tag_index).unwrap().to_owned();
                match function(&tag_file, log_mutex.clone(), available_threads, &data) {
                    Ok(true) => {
                        success.fetch_add(1, Ordering::Relaxed);
                    },
                    Ok(_) => {
                        skipped.fetch_add(1, Ordering::Relaxed);
                    },
                    Err(e) => {
                        errors.fetch_add(1, Ordering::Relaxed);
                        let l = log_mutex.lock();
                        eprintln_error_pre!("Failed to process {tag}: {e}", tag=tag_file.tag_path, e=e);
                        drop(l);
                    }
                }

            }
        }));
    }

    for t in threads {
        t.join().unwrap();
    }

    let success = success.load(Ordering::Relaxed);
    let errors = errors.load(Ordering::Relaxed);
    let skipped = skipped.load(Ordering::Relaxed);
    let total = success + errors + skipped;

    if TagFile::uses_batching(pattern) {
        if total == 0 {
            return Err(ErrorMessage::AllocatedString(format!("No tags match {pattern}", pattern=pattern)));
        }
        if skipped > 0 {
            println!("Skipped {skipped} tag(s).", skipped=skipped);
        }
        if errors > 0 {
            println_warn!("Successfully processed {success} tag(s) with {errors} errors.", success=success, errors=errors);
        }
        else {
            println_success!("Successfully processed {total} tag(s).", total=total);
        }
    }

    Ok(BatchResult { success, errors, skipped })
}
