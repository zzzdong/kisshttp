use std::borrow::Cow;

use crate::parser::{self, ParseError};

const DEFAULT_HEADER_COUNT: usize = 16;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HeaderType {
    Keep,
    Add,
    Del,
}

pub struct Header<'a> {
    ty: HeaderType,
    name: Cow<'a, [u8]>,
    value: Cow<'a, [u8]>,
}

impl<'a> Header<'a> {
    pub fn new(name: &'a [u8], value: &'a [u8]) -> Self {
        Header {
            ty: HeaderType::Keep,
            name: Cow::Borrowed(name),
            value: Cow::Borrowed(value),
        }
    }
}

impl<'a> std::fmt::Debug for Header<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Header")
            .field("type", &self.ty)
            .field("name", &String::from_utf8_lossy(&self.name))
            .field("value", &String::from_utf8_lossy(&self.value))
            .finish()
    }
}

pub struct RawRequest<'a> {
    pub method: &'a [u8],
    pub uri: &'a [u8],
    pub version: &'a [u8],
    pub headers: Vec<Header<'a>>,
}

impl<'a> RawRequest<'a> {
    pub fn new() -> Self {
        RawRequest {
            method: &[],
            uri: &[],
            version: &[],
            headers: Vec::with_capacity(DEFAULT_HEADER_COUNT),
        }
    }

    pub fn parse(&mut self, buf: &'a [u8]) -> Result<usize, ParseError> {
        parser::parse_request(buf, self)
    }

    pub fn add_header_name(&mut self, name: &'a [u8]) {
        self.headers.push(Header {
            ty: HeaderType::Keep,
            name: Cow::Borrowed(name),
            value: Cow::Borrowed(&[]),
        })
    }

    pub fn add_header_value(&mut self, value: &'a [u8]) {
        self.headers.last_mut().unwrap().value = Cow::Borrowed(value);
    }

    pub fn set_header(&mut self, key: &str, value: Vec<u8>) {
        self.remove_header(key);
        self.add_header(key, value);
    }

    pub fn add_header(&mut self, key: &str, value: Vec<u8>) {
        // add new one
        self.headers.push(Header {
            ty: HeaderType::Add,
            name: Cow::Owned(key.as_bytes().to_vec()),
            value: Cow::Owned(value.to_vec()),
        })
    }

    pub fn remove_header(&mut self, key: &str) {
        for h in &mut self.headers {
            if h.ty != HeaderType::Del && &h.name[..] == key.as_bytes() {
                h.ty = HeaderType::Del;
            }
        }
    }

    pub fn get_header(&self, key: &str) -> Vec<Cow<'a, [u8]>> {
        self.headers
            .iter()
            .filter(|x| x.ty != HeaderType::Del && x.name == key.as_bytes())
            .map(|h| h.value.clone())
            .collect()
    }

    pub fn headers(&self) -> &'a [Header] {
        &self.headers
    }
}

impl<'a> std::fmt::Debug for RawRequest<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawRequest")
            .field("method", &String::from_utf8_lossy(self.method))
            .field("uri", &String::from_utf8_lossy(self.uri))
            .field("version", &String::from_utf8_lossy(self.version))
            .field("headers", &self.headers)
            .finish()
    }
}

pub struct RawResponse<'a> {
    pub status_code: &'a [u8],
    pub reason: &'a [u8],
    pub version: &'a [u8],
    pub headers: Vec<Header<'a>>,
}

impl<'a> RawResponse<'a> {
    pub fn new() -> Self {
        RawResponse {
            status_code: &[],
            reason: &[],
            version: &[],
            headers: Vec::with_capacity(DEFAULT_HEADER_COUNT),
        }
    }

    pub fn parse(&mut self, buf: &'a [u8]) -> Result<usize, ParseError> {
        parser::parse_response(buf, self)
    }

    pub fn add_header_name(&mut self, name: &'a [u8]) {
        self.headers.push(Header {
            ty: HeaderType::Keep,
            name: Cow::Borrowed(name),
            value: Cow::Borrowed(&[]),
        })
    }

    pub fn add_header_value(&mut self, value: &'a [u8]) {
        self.headers.last_mut().unwrap().value = Cow::Borrowed(value);
    }

    pub fn set_header(&mut self, key: &str, value: Vec<u8>) {
        self.remove_header(key);
        self.add_header(key, value);
    }

    pub fn add_header(&mut self, key: &str, value: Vec<u8>) {
        // add new one
        self.headers.push(Header {
            ty: HeaderType::Add,
            name: Cow::Owned(key.as_bytes().to_vec()),
            value: Cow::Owned(value.to_vec()),
        })
    }

    pub fn remove_header(&mut self, key: &str) {
        for h in &mut self.headers {
            if h.ty != HeaderType::Del && &h.name[..] == key.as_bytes() {
                h.ty = HeaderType::Del;
            }
        }
    }

    pub fn get_header(&self, key: &str) -> Vec<Cow<'a, [u8]>> {
        self.headers
            .iter()
            .filter(|x| x.ty != HeaderType::Del && x.name == key.as_bytes())
            .map(|h| h.value.clone())
            .collect()
    }

    pub fn headers(&self) -> &'a [Header] {
        &self.headers
    }
}

impl<'a> std::fmt::Debug for RawResponse<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawRequest")
            .field("status_code", &String::from_utf8_lossy(self.status_code))
            .field("reason", &String::from_utf8_lossy(self.reason))
            .field("version", &String::from_utf8_lossy(self.version))
            .field("headers", &self.headers)
            .finish()
    }
}

