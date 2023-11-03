use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::Parser;

use std::collections::HashMap;
use std::fmt::Display;
use std::result::Result;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct HttpParser;

#[derive(Debug, Clone, Default)]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Delete,
}

impl<'i> From<Pair<'i, Rule>> for HttpMethod {
    fn from(pair: Pair<'i, Rule>) -> Self {
        match pair.as_str() {
            "GET" => Self::Get,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "DELETE" => Self::Delete,
            _ => unreachable!(),
        }
    }
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
        })
    }
}

#[derive(Clone, Debug, Default)]
struct HttpHeaders(HashMap<String, String>);

impl Display for HttpHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .0
            .iter()
            .map(|(key, value)| format!("{key}: {value}"))
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "[{s}]")
    }
}

impl<'i> From<Pairs<'i, Rule>> for HttpHeaders {
    fn from(pairs: Pairs<'i, Rule>) -> Self {
        let headers = pairs
            .map(|pair| {
                let mut kv = pair.into_inner();
                let key = kv.next().unwrap().as_str().to_string();
                let value = kv.next().unwrap().as_str().to_string();

                (key, value)
            })
            .collect();

        HttpHeaders(headers)
    }
}

#[derive(Debug, Clone, Default)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub version: String,
    headers: HttpHeaders,
    pub body: String,
}

impl HttpRequest {
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers.0
    }
}

impl<'i> From<Pair<'i, Rule>> for HttpRequest {
    fn from(request: Pair<'i, Rule>) -> Self {
        let mut pairs = request.into_inner().peekable();

        let method: HttpMethod = pairs
            .next_if(|pair| pair.as_rule() == Rule::method)
            .map(|pair| pair.into())
            .unwrap_or_default();

        let url = pairs.next().unwrap().as_str().to_string();
        let version = pairs
            .next_if(|pair| pair.as_rule() == Rule::version)
            .map(|pair| pair.as_str().to_string())
            .unwrap_or_default();

        let headers: HttpHeaders = pairs
            .next_if(|pair| pair.as_rule() == Rule::headers)
            .map(|pair| pair.into_inner().into())
            .unwrap_or_default();

        let body = pairs
            .next()
            .map(|pair| pair.as_str().to_string())
            .unwrap_or_default();

        Self {
            method,
            url,
            version,
            headers,
            body,
        }
    }
}

impl Display for HttpRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} HTTP/{} {}",
            self.method, self.url, self.version, self.headers
        )
    }
}

#[derive(Debug)]
pub struct HttpFile {
    pub requests: Vec<HttpRequest>,
}

impl<'i> From<Pair<'i, Rule>> for HttpFile {
    fn from(pair: Pair<Rule>) -> Self {
        let requests = pair
            .into_inner()
            .filter_map(|pair| match pair.as_rule() {
                Rule::request => Some(pair.into()),
                _ => None,
            })
            .collect();

        Self { requests }
    }
}

impl Display for HttpFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.requests.is_empty() {
            writeln!(f, "No requests found")?;
            return Ok(());
        }
        for (i, req) in self.requests.iter().enumerate() {
            write!(f, "#{}\n{}\n", i, req)?;
        }
        Ok(())
    }
}

pub fn parse(input: &str) -> Result<HttpFile, Box<Error<Rule>>> {
    let pair = HttpParser::parse(Rule::file, input.trim_start())?
        .next()
        .unwrap();
    Ok(HttpFile::from(pair))
}

#[cfg(test)]
mod tests {
    use super::{parse, HttpFile, HttpMethod};

    fn assert_parses(input: &str) -> HttpFile {
        let parsed = parse(input);
        assert!(parsed.is_ok());
        parsed.unwrap()
    }

    #[test]
    fn test_empty_input() {
        let file = assert_parses("");
        assert_eq!(file.to_string(), "No requests found\n");
    }

    #[test]
    fn test_http_methods() {
        const METHODS: [HttpMethod; 4] = [
            HttpMethod::Get,
            HttpMethod::Post,
            HttpMethod::Put,
            HttpMethod::Delete,
        ];
        for method in METHODS {
            let input = format!("{} test.dev HTTP/1.1\n\n", method);
            let file = assert_parses(input.as_str());
            assert_eq!(file.requests.len(), 1);
            assert_eq!(
                file.requests[0].to_string(),
                format!("{} test.dev HTTP/1.1 []", method)
            );
        }
    }

    #[test]
    fn test_http_headers() {
        let input = r#"
POST test.dev HTTP/1
authorization: Bearer xxxx

"#;
        let file = assert_parses(input);
        assert_eq!(file.requests[0].headers.0.len(), 1);
        assert_eq!(
            file.requests[0].headers.0.get("authorization").unwrap(),
            "Bearer xxxx"
        );
    }

    #[test]
    fn test_http_body() {
        let input = r#"
POST test.dev HTTP/1

{ "test": "body" }"#;
        let file = assert_parses(input);
        assert_eq!(file.requests[0].body, "{ \"test\": \"body\" }");
    }

    #[test]
    fn test_http_file() {
        let input = r#"
POST test.dev HTTP/1
authorization: token

###

GET test.dev HTTP/1

"#;
        let file = assert_parses(input);
        assert_eq!(file.requests.len(), 2);
        assert_eq!(
            file.to_string(),
            "#0\nPOST test.dev HTTP/1 [authorization: token]\n#1\nGET test.dev HTTP/1 []\n"
        );
    }
}
