use std::{borrow::Cow, collections::BTreeMap, fmt};

use bstr::{BStr, BString, ByteSlice};

use crate::parser::{self, ParseError, RawRequest};

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
        f.debug_map().entries(self.0.iter().map(|(ref k, ref v)| (*k, *v))).finish()
    }
}


impl From<parser::RawHeader<'_>> for Header {
    fn from(header: parser::RawHeader<'_>) -> Self {
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
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub header_map: HeaderMap,
}

impl From<RawRequest<'_>> for Request {
    fn from(req: RawRequest<'_>) -> Self {
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

        let mut header_map = HeaderMap::new();

        for h in req.headers {
            header_map.append(h.name, h.value);
        }

        Request {
            method,
            uri,
            version,
            header_map,
        }
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
