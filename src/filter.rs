// src/filter.rs

use std::path::Path;

pub fn should_include_file(
    path: &Path,
    include_extensions: &[String],
    exclude_extensions: &[String],
    include_files: &[String],
    exclude_files: &[String],
) -> bool {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("").to_string();
    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("").to_string();

    if include_files.contains(&file_name) || include_extensions.contains(&extension) {
        return true;
    }

    if exclude_files.contains(&file_name) || exclude_extensions.contains(&extension) {
        return false;
    }

    if include_files.is_empty() && exclude_files.is_empty() && include_extensions.is_empty() && exclude_extensions.is_empty() {
        return true;
    }

    false
}
