use std::{collections::BTreeMap, fmt};

use bstr::{BStr, BString, ByteSlice};
use bytes::{BufMut, Bytes, BytesMut};

use crate::body::Body;
use crate::error::Error;
use crate::parser::{ParseError, RawHeader, RawRequest};

pub mod headers {
    pub const CONTENT_LENGTH: &[u8] = b"Content-Length";
    pub const TRANSFER_ENCODING: &[u8] = b"Transfer-Encoding";
    pub const CONNECTION: &[u8] = b"Connection";

    pub const CHUNKED: &[u8] = b"chunked";
    pub const CLOSE: &[u8] = b"close";
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
    Sized(usize),
    Chunked,
    None,
    Close,
}

#[derive(Debug)]
pub(crate) struct RequestInfo {
    pub content_length: ContentLength,
    pub should_close: bool,
}

impl RequestInfo {
    pub fn new() -> Self {
        RequestInfo { content_length: ContentLength::None, should_close: false }
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub header_map: HeaderMap,
    pub body: Body,
}

impl Request {
    // pub fn new(url: &str) -> Self {
    //     Request { method: Method::GET, uri: (), version: (), header_map: (), content_length: () }
    // }

   pub(crate) fn from_raw_request(req: RawRequest<'_>, info: &mut RequestInfo) -> Result<Self, Error> {
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

        let mut should_close = false;
        let mut content_length = ContentLength::None;
        let mut header_map = HeaderMap::new();

        let mut had_transfer_encoding = false;
        for h in req.headers {
            header_map.append(h.name, h.value);

            if h.name.eq_ignore_ascii_case(headers::TRANSFER_ENCODING) {
                had_transfer_encoding = true;
                if header_values_contains_token(h.value, headers::CHUNKED) {
                    content_length = ContentLength::Chunked;
                }
            } else if h.name.eq_ignore_ascii_case(headers::CONTENT_LENGTH) {
                if had_transfer_encoding {
                    return Err(ParseError::BadRequest.into());
                }

                // content-length must be digit
                for d in h.value {
                    if !d.is_ascii_digit() {
                        return Err(ParseError::BadRequest.into());
                    }
                }
                match String::from_utf8_lossy(h.value).parse::<usize>() {
                    Ok(len) => {
                        info.content_length = ContentLength::Sized(len);
                    }
                    Err(_err) => return Err(ParseError::BadRequest.into()),
                }
            } else if h.name.eq_ignore_ascii_case(headers::CONNECTION) {
                if header_values_contains_token(h.value, headers::CLOSE) {
                    info.should_close = true;
                    info.content_length = ContentLength::Close;
                }
            }
        }

        Ok(Request {
            method,
            uri,
            version,
            header_map,
            body: Body::empty(),
        })
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

    pub fn header_buf(self) -> Bytes {
        let mut buf = BytesMut::with_capacity(1024);

        self.put_status_line(&mut buf);

        for (_, values) in self.header_map.0 {
            for v in values {
                buf.put_slice(&v.name);
                buf.put_slice(b": ");
                buf.put_slice(&v.value);
                buf.put_slice(b"\r\n");
            }
        }

        buf.put_slice(b"\r\n");

        // println!("=> {:?}", String::from_utf8_lossy(&buf));

        buf.freeze()
    }

    fn put_status_line(&self, buf: &mut BytesMut) {
        buf.put_slice(b"HTTP/1.1 ");
        buf.put_slice(self.status_code.to_string().as_bytes());
        buf.put_slice(b" ");

        let s: &'static [u8] = match self.status_code {
            200 => b"OK",
            400 => b"Bad Request",
            404 => b"Not Found",
            500 => b"Internal Server Error",
            _ => unimplemented!(),
        };

        buf.put_slice(s);

        buf.put_slice(b"\r\n");
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

fn header_values_contains_token(values: &[u8], token: &[u8]) -> bool {
    for part in values.split_str(",") {
        if part.trim().eq_ignore_ascii_case(token) {
            return true;
        }
    }

    false
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
