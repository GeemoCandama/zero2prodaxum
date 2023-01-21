use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<Self, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_characters = ['<', '>', '"', '`', '(', ')', '{', '}', '\\', '/'];
        let contains_forbidden_characters = s.chars().any(|c| forbidden_characters.contains(&c));
        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err("invalid subscriber name".to_string())
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;

    #[test]
    fn a_256_grapheme_name_is_valid() {
        let name = SubscriberName::parse("a".repeat(256));
        assert!(name.is_ok());
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_invalid() {
        let name = SubscriberName::parse("a".repeat(257));
        assert!(name.is_err());
    }

    #[test]
    fn whitespace_only_names_are_invalid() {
        let name = SubscriberName::parse(" ".into());
        assert!(name.is_err());
    }

    #[test]
    fn empty_names_are_invalid() {
        let name = SubscriberName::parse("".into());
        assert!(name.is_err());
    }

    #[test]
    fn names_with_forbidden_characters_are_invalid() {
        for name in &["<", ">", "\"", "`", "(", ")", "{", "}", "\\", "/"] {
            let name = SubscriberName::parse(name.to_string());
            assert!(name.is_err());
        }
    }

    #[test]
    fn valid_names_are_valid() {
        let name = SubscriberName::parse("Ursula Le Guin".to_string());
        assert!(name.is_ok());
    }
}
