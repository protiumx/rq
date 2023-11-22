use bytes::Bytes;
use encoding_rs::{Encoding, UTF_8};

pub async fn decode_with_encoding(
    bytes: Bytes,
    encoding_name: &str,
) -> (String, &'static Encoding) {
    let encoding = Encoding::for_label(encoding_name.as_bytes()).unwrap_or(UTF_8);

    let (text, encoding, _) = encoding.decode(&bytes);
    (text.into_owned(), encoding)
}
