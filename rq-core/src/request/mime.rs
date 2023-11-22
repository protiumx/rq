use bytes::Bytes;
use mime::Mime;
use reqwest::{header::CONTENT_TYPE, Response};

use super::decode::decode_with_encoding;

#[derive(Debug, Clone)]
pub struct BytePayload {
    pub extension: Option<String>,
    pub bytes: Bytes,
}

#[derive(Debug, Clone)]
pub struct TextPayload {
    pub charset: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum Payload {
    Bytes(BytePayload),
    Text(TextPayload),
}

impl Payload {
    pub async fn of_response(response: Response) -> Payload {
        let mime = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<Mime>().ok());

        match mime {
            Some(mime) => match (mime.type_(), mime.subtype()) {
                (mime::TEXT, _) => {
                    let charset = mime
                        .get_param("charset")
                        .map(|charset| charset.to_string())
                        .unwrap_or("utf-8".into());
                    let (text, encoding) =
                        decode_with_encoding(response.bytes().await.unwrap(), &charset).await;
                    Payload::Text(TextPayload {
                        charset: encoding.name().to_owned(),
                        text,
                    })
                }
                _ => Payload::Bytes(BytePayload {
                    extension: None,
                    bytes: response.bytes().await.unwrap(),
                }),
            },
            None => Payload::Bytes(BytePayload {
                extension: None,
                bytes: response.bytes().await.unwrap(),
            }),
        }
    }
}

// impl Display for Payload {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Payload::Bytes(_) => write!(f, "raw bytes"),
//             Payload::Text(t) => write!(f, "decoded with encoding: '{}'\n{}", t.charset, t.text),
//         }
//     }
// }
