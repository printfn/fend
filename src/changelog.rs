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
        for line in changelog.lines() {
            assert!(
                line == "# Changelog"
                    || line.is_empty()
                    || line.starts_with("## v")
                    || line.starts_with("* ")
                    || line == "Initial release:",
                "Invalid line format: {}",
                line
            );
        }
    }
}
