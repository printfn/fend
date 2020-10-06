const CHANGELOG: &str = include_str!("../CHANGELOG.md");

pub fn get_changelog() -> &'static str {
    CHANGELOG
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_changelog() {
        let changelog = get_changelog();
        assert!(changelog.starts_with("# Changelog\n\n## v"));
    }
}
