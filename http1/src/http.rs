use std::{collections::BTreeMap, fmt};

use bstr::{BStr, BString, ByteSlice};

use crate::error::Error;
use crate::parser::{ParseError, RawHeader, RawRequest};

pub mod headers {
    use bstr::BStr;



    pub const CONTENT_LENGTH: &[u8] = b"Content-Length";
    pub const TRANSFER_ENCODING: &[u8] = b"Transfer-Encoding";

    pub const CHUNKED: &[u8] = b"chunked";
}

pub enum Scheme {
    HTTP,
    HTTPS,
}

#[derive(Debug)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    Unknown(BString),
}

#[derive(Debug)]
pub struct Uri {
    raw: BString,
}

#[derive(Debug)]
pub enum Version {
    V1_0,
    V1_1,
    V2,
}

/// HeaderMap, use titiled case name as key, a vec of header to
/// hold raw header name.
pub struct HeaderMap(BTreeMap<BString, Vec<Header>>);

impl HeaderMap {
    pub fn new() -> Self {
        HeaderMap(BTreeMap::new())
    }

    pub fn get(&self, name: &[u8]) -> Option<&[Header]> {
        let key = title_case(name);

        self.0.get(&key).map(|v| v.as_ref())
    }

    pub fn set(&mut self, name: &[u8], value: &[u8]) {
        let key = title_case(name);
        let header = Header::new(name, value);

        self.0.insert(key, vec![header]);
    }

    pub fn append(&mut self, name: &[u8], value: &[u8]) {
        let key = title_case(name);
        let header = Header::new(name, value);

        self.0.entry(key).or_insert_with(|| Vec::new()).push(header);
    }

    pub fn remove(&mut self, name: &BStr) {
        let key = title_case(name);

        self.0.remove(&key);
    }
}

impl fmt::Debug for HeaderMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|(ref k, ref v)| (*k, *v)))
            .finish()
    }
}

impl From<RawHeader<'_>> for Header {
    fn from(header: RawHeader<'_>) -> Self {
        Header::new(header.name, header.value)
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub name: BString,
    pub value: BString,
}

impl Header {
    pub fn new<B>(name: B, value: B) -> Self
    where
        B: Into<BString>,
    {
        Header {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug)]
pub enum ContentLength {
    Sized(u64),
    Chunked,
    None,
}

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub header_map: HeaderMap,

    pub content_length: ContentLength,
}

impl Request {
    // pub fn new(url: &str) -> Self {
    //     Request { method: Method::GET, uri: (), version: (), header_map: (), content_length: () }
    // }

    pub fn from_raw_request(req: RawRequest<'_>) -> Result<Self, Error> {
        let method = match req.method {
            b"GET" => Method::GET,
            b"HEAD" => Method::HEAD,
            b"POST" => Method::POST,
            b"PUT" => Method::PUT,
            b"DELETE" => Method::DELETE,
            b"CONNECT" => Method::CONNECT,
            b"OPTIONS" => Method::OPTIONS,
            b"TRACE" => Method::TRACE,
            _ => Method::Unknown(req.method.into()),
        };

        let uri = Uri {
            raw: req.uri.into(),
        };
        let version = match req.version {
            b"1.0" => Version::V1_0,
            b"1.1" => Version::V1_1,
            _ => Version::V1_1,
        };

        let mut content_length = ContentLength::None;
        let mut header_map = HeaderMap::new();

        let mut had_transfer_encoding = false;
        for h in req.headers {
            header_map.append(h.name, h.value);

            if h.name.eq_ignore_ascii_case(headers::TRANSFER_ENCODING) {
                had_transfer_encoding = true;
                for part in h.value.split_str(",") {
                    if part.trim().eq_ignore_ascii_case(headers::CHUNKED) {
                        content_length = ContentLength::Chunked;
                    }
                }
            } else if h.name.eq_ignore_ascii_case(headers::CONTENT_LENGTH) {
                if had_transfer_encoding {
                    return Err(ParseError::BadRequest.into());
                }

                // content-length must be digit
                for d in h.value {
                    if !d.is_ascii_digit() {
                        return Err(ParseError::BadRequest.into())
                    }
                }
                match String::from_utf8_lossy(h.value).parse::<u64>() {
                    Ok(len) => {
                        content_length = ContentLength::Sized(len);
                    }
                    Err(_err) => {
                        return Err(ParseError::BadRequest.into())
                    }
                }
            }
        }

        Ok(Request {
            method,
            uri,
            version,
            header_map,
            content_length,
        })
    }
}

impl TryFrom<RawRequest<'_>> for Request {
    type Error = Error;

    fn try_from(req: RawRequest<'_>) -> Result<Self, Self::Error> {
        Self::from_raw_request(req)
    }
}

pub struct Response {
    pub status_code: u16,
    pub header_map: HeaderMap,
}

impl Response {
    pub fn new() -> Self {
        Response {
            status_code: 200,
            header_map: HeaderMap::new(),
        }
    }
}

fn title_case(s: &[u8]) -> BString {
    let mut ret = BString::new(Vec::with_capacity(s.len()));
    let mut upper = true;

    for c in s.iter() {
        if upper {
            ret.push(c.to_ascii_uppercase());
        } else {
            ret.push(*c);
        }

        upper = *c == b'-';
    }

    ret
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_title_case() {
        assert_eq!("Host", title_case(BStr::new("host")));
        assert_eq!("Host", title_case(BStr::new("Host")));
        assert_eq!("X-Forwarded-For", title_case(BStr::new("x-forwarded-for")));
        assert_eq!("X-Forwarded-For", title_case(BStr::new("X-Forwarded-For")));
        assert_eq!("Via", title_case(BStr::new("via")));
    }
}
