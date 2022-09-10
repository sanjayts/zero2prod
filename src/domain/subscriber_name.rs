use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    const FORBIDDEN_CHARS: [char; 9] = ['/', '\\', '(', ')', '{', '}', '<', '>', '"'];

    /// Returns an instance of SubscriberName if
    pub fn parse(name: String) -> Result<Self, String> {
        let is_empty_or_whitespace = name.trim().is_empty();

        let has_forbidden_chars = name
            .chars()
            .any(|c| SubscriberName::FORBIDDEN_CHARS.contains(&c));

        let is_too_long = name.graphemes(true).count() > 256;

        if is_empty_or_whitespace || has_forbidden_chars || is_too_long {
            Err(format!("Invalid subscriber name {}", name))
        } else {
            Ok(Self(name))
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
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "ते".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_257_grapheme_long_name_is_invalid() {
        let name = "ते".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn an_empty_name_is_invalid() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_with_forbidden_chars_is_valid() {
        for c in SubscriberName::FORBIDDEN_CHARS {
            let name = c.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn a_valid_name_should_parse_successfully() {
        let name = "Sanju Baba".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
