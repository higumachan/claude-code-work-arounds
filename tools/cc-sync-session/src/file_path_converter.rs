use std::collections::BTreeSet;
use std::path::PathBuf;

pub fn dir_path_to_claude_code_stype(dir_path: PathBuf) -> anyhow::Result<String> {
    if !dir_path.is_absolute() {
        return Err(anyhow::anyhow!("Path must be absolute"));
    }
    if dir_path.is_file() {
        return Err(anyhow::anyhow!("Path must be a directory, not a file"));
    }
   
    // Convert path to string and normalize
    let path_str = dir_path.to_string_lossy();
    
    // Remove leading slash and replace path separators with dashes
    let without_leading_slash = path_str.trim_start_matches('/');
    let with_dashes = without_leading_slash.replace('/', "-").replace('.', "-");
    
    // Add leading dash
    Ok(format!("-{}", with_dashes))
}

pub fn claude_code_stype_to_file_path(code_stype: &str) -> BTreeSet<PathBuf> {
    let mut results = BTreeSet::new();
    
    // Remove leading dash if present
    let without_leading_dash = code_stype.strip_prefix('-').unwrap_or(code_stype);
    
    // Replace dashes with slashes to create base path
    let base_path = format!("/{}", without_leading_dash.replace('-', "/"));
    results.insert(PathBuf::from(base_path));
    
    // Since dots were converted to dashes, we need to consider possible original paths
    // For example, "-Users-yuta-github-com" could be:
    // - /Users/yuta/github.com
    // - /Users/yuta/github/com
    // For now, we'll just return the basic conversion
    // A more sophisticated implementation would need to check common patterns
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_path_to_claude_code_stype() {
        let path = PathBuf::from("/path/to");
        let result = dir_path_to_claude_code_stype(path);
        // Add assertions based on expected output
        assert_eq!(result.unwrap(), "-path-to"); // Replace with actual expected output
    }

    #[test]
    fn test_file_path_to_claude_code_stype_with_dot() {
        let path = PathBuf::from("/path/to/github.com");
        let result = dir_path_to_claude_code_stype(path);
        // Add assertions based on expected output
        assert_eq!(result.unwrap(), "-path-to-github-com"); // Replace with actual expected output
    }


    #[test]
    fn test_claude_code_stype_to_file_path() {
        let code_stype = "-path-to";
        let result = claude_code_stype_to_file_path(code_stype);
        // Add assertions based on expected output
        assert!(result.contains(&PathBuf::from("/path/to")));
    }
    
    /// prop-test for file path conversion back to back consistency
    #[test]
    fn test_file_path_conversion_consistency() {
        let original_path = PathBuf::from("/path/to/original");
        let cc_stype = dir_path_to_claude_code_stype(original_path.clone()).unwrap();
        let converted_paths = claude_code_stype_to_file_path(&cc_stype);
        
        // The original path should be one of the possible results
        assert!(converted_paths.contains(&original_path));
    }
}