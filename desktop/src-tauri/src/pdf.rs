//! Authenticate PDF user passwords without modifying source documents.

use lopdf::{encryption::DecryptionError, Document, Error};

/// Distinguish a wrong password from broken or unsupported encryption.
pub fn authenticate(document: &Document, password: &str) -> Result<bool, String> {
    match document.authenticate_raw_user_password(password.as_bytes()) {
        Ok(()) => Ok(true),
        Err(Error::Decryption(DecryptionError::IncorrectPassword)) => Ok(false),
        Err(error) => Err(format!("Cannot authenticate this PDF: {error}")),
    }
}

/// A malformed encryption reference must not be treated as an unlocked PDF.
pub fn protected(document: &Document) -> Result<bool, String> {
    if !document.trailer.has(b"Encrypt") {
        return Ok(false);
    }
    document
        .get_encrypted()
        .map_err(|e| format!("Invalid PDF encryption: {e}"))?;
    authenticate(document, "").map(|accepted| !accepted)
}

#[cfg(test)]
/// In-memory PDF encryption checks using a known password.
mod tests {
    use super::*;
    use lopdf::{EncryptionState, EncryptionVersion, Object, Permissions};

    #[test]
    /// Accept 000008, reject 000007, and leave the document encrypted.
    fn authenticates_real_encryption_without_decrypting_the_document() {
        let mut doc = Document::new();
        doc.trailer.set(
            "ID",
            Object::Array(vec![
                Object::string_literal("fixture-id"),
                Object::string_literal("fixture-id"),
            ]),
        );
        let user_password = format!("{:06}", 8);
        let owner_password = format!("fixture-owner-{user_password}");
        let state = EncryptionState::try_from(EncryptionVersion::V1 {
            document: &doc,
            owner_password: &owner_password,
            user_password: &user_password,
            permissions: Permissions::all(),
        })
        .unwrap();
        doc.encrypt(&state).unwrap();
        assert_eq!(protected(&doc), Ok(true));
        assert_eq!(authenticate(&doc, "000007"), Ok(false));
        assert_eq!(authenticate(&doc, "000008"), Ok(true));
        assert!(doc.is_encrypted());
    }

    #[test]
    /// A missing encryption object cannot be called an unlocked document.
    fn malformed_encryption_is_an_error() {
        let mut doc = Document::new();
        doc.trailer.set("Encrypt", Object::Reference((999, 0)));
        assert!(protected(&doc).is_err());
    }

    #[test]
    /// A document without an encryption entry needs no password.
    fn plain_document_is_unprotected() {
        assert_eq!(protected(&Document::new()), Ok(false));
    }
}
