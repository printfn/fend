const CHANGELOG: &'static str = include_str!("../CHANGELOG.md");

pub fn get_changelog() -> &'static str {
    CHANGELOG
}
