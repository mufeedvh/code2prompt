use serde_json::Value;

///! Sorting methods for files.

// Define the available sort methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSortMethod {
    NameAsc,  // Sort files alphabetically (A → Z)
    NameDesc, // Sort files alphabetically in reverse (Z → A)
    DateAsc,  // Sort files by modification date (oldest first)
    DateDesc, // Sort files by modification date (newest first)
}

/// Sorts the provided `files` in place using the specified `sort_method`.
///
/// If `sort_method` is `None`, no sorting will be performed.
///
/// # Arguments
///
/// * `files` - A mutable slice of JSON values representing files. Each file is expected
///             to have a `"path"` key (as a string) and a `"mod_time"` key (as a u64).
/// * `sort_method` - An optional `FileSortMethod` indicating how to sort the files.
pub fn sort_files(files: &mut Vec<Value>, sort_method: Option<FileSortMethod>) {
    if let Some(method) = sort_method {
        files.sort_by(|a, b| match method {
            FileSortMethod::NameAsc => {
                let a_path = a.get("path").and_then(Value::as_str).unwrap_or("");
                let b_path = b.get("path").and_then(Value::as_str).unwrap_or("");
                a_path.cmp(b_path)
            }
            FileSortMethod::NameDesc => {
                let a_path = a.get("path").and_then(Value::as_str).unwrap_or("");
                let b_path = b.get("path").and_then(Value::as_str).unwrap_or("");
                b_path.cmp(a_path)
            }
            FileSortMethod::DateAsc => {
                let a_time = a.get("mod_time").and_then(Value::as_u64).unwrap_or(0);
                let b_time = b.get("mod_time").and_then(Value::as_u64).unwrap_or(0);
                a_time.cmp(&b_time)
            }
            FileSortMethod::DateDesc => {
                let a_time = a.get("mod_time").and_then(Value::as_u64).unwrap_or(0);
                let b_time = b.get("mod_time").and_then(Value::as_u64).unwrap_or(0);
                b_time.cmp(&a_time)
            }
        });
    }
}
