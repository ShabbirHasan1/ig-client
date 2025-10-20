#[cfg(test)]
mod tests {
    use ig_client::{VERSION, version};

    #[test]
    fn test_version_constant() {
        // Verify VERSION is not empty
        assert!(!VERSION.is_empty());

        // Verify VERSION matches the value from CARGO_PKG_VERSION
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_version_function() {
        // Verify version() returns the same value as VERSION
        assert_eq!(version(), VERSION);

        // Verify version() returns the package version
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_version_format() {
        // Verify version follows semantic versioning format (x.y.z)
        let version_regex = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
        assert!(version_regex.is_match(VERSION));
    }
}
