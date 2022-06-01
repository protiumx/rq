use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::Parser;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Display;
use std::result::Result;
use std::slice::Iter;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct HttpParser;

#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl Default for HttpMethod {
    fn default() -> Self {
        HttpMethod::Get
    }
}

impl HttpMethod {
    pub fn iterator() -> Iter<'static, HttpMethod> {
        static METHODS: [HttpMethod; 4] = [
            HttpMethod::Get,
            HttpMethod::Post,
            HttpMethod::Put,
            HttpMethod::Delete,
        ];
        METHODS.iter()
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for HttpMethod {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        Ok(match pair.as_str() {
            "GET" => Self::Get,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "DELETE" => Self::Delete,
            _ => unreachable!(),
        })
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

#[derive(Debug, Clone, Default)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl<'i> TryFrom<Pair<'i, Rule>> for HttpRequest {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        let mut iterator = pair.into_inner();
        // {
        //  method target version
        //  headers
        //  body
        // }
        let mut ret = Self {
            method: iterator.next().unwrap().try_into()?,
            url: iterator.next().unwrap().as_str().to_string(),
            version: iterator.next().unwrap().as_str().to_string(),
            headers: HashMap::new(),
            body: String::new(),
        };

        for item in iterator {
            match item.as_rule() {
                Rule::headers => {
                    ret.parse_headers(item.into_inner());
                }
                Rule::body => {
                    ret.body = item.as_str().trim().to_string();
                }
                _ => {
                    unreachable!();
                }
            }
        }

        Ok(ret)
    }
}

impl HttpRequest {
    fn parse_headers(&mut self, pairs: Pairs<Rule>) {
        for item in pairs {
            let mut kv = item.into_inner();
            let key = kv.next().unwrap().as_str().to_string();
            let value = kv.next().unwrap().as_str().to_string();
            self.headers.insert(key, value);
        }
    }
}

impl Display for HttpRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} HTTP/{}", self.method, self.url, self.version)?;
        if !self.headers.is_empty() {
            f.write_str(" [")?;
            for (i, (k, v)) in self.headers.iter().enumerate() {
                write!(f, "{}: {}", k, v)?;
                if i != self.headers.len() - 1 {
                    f.write_str(", ")?;
                }
            }
            f.write_str("]")?
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct HttpFile {
    pub requests: Vec<HttpRequest>,
}

impl<'i> TryFrom<Pair<'i, Rule>> for HttpFile {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        let iterator = pair.into_inner();
        let mut requests = vec![];
        for item in iterator {
            match item.as_rule() {
                Rule::EOI => {
                    break;
                }
                Rule::request => {
                    requests.push(item.try_into()?);
                }
                _ => {}
            }
        }
        Ok(Self { requests })
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

pub fn parse(input: &str) -> Result<HttpFile, Error<Rule>> {
    let file = HttpParser::parse(Rule::file, input.trim_start())
        .expect("unable to parse")
        .next()
        .unwrap();
    HttpFile::try_from(file)
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
        for method in HttpMethod::iterator() {
            let input = format!("{} test.dev HTTP/1.1\n\n", method);
            let file = assert_parses(input.as_str());
            assert_eq!(file.requests.len(), 1);
            assert_eq!(
                file.requests[0].to_string(),
                format!("{} test.dev HTTP/1.1", method)
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
        assert_eq!(file.requests[0].headers.len(), 1);
        assert_eq!(
            file.requests[0].headers.get("authorization").unwrap(),
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
            "#0\nPOST test.dev HTTP/1 [authorization: token]\n#1\nGET test.dev HTTP/1\n"
        );
    }
}
