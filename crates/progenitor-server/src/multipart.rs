// Copyright 2026 Oxide Computer Company

//! Collected, typed multipart request parts.

use bytes::Bytes;
use indexmap::IndexMap;

use crate::{Multipart, Rejection};

/// Represents one uploaded file part, fully read into memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePart {
    /// The untrusted filename reported in the part's content disposition.
    pub filename: Option<String>,
    /// The part's media type, when present.
    pub content_type: Option<String>,
    /// The complete file contents.
    pub bytes: Bytes,
}

#[derive(Debug)]
struct RawPart {
    filename: Option<String>,
    content_type: Option<String>,
    bytes: Bytes,
}

/// Stores multipart parts in memory, grouped by their wire names.
///
/// Collection is subject to axum's configured
/// [`DefaultBodyLimit`](axum::extract::DefaultBodyLimit). Generated adapters
/// consume known fields through the typed accessors; unconsumed fields are
/// ignored for forward compatibility.
#[derive(Debug, Default)]
pub struct Parts(IndexMap<String, Vec<RawPart>>);

impl Parts {
    /// Collects every part from an extracted multipart request.
    ///
    /// # Errors
    ///
    /// Returns a rejection when the multipart stream or a part body is
    /// malformed.
    pub async fn collect(mut multipart: Multipart) -> Result<Self, Rejection> {
        let mut parts = IndexMap::<String, Vec<RawPart>>::new();
        loop {
            let field = multipart
                .0
                .next_field()
                .await
                .map_err(|error| invalid_multipart_body(error.status()))?;
            let Some(field) = field else {
                break;
            };
            let name = field
                .name()
                .ok_or_else(|| Rejection::invalid_part("multipart part is missing a name"))?
                .to_string();
            let filename = field.file_name().map(ToOwned::to_owned);
            let content_type = field.content_type().map(ToOwned::to_owned);
            let bytes = field
                .bytes()
                .await
                .map_err(|error| invalid_multipart_body(error.status()))?;
            parts.entry(name).or_default().push(RawPart {
                filename,
                content_type,
                bytes,
            });
        }
        Ok(Parts(parts))
    }

    /// Removes and parses the first required text part in wire order.
    ///
    /// Like [`required_header`](crate::required_header), the part name and
    /// target type are known to generated code, so parsing lives here rather
    /// than in per-operation generated blocks.
    ///
    /// # Errors
    ///
    /// Returns a rejection when the part is absent, is not valid UTF-8, or
    /// cannot be parsed as `T`.
    pub fn required_text<T>(&mut self, api_name: &str) -> Result<T, Rejection>
    where
        T: std::str::FromStr,
    {
        let part = self
            .take_one(api_name)
            .ok_or_else(|| missing_part(api_name))?;
        parse_text(&part.bytes, api_name)
    }

    /// Removes and parses the first optional text part in wire order.
    ///
    /// # Errors
    ///
    /// Returns a rejection when a present part is not valid UTF-8 or cannot
    /// be parsed as `T`.
    pub fn optional_text<T>(&mut self, api_name: &str) -> Result<Option<T>, Rejection>
    where
        T: std::str::FromStr,
    {
        self.take_one(api_name)
            .map(|part| parse_text(&part.bytes, api_name))
            .transpose()
    }

    /// Removes the first required file part in wire order.
    ///
    /// # Errors
    ///
    /// Returns a rejection when the part is absent.
    pub fn required_file(&mut self, api_name: &str) -> Result<FilePart, Rejection> {
        self.take_one(api_name)
            .map(FilePart::from)
            .ok_or_else(|| missing_part(api_name))
    }

    /// Removes the first optional file part in wire order.
    ///
    /// # Errors
    ///
    /// This operation is currently infallible after collection.
    pub fn optional_file(&mut self, api_name: &str) -> Result<Option<FilePart>, Rejection> {
        Ok(self.take_one(api_name).map(FilePart::from))
    }

    /// Removes all file parts with the given name in wire order.
    ///
    /// # Errors
    ///
    /// This operation is currently infallible after collection.
    pub fn repeated_file(&mut self, api_name: &str) -> Result<Vec<FilePart>, Rejection> {
        Ok(self
            .0
            .shift_remove(api_name)
            .unwrap_or_default()
            .into_iter()
            .map(FilePart::from)
            .collect())
    }

    fn take_one(&mut self, api_name: &str) -> Option<RawPart> {
        let parts = self.0.get_mut(api_name)?;
        let part = (!parts.is_empty()).then(|| parts.remove(0));
        if parts.is_empty() {
            self.0.shift_remove(api_name);
        }
        part
    }
}

impl From<RawPart> for FilePart {
    fn from(part: RawPart) -> Self {
        FilePart {
            filename: part.filename,
            content_type: part.content_type,
            bytes: part.bytes,
        }
    }
}

