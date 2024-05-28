use code2prompt::git::get_git_diff;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use git2::{Repository, Signature};

    #[test]
    fn test_get_git_diff() {
        // Create a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        // Initialize a new Git repository
        let repo = Repository::init(repo_path).expect("Failed to initialize repository");

        // Create a new file in the repository
        let file_path = repo_path.join("test_file.txt");
        fs::write(&file_path, "Initial content").expect("Failed to write to test file");

        // Stage and commit the new file
        let mut index = repo.index().expect("Failed to get repository index");
        index.add_path(file_path.strip_prefix(repo_path).unwrap()).expect("Failed to add file to index");
        index.write().expect("Failed to write index");

        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
        let signature = Signature::now("Test", "test@example.com").expect("Failed to create signature");

        repo.commit(Some("HEAD"), &signature, &signature, "Initial commit", &tree, &[]).expect("Failed to commit");

        // Modify the file
        fs::write(&file_path, "Modified content").expect("Failed to modify test file");

        // Add the modified file to the index again
        let mut index = repo.index().expect("Failed to get repository index");
        index.add_path(file_path.strip_prefix(repo_path).unwrap()).expect("Failed to add file to index");
        index.write().expect("Failed to write index");

        // Get the git diff using the function from the module
        let diff = get_git_diff(repo_path).expect("Failed to get git diff");

        // Print the diff for debugging
        println!("Generated diff:\n{}", diff);

        // Assert that the diff contains the expected content
        assert!(diff.contains("Modified content"));
    }
}
