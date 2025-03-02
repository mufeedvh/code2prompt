use code2prompt_core::sort::{sort_files, sort_tree, FileSortMethod};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use termtree::Tree;

    #[test]
    fn test_sort_files_name_asc() {
        // Create a vector of file JSON objects with "path" and "mod_time"
        let mut files = vec![
            json!({"path": "zeta.txt", "mod_time": 100}),
            json!({"path": "alpha.txt", "mod_time": 200}),
            json!({"path": "beta.txt", "mod_time": 150}),
        ];

        // Sort by file name in ascending order (A → Z)
        sort_files(&mut files, Some(FileSortMethod::NameAsc));

        // Expected order is: "alpha.txt", "beta.txt", "zeta.txt"
        let expected = vec!["alpha.txt", "beta.txt", "zeta.txt"];
        let result: Vec<String> = files
            .iter()
            .map(|v| {
                v.get("path")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string()
            })
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sort_files_name_desc() {
        // Create a vector of file JSON objects with "path" and "mod_time"
        let mut files = vec![
            json!({"path": "alpha.txt", "mod_time": 100}),
            json!({"path": "zeta.txt", "mod_time": 200}),
            json!({"path": "beta.txt", "mod_time": 150}),
        ];

        // Sort by file name in descending order (Z → A)
        sort_files(&mut files, Some(FileSortMethod::NameDesc));

        // Expected order is: "zeta.txt", "beta.txt", "alpha.txt"
        let expected = vec!["zeta.txt", "beta.txt", "alpha.txt"];
        let result: Vec<String> = files
            .iter()
            .map(|v| {
                v.get("path")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string()
            })
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sort_files_date_asc() {
        // Create a vector of file JSON objects with "path" and "mod_time"
        let mut files = vec![
            json!({"path": "file1.txt", "mod_time": 300}),
            json!({"path": "file2.txt", "mod_time": 100}),
            json!({"path": "file3.txt", "mod_time": 200}),
        ];

        // Sort by modification time in ascending order (oldest first)
        sort_files(&mut files, Some(FileSortMethod::DateAsc));

        // Expected order is: "file2.txt" (100), "file3.txt" (200), "file1.txt" (300)
        let expected = vec!["file2.txt", "file3.txt", "file1.txt"];
        let result: Vec<String> = files
            .iter()
            .map(|v| {
                v.get("path")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string()
            })
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sort_files_date_desc() {
        // Create a vector of file JSON objects with "path" and "mod_time"
        let mut files = vec![
            json!({"path": "file1.txt", "mod_time": 300}),
            json!({"path": "file2.txt", "mod_time": 100}),
            json!({"path": "file3.txt", "mod_time": 200}),
        ];

        // Sort by modification time in descending order (newest first)
        sort_files(&mut files, Some(FileSortMethod::DateDesc));

        // Expected order is: "file1.txt" (300), "file3.txt" (200), "file2.txt" (100)
        let expected = vec!["file1.txt", "file3.txt", "file2.txt"];
        let result: Vec<String> = files
            .iter()
            .map(|v| {
                v.get("path")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string()
            })
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sort_files_none() {
        // When sort method is None, the original order should be preserved.
        let original_files = vec![
            json!({"path": "zeta.txt", "mod_time": 300}),
            json!({"path": "alpha.txt", "mod_time": 100}),
            json!({"path": "beta.txt", "mod_time": 200}),
        ];
        let mut files = original_files.clone();

        // Sorting with None should leave the order unchanged.
        sort_files(&mut files, None);
        assert_eq!(files, original_files);
    }

    #[test]
    fn test_sort_tree_name_asc() {
        // Build a simple tree with unsorted leaf nodes.
        let mut tree = Tree::new("root".to_string());
        tree.leaves.push(Tree::new("zeta".to_string()));
        tree.leaves.push(Tree::new("alpha".to_string()));
        tree.leaves.push(Tree::new("beta".to_string()));

        // Sort the tree using NameAsc.
        sort_tree(&mut tree, Some(FileSortMethod::NameAsc));

        // Extract the sorted names.
        let sorted: Vec<String> = tree.leaves.iter().map(|node| node.root.clone()).collect();
        let expected = vec!["alpha".to_string(), "beta".to_string(), "zeta".to_string()];
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_sort_tree_name_desc() {
        let mut tree = Tree::new("root".to_string());
        tree.leaves.push(Tree::new("alpha".to_string()));
        tree.leaves.push(Tree::new("zeta".to_string()));
        tree.leaves.push(Tree::new("beta".to_string()));

        // Sort the tree using NameDesc.
        sort_tree(&mut tree, Some(FileSortMethod::NameDesc));

        let sorted: Vec<String> = tree.leaves.iter().map(|node| node.root.clone()).collect();
        let expected = vec!["zeta".to_string(), "beta".to_string(), "alpha".to_string()];
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_sort_tree_date_asc_falls_back_to_name() {
        // For directory trees, date-based sorting should fall back to name-based sorting.
        let mut tree = Tree::new("root".to_string());
        tree.leaves.push(Tree::new("delta".to_string()));
        tree.leaves.push(Tree::new("charlie".to_string()));
        tree.leaves.push(Tree::new("bravo".to_string()));

        sort_tree(&mut tree, Some(FileSortMethod::DateAsc));

        let sorted: Vec<String> = tree.leaves.iter().map(|node| node.root.clone()).collect();
        let expected = vec![
            "bravo".to_string(),
            "charlie".to_string(),
            "delta".to_string(),
        ];
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_sort_tree_none() {
        // If sort_method is None, the tree should remain in its original order.
        let mut tree = Tree::new("root".to_string());
        tree.leaves.push(Tree::new("zeta".to_string()));
        tree.leaves.push(Tree::new("alpha".to_string()));
        tree.leaves.push(Tree::new("beta".to_string()));

        let original: Vec<String> = tree.leaves.iter().map(|node| node.root.clone()).collect();
        sort_tree(&mut tree, None);
        let after: Vec<String> = tree.leaves.iter().map(|node| node.root.clone()).collect();

        assert_eq!(original, after);
    }
}