fn parse_text<T>(bytes: &Bytes, api_name: &str) -> Result<T, Rejection>
where
    T: std::str::FromStr,
{
    let text = std::str::from_utf8(bytes).map_err(|_| {
        Rejection::invalid_part(format!("multipart part `{api_name}` is not UTF-8"))
    })?;
    text.parse::<T>()
        .map_err(|_| Rejection::invalid_part(format!("multipart part `{api_name}` is malformed")))
}

fn missing_part(api_name: &str) -> Rejection {
    Rejection::missing_part(format!("missing required multipart part `{api_name}`"))
}

fn invalid_multipart_body(status: http::StatusCode) -> Rejection {
    Rejection::invalid_body(status, "invalid multipart body")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::extract::FromRequest;

    fn raw(bytes: impl Into<Bytes>) -> RawPart {
        RawPart {
            filename: None,
            content_type: None,
            bytes: bytes.into(),
        }
    }

    fn parts(values: impl IntoIterator<Item = (&'static str, RawPart)>) -> Parts {
        let mut parts = IndexMap::<String, Vec<RawPart>>::new();
        for (name, part) in values {
            parts.entry(name.to_string()).or_default().push(part);
        }
        Parts(parts)
    }

    #[test]
    fn required_and_optional_text_parts_are_decoded() {
        let mut parts = parts([("required", raw("one")), ("optional", raw("2"))]);

        assert_eq!(parts.required_text::<String>("required").unwrap(), "one");
        assert_eq!(parts.optional_text::<i32>("optional").unwrap(), Some(2));
        assert_eq!(parts.optional_text::<String>("absent").unwrap(), None);
    }

    #[test]
    fn unparseable_text_has_stable_kind_and_names_the_part() {
        let mut parts = parts([("rating", raw("excellent"))]);

        let rejection = parts.required_text::<i32>("rating").unwrap_err();

        assert_eq!(rejection.kind(), crate::RejectionKind::InvalidPart);
        assert_eq!(rejection.message(), "multipart part `rating` is malformed");
    }

    #[test]
    fn missing_required_part_has_stable_kind_and_message() {
        let rejection = Parts::default().required_file("photo").unwrap_err();

        assert_eq!(rejection.kind(), crate::RejectionKind::MissingPart);
        assert_eq!(
            rejection.message(),
            "missing required multipart part `photo`"
        );
    }

    #[test]
    fn invalid_text_has_stable_kind_and_names_the_part() {
        let mut parts = parts([("note", raw(Bytes::from_static(&[0xff])))]);

        let rejection = parts.required_text::<String>("note").unwrap_err();

        assert_eq!(rejection.kind(), crate::RejectionKind::InvalidPart);
        assert_eq!(rejection.message(), "multipart part `note` is not UTF-8");
    }

    #[test]
    fn repeated_files_preserve_wire_order() {
        let mut parts = parts([("photo", raw("one")), ("photo", raw("two"))]);

        let files = parts.repeated_file("photo").unwrap();

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].bytes, "one");
        assert_eq!(files[1].bytes, "two");
    }

    #[test]
    fn unknown_parts_do_not_affect_known_accessors() {
        let mut parts = parts([("future", raw("ignored")), ("note", raw("known"))]);

        assert_eq!(parts.required_text::<String>("note").unwrap(), "known");
    }

    #[tokio::test]
    async fn collection_preserves_file_metadata_and_bytes() {
        let boundary = "progenitor-boundary";
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"photo\"; filename=\"pet.png\"\r\n\
             Content-Type: image/png\r\n\
             \r\n\
             png bytes\r\n\
             --{boundary}--\r\n"
        );
        let request = http::Request::builder()
            .header(
                http::header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();
        let multipart = Multipart::from_request(request, &()).await.unwrap();
        let mut parts = Parts::collect(multipart).await.unwrap();

        let file = parts.required_file("photo").unwrap();

        assert_eq!(file.filename.as_deref(), Some("pet.png"));
        assert_eq!(file.content_type.as_deref(), Some("image/png"));
        assert_eq!(file.bytes, "png bytes");
    }

    #[tokio::test]
    async fn extractor_maps_missing_boundary_to_runtime_rejection() {
        let request = http::Request::builder()
            .header(http::header::CONTENT_TYPE, "multipart/form-data")
            .body(Body::empty())
            .unwrap();

        let rejection = Multipart::from_request(request, &()).await.unwrap_err();

        assert_eq!(rejection.kind(), crate::RejectionKind::InvalidBody);
        assert_eq!(rejection.status(), http::StatusCode::BAD_REQUEST);
        assert_eq!(rejection.message(), "invalid multipart body");
    }
}
