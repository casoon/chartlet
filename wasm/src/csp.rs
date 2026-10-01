//! Content-Security-Policy hashes of the inline `<style>` elements a chart carries.
//!
//! A page with a strict `style-src` allows each inline style by the SHA-256 of its text. The
//! host cannot compute that synchronously in every runtime (`crypto.subtle` is asynchronous), so
//! the renderer hands the source expressions back with the content. base64 is written out
//! here rather than pulled in as a dependency, and SHA-256 comes from chartlet itself; both are
//! small and fixed by their standards.

use chartlet::sha256;

/// The `'sha256-…'` source expression of every distinct `<style>` element in `content`, in the
/// order they first appear. chartlet writes its styles as bare `<style>` tags with nothing in
/// their text that a parser would decode, so the hash covers the text exactly as written.
pub fn style_hashes(content: &str) -> Vec<String> {
    let mut hashes: Vec<String> = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find("<style>") {
        rest = &rest[start + "<style>".len()..];
        let Some(end) = rest.find("</style>") else {
            break;
        };
        let hash = format!("'sha256-{}'", base64(&sha256(&rest.as_bytes()[..end])));
        if !hashes.contains(&hash) {
            hashes.push(hash);
        }
        rest = &rest[end..];
    }
    hashes
}

/// Standard base64 with padding, as CSP source expressions use it.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let triple = chunk
            .iter()
            .enumerate()
            .fold(0u32, |triple, (index, byte)| {
                triple | u32::from(*byte) << (16 - 8 * index)
            });
        for index in 0..4 {
            if index <= chunk.len() {
                encoded.push(char::from(
                    ALPHABET[(triple >> (18 - 6 * index)) as usize & 63],
                ));
            } else {
                encoded.push('=');
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{base64, sha256, style_hashes};

    #[test]
    fn base64_pads_like_the_standard() {
        for (input, expected) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(input.as_bytes()), expected);
        }
    }

    #[test]
    fn hashes_every_distinct_style_in_order() {
        let content =
            "<div><style>b{}</style><svg><style>abc</style></svg><style>b{}</style></div>";
        assert_eq!(
            style_hashes(content),
            [
                format!("'sha256-{}'", base64(&sha256(b"b{}"))),
                "'sha256-ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0='".to_owned(),
            ]
        );
        assert_eq!(style_hashes("<svg></svg>"), Vec::<String>::new());
    }
}
