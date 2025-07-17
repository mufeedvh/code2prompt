use code2prompt_core::git::{get_git_diff, get_git_diff_between_branches, get_git_log};

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository, RepositoryInitOptions, Signature};
    use std::fs;
    use tempfile::TempDir;

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
        index
            .add_path(file_path.strip_prefix(repo_path).unwrap())
            .expect("Failed to add file to index");
        index.write().expect("Failed to write index");

        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
        let signature =
            Signature::now("Test", "test@example.com").expect("Failed to create signature");

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )
        .expect("Failed to commit");

        // Modify the file
        fs::write(&file_path, "Modified content").expect("Failed to modify test file");

        // Add the modified file to the index again
        let mut index = repo.index().expect("Failed to get repository index");
        index
            .add_path(file_path.strip_prefix(repo_path).unwrap())
            .expect("Failed to add file to index");
        index.write().expect("Failed to write index");

        // Get the git diff using the function from the module
        let diff = get_git_diff(repo_path).expect("Failed to get git diff");

        // Print the diff for debugging
        println!("Generated diff:\n{}", diff);

        // Assert that the diff contains the expected content
        assert!(diff.contains("Modified content"));
    }

    #[test]
    fn test_get_git_diff_between_branches() {
        // Create a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        // Initialize a new Git repository
        let mut binding = RepositoryInitOptions::new();
        let init_options = binding.initial_head("master");
        let repo = Repository::init_opts(repo_path, init_options)
            .expect("Failed to initialize repository");

        // Create a new file in the repository
        let file_path = repo_path.join("test_file.txt");
        fs::write(&file_path, "Initial content").expect("Failed to write to test file");

        // Stage and commit the new file
        let mut index = repo.index().expect("Failed to get repository index");
        index
            .add_path(file_path.strip_prefix(repo_path).unwrap())
            .expect("Failed to add file to index");
        index.write().expect("Failed to write index");

        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
        let signature =
            Signature::now("Test", "test@example.com").expect("Failed to create signature");

        let master_commit = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initial commit in master branch",
                &tree,
                &[],
            )
            .expect("Failed to commit");

        // Create a new branch and make a commit on the master branch
        repo.branch(
            "development",
            &repo
                .find_commit(master_commit)
                .expect("Failed to find commit"),
            false,
        )
        .expect("Failed to create new branch");

        // Modify the file in the new branch
        repo.set_head("refs/heads/development")
            .expect("Failed to set HEAD");
        repo.checkout_head(None).expect("Failed to checkout HEAD");
        fs::write(&file_path, "Content in new branch")
            .expect("Failed to modify test file in new branch");

        let mut index = repo.index().expect("Failed to get repository index");
        index
            .add_path(file_path.strip_prefix(repo_path).unwrap())
            .expect("Failed to add file to index");
        index.write().expect("Failed to write index");

        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "New commit in branch development",
            &tree,
            &[&repo
                .find_commit(master_commit)
                .expect("Failed to find commit")],
        )
        .expect("Failed to commit in new branch");

        // Get the git diff between branches
        let diff = get_git_diff_between_branches(repo_path, "master", "development")
            .expect("Failed to get git diff between branches");

        // Print the diff for debugging
        println!("Generated diff between branches:\n{}", diff);

        // Assert that the diff contains the expected content
        assert!(diff.contains("Initial content"));
        assert!(diff.contains("Content in new branch"));
    }

    #[test]
    fn test_get_git_log() {
        // Create a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        // Initialize a new Git repository
        let mut binding = RepositoryInitOptions::new();
        let init_options = binding.initial_head("master");
        let repo = Repository::init_opts(repo_path, init_options)
            .expect("Failed to initialize repository");

        // Create a new file in the repository
        let file_path = repo_path.join("test_file.txt");
        fs::write(&file_path, "Initial content").expect("Failed to write to test file");

        // Stage and commit the new file
        let mut index = repo.index().expect("Failed to get repository index");
        index
            .add_path(file_path.strip_prefix(repo_path).unwrap())
            .expect("Failed to add file to index");
        index.write().expect("Failed to write index");

        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
        let signature =
            Signature::now("Test", "test@example.com").expect("Failed to create signature");

        let master_commit = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initial commit in branch master",
                &tree,
                &[],
            )
            .expect("Failed to commit");

        // Create a new branch and make a commit on the master branch
        repo.branch(
            "development",
            &repo
                .find_commit(master_commit)
                .expect("Failed to find commit"),
            false,
        )
        .expect("Failed to create new branch");

        // Modify the file in the new branch
        repo.set_head("refs/heads/development")
            .expect("Failed to set HEAD");
        repo.checkout_head(None).expect("Failed to checkout HEAD");
        fs::write(&file_path, "Content in development")
            .expect("Failed to modify test file in new branch");

        let mut index = repo.index().expect("Failed to get repository index");
        index
            .add_path(file_path.strip_prefix(repo_path).unwrap())
            .expect("Failed to add file to index");
        index.write().expect("Failed to write index");

        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "First commit in development",
            &tree,
            &[&repo
                .find_commit(master_commit)
                .expect("Failed to find commit")],
        )
        .expect("Failed to commit in new branch");

        // Make a second commit in the development branch
        fs::write(&file_path, "Second content in development")
            .expect("Failed to modify test file in new branch");

        let mut index = repo.index().expect("Failed to get repository index");
        index
            .add_path(file_path.strip_prefix(repo_path).unwrap())
            .expect("Failed to add file to index");
        index.write().expect("Failed to write index");

        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Second commit in development",
            &tree,
            &[&repo
                .find_commit(repo.head().unwrap().target().unwrap())
                .expect("Failed to find commit")],
        )
        .expect("Failed to commit second change in new branch");

        // Get the git log between branches
        let log = get_git_log(repo_path, "master", "development")
            .expect("Failed to get git log between branches");

        // Print the log for debugging
        println!("Generated git log:\n{}", log);

        // Assert that the log contains the expected content
        assert!(log.contains("First commit in development"));
        assert!(log.contains("Second commit in development"));
    }

    #[test]
    fn test_git_diff_with_commit_hashes_and_tags() {
        // Create a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        // Initialize a new Git repository
        let mut binding = RepositoryInitOptions::new();
        let init_options = binding.initial_head("master");
        let repo = Repository::init_opts(repo_path, init_options)
            .expect("Failed to initialize repository");

        // Create a new file in the repository
        let file_path = repo_path.join("test_file.txt");
        fs::write(&file_path, "Initial content").expect("Failed to write to test file");

        // Stage and commit the new file
        let mut index = repo.index().expect("Failed to get repository index");
        index
            .add_path(file_path.strip_prefix(repo_path).unwrap())
            .expect("Failed to add file to index");
        index.write().expect("Failed to write index");

        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
        let signature =
            Signature::now("Test", "test@example.com").expect("Failed to create signature");

        let first_commit_id = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "First commit",
                &tree,
                &[],
            )
            .expect("Failed to commit");

        // Create a tag for the first commit
        let first_commit = repo
            .find_commit(first_commit_id)
            .expect("Failed to find first commit");
        repo.tag(
            "v1.0.0",
            &first_commit.as_object(),
            &signature,
            "Version 1.0.0",
            false,
        )
        .expect("Failed to create tag");

        // Make a second commit
        fs::write(&file_path, "Modified content").expect("Failed to modify test file");
        let mut index = repo.index().expect("Failed to get repository index");
        index
            .add_path(file_path.strip_prefix(repo_path).unwrap())
            .expect("Failed to add file to index");
        index.write().expect("Failed to write index");

        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");

        let second_commit_id = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Second commit",
                &tree,
                &[&first_commit],
            )
            .expect("Failed to commit second change");

        // Test 1: Diff between commit hashes (full hash)
        let first_commit_hash = first_commit_id.to_string();
        let second_commit_hash = second_commit_id.to_string();

        let diff_full_hash =
            get_git_diff_between_branches(repo_path, &first_commit_hash, &second_commit_hash)
                .expect("Failed to get git diff between full commit hashes");

        assert!(diff_full_hash.contains("Initial content"));
        assert!(diff_full_hash.contains("Modified content"));

        // Test 2: Diff between abbreviated commit hashes
        let first_commit_short = &first_commit_hash[..7];
        let second_commit_short = &second_commit_hash[..7];

        let diff_short_hash =
            get_git_diff_between_branches(repo_path, first_commit_short, second_commit_short)
                .expect("Failed to get git diff between abbreviated commit hashes");

        assert!(diff_short_hash.contains("Initial content"));
        assert!(diff_short_hash.contains("Modified content"));

        // Test 3: Diff between tag and commit hash
        let diff_tag_to_hash =
            get_git_diff_between_branches(repo_path, "v1.0.0", &second_commit_hash)
                .expect("Failed to get git diff between tag and commit hash");

        assert!(diff_tag_to_hash.contains("Initial content"));
        assert!(diff_tag_to_hash.contains("Modified content"));

        // Test 4: Diff between tag and HEAD
        let diff_tag_to_head = get_git_diff_between_branches(repo_path, "v1.0.0", "HEAD")
            .expect("Failed to get git diff between tag and HEAD");

        assert!(diff_tag_to_head.contains("Initial content"));
        assert!(diff_tag_to_head.contains("Modified content"));

        // Test 5: Error case - invalid reference should still fail
        let result = get_git_diff_between_branches(repo_path, "nonexistent_reference", "HEAD");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Branch nonexistent_reference doesn't exist!"));
    }
}
