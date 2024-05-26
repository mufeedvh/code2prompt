use code2prompt::get_git_diff;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_git_diff() {
        let path = PathBuf::from(".");
        let result = get_git_diff(&path);
        assert!(result.is_ok());
    }
}
