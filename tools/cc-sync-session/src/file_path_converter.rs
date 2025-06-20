use std::path::PathBuf;

pub fn dir_path_to_claude_code_stype(dir_path: PathBuf) -> anyhow::Result<String> {
    if !dir_path.is_absolute() {
        return Err(anyhow::anyhow!("Path must be absolute"));
    }
    if let Some(ext) =  dir_path.extension() {
        return Err(anyhow::anyhow!("Path should not have an extension, found: {}", ext.to_string_lossy()));
    }
    
    // Convert path to string and normalize
    let path_str = dir_path.to_string_lossy();
    
    // Remove leading slash and replace path separators with dashes
    let without_leading_slash = path_str.trim_start_matches('/');
    let with_dashes = without_leading_slash.replace('/', "-");
    
    // Add leading dash
    Ok(format!("-{}", with_dashes))
}

pub fn claude_code_stype_to_file_path(code_stype: &str) -> PathBuf {
    // Remove leading dash if present
    let without_leading_dash = code_stype.trim_start_matches('-');
    
    // Replace dashes with path separators and add leading slash
    let path_with_slashes = format!("/{}", without_leading_dash.replace('-', "/"));
    
    PathBuf::from(path_with_slashes)
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
    fn test_claude_code_stype_to_file_path() {
        let code_stype = "-path-to";
        let result = claude_code_stype_to_file_path(code_stype);
        // Add assertions based on expected output
        assert_eq!(result, PathBuf::from("/path/to")); // Replace with actual expected output
    }
    
    /// prop-test for file path conversion back to back consistency
    #[test]
    fn test_file_path_conversion_consistency() {
        let original_path = PathBuf::from("/path/to/original");
        let cc_stype = dir_path_to_claude_code_stype(original_path.clone());
        let converted_path = claude_code_stype_to_file_path(&cc_stype.unwrap());
        
        assert_eq!(converted_path, original_path);
    }
}