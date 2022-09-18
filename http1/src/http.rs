use std::{borrow::Cow, collections::BTreeMap};

use crate::parser::{self, ParseError};


pub enum Scheme {
    HTTP,
    HTTPS,
}

pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    Unknown(String),
}

pub struct Uri {

}

pub enum Version {
    V1_0,
    V1_1,
    V2,
}

pub struct HeaderMap {
    map: BTreeMap<String, Vec<u8>>
}



pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub header_map: HeaderMap,
    
}
