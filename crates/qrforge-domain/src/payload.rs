use url::Url;

/// A parsed URL that is safe for the browser-opening port.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafeHttpUrl {
    normalized: String,
    identity: String,
}

impl SafeHttpUrl {
    /// Returns the normalized URL supplied to the operating system.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.normalized
    }

    /// Returns a fragment-free normalized identity suitable for deduplication.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }
}

/// Security classification for untrusted decoded payload bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PayloadClass {
    /// A validated HTTP or HTTPS URL.
    SafeUrl(SafeHttpUrl),
    /// Valid UTF-8 that is not syntactically a URL.
    PlainText(String),
    /// A syntactically valid URL with a scheme QRForge never launches.
    BlockedScheme {
        /// Lowercase parsed scheme.
        scheme: String,
        /// Original text, retained only for optional clipboard copying.
        text: String,
    },
    /// Non-UTF-8 payload bytes. These are preserved by the detection but not
    /// coerced onto a text clipboard.
    Binary,
}

/// Classifies untrusted QR payload bytes without executing or dereferencing them.
#[must_use]
pub fn classify_payload(payload: &[u8]) -> PayloadClass {
    let Ok(text) = std::str::from_utf8(payload) else {
        return PayloadClass::Binary;
    };
    if text.is_empty() || text.trim() != text || text.bytes().any(|byte| byte.is_ascii_control()) {
        return PayloadClass::PlainText(text.to_owned());
    }

    let Ok(mut parsed) = Url::parse(text) else {
        return PayloadClass::PlainText(text.to_owned());
    };
    let scheme = parsed.scheme().to_ascii_lowercase();
    if !matches!(scheme.as_str(), "http" | "https") {
        return PayloadClass::BlockedScheme {
            scheme,
            text: text.to_owned(),
        };
    }
    let authority = text
        .split_once(':')
        .map(|(_, remainder)| remainder)
        .unwrap_or_default();
    if !authority.starts_with("//")
        || authority[2..].starts_with(['/', '\\'])
        || parsed.cannot_be_a_base()
        || parsed.host_str().is_none()
    {
        return PayloadClass::PlainText(text.to_owned());
    }

    let normalized = parsed.to_string();
    parsed.set_fragment(None);
    PayloadClass::SafeUrl(SafeHttpUrl {
        normalized,
        identity: parsed.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_well_formed_http_urls() {
        let PayloadClass::SafeUrl(url) = classify_payload(b"HTTPS://Example.COM:443/a#section")
        else {
            panic!("expected safe URL");
        };
        assert_eq!(url.as_str(), "https://example.com/a#section");
        assert_eq!(url.identity(), "https://example.com/a");
        assert!(matches!(
            classify_payload(b"http:///missing-host"),
            PayloadClass::PlainText(_)
        ));
    }

    #[test]
    fn blocks_every_non_http_scheme() {
        for value in [
            "javascript:alert(1)",
            "data:text/html,hello",
            "file:///C:/secret.txt",
            "mailto:user@example.com",
            "custom:action",
            "C:\\Windows\\System32\\cmd.exe",
        ] {
            assert!(
                matches!(
                    classify_payload(value.as_bytes()),
                    PayloadClass::BlockedScheme { .. }
                ),
                "{value} should be blocked"
            );
        }
    }

    #[test]
    fn preserves_plain_text_and_binary_classification() {
        assert_eq!(
            classify_payload(b"hello from QRForge"),
            PayloadClass::PlainText("hello from QRForge".to_owned())
        );
        assert_eq!(classify_payload(&[0xff, 0xfe]), PayloadClass::Binary);
    }

    #[test]
    fn does_not_allow_url_parser_whitespace_normalization() {
        assert!(matches!(
            classify_payload(b" https://example.com"),
            PayloadClass::PlainText(_)
        ));
    }
}
