use bytes::{Bytes, BytesMut};

const BYTE_SP: u8 = b' ';
const BYTE_CR: u8 = b'\r';
const BYTE_LF: u8 = b'\n';
const BYTE_NUL: u8 = 0x00;
const BYTE_COLON: u8 = b':';
const BYTES_CRLF: [u8; 2] = [b'\r', b'\n'];

// rfc9110, 5.6.2
// tchar = "!" / "#" / "$" / "%" / "&" / "'" / "*" / "+" / "-" / "." /
// "^" / "_" / "`" / "|" / "~" / DIGIT / ALPHA
const TCHAR_TABLE: [bool; 127] = [
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, true, false, true, true, true, true, true,
    false, false, true, true, false, true, true, false, true, true, true, true, true, true, true,
    true, true, true, false, false, false, false, false, false, false, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, false, false, false, true, true, true, true, true, true,
    true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
    true, true, true, true, true, true, true, false, true, false, true,
];

const DEFAULT_HEADER_COUNT: usize = 16;

pub struct Header<'a> {
    name: &'a [u8],
    value: &'a [u8],
}

impl<'a> Header<'a> {
    pub fn new(name: &'a [u8], value: &'a [u8]) -> Self {
        Header { name, value }
    }
}

impl<'a> std::fmt::Debug for Header<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Header")
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseStatus {
    Completed(usize),
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    Incomplete,
    BadRequest,
    BadResponse,
    UnsupportMethod,
    BadData,
    BadHeaderName,
}

pub fn parse_request<'a>(buf: &'a [u8], req: &mut RawRequest<'a>) -> Result<usize, ParseError> {
    let mut input = buf;

    // skip first empty line (some clients add CRLF after POST content)
    if input.len() < 2 {
        return Err(ParseError::Incomplete);
    }
    if &input[..2] == BYTES_CRLF {
        input = &input[2..];
    }

    let (input, line) = read_line(input)?;

    parse_request_line(line, req)?;

    let input = parse_headers(input, &mut req.headers)?;

    Ok(buf.len() - input.len())
}

fn parse_request_line<'a>(buf: &'a [u8], req: &mut RawRequest<'a>) -> Result<(), ParseError> {
    let input = buf;

    // get method until space
    let (input, method) = must_split(input, BYTE_SP)?;
    req.method = method;

    let (input, uri) = must_split(input, BYTE_SP)?;
    req.uri = uri;

    // parse version
    req.version = parse_http_version(input)?;

    Ok(())
}

fn parse_headers<'a>(buf: &'a [u8], headers: &mut Vec<Header<'a>>) -> Result<&'a [u8], ParseError> {
    let mut input = buf;

    loop {
        let (i, line) = read_line(input)?;

        let (value, name) =
            validate_until(line, BYTE_COLON, |b| b < 127 && TCHAR_TABLE[b as usize])?;

        // validate header value, reject bad data
        // a recipient of CR, LF, or NUL within a field value 
        // MUST either reject the message or replace each of those characters with SP 
        // before further processing or forwarding of that message. 
        memchr::memchr3(BYTE_CR, BYTE_LF, BYTE_NUL, value).ok_or(ParseError::BadData)?;

        let value = trim_ows(value);

        headers.push(Header::new(name, value));

        if i.len() < 2 {
            return Err(ParseError::Incomplete);
        }
        if &i[..2] == b"\r\n" {
            input = &i[2..];
            break;
        }

        input = i;
    }

    Ok(input)
}

pub fn parse_response<'a>(buf: &'a [u8], rsp: &mut RawResponse<'a>) -> Result<usize, ParseError> {
    let (input, line) = read_line(buf)?;

    parse_status_line(line, rsp)?;

    let input = parse_headers(input, &mut rsp.headers)?;

    Ok(buf.len() - input.len())
}

fn parse_status_line<'a>(buf: &'a [u8], rsp: &mut RawResponse<'a>) -> Result<(), ParseError> {
    let (input, version) = must_split(buf, BYTE_SP)?;

    rsp.version = parse_http_version(version)?;

    let (reason, code) = must_split(input, BYTE_SP)?;

    if code.len() != 3 {
        return Err(ParseError::BadResponse);
    }

    for b in code {
        if !is_digit(*b) {
            return Err(ParseError::BadResponse);
        }
    }

    rsp.status_code = code;

    // get reason until line end
    rsp.reason = reason;

    Ok(())
}

fn parse_http_version(input: &[u8]) -> Result<&[u8], ParseError> {
    // parse version
    if matches!(input, b"HTTP/1.1" | b"HTTP/1.0") {
        return Ok(&input[5..]);
    }

    if !input.starts_with(b"HTTP/") {
        return Err(ParseError::BadRequest);
    }

    let version = &input[5..];
    if version[0] < 48
        || version[0] > 57
        || version[1] != b'.'
        || version[2] < 48
        || version[2] > 57
    {
        return Err(ParseError::BadRequest);
    }

    Ok(version)
}

fn read_line(buf: &[u8]) -> Result<(&[u8], &[u8]), ParseError> {
    for p in memchr::memchr_iter(b'\n', buf) {
        if p > 0 && buf[p - 1] == b'\r' {
            return Ok((&buf[p + 1..], &buf[..p - 1]));
        }
    }

    Err(ParseError::Incomplete)
}

fn must_split(buf: &[u8], pat: u8) -> Result<(&[u8], &[u8]), ParseError> {
    match memchr::memchr(pat, buf) {
        Some(p) => Ok((&buf[p + 1..], &buf[..p])),
        None => Err(ParseError::BadData),
    }
}

fn find_and_skip_byte<'a, 'b>(
    buf: &'a [u8],
    needle: u8,
) -> Result<(&'a [u8], &'a [u8]), ParseError> {
    match memchr::memchr(needle, buf) {
        Some(p) => {
            if p + 1 == buf.len() {
                return Err(ParseError::Incomplete);
            }
            Ok((&buf[p + 1..], &buf[..p]))
        }
        None => Err(ParseError::Incomplete),
    }
}

fn find_and_skip_2bytes<'a, 'b>(
    buf: &'a [u8],
    needle: [u8; 2],
) -> Result<(&'a [u8], &'a [u8]), ParseError> {
    for p in memchr::memchr_iter(needle[0], buf) {
        if buf[p + 1] == needle[1] {
            return Ok((&buf[p + 2..], &buf[..p]));
        }
    }

    Err(ParseError::Incomplete)
}

// OWS rfc9110 5.6.3
fn is_whitespace(b: u8) -> bool {
    matches!(b, BYTE_SP | b'\t')
}

fn is_digit(b: u8) -> bool {
    matches!(b, b'0'..=b'9')
}

fn trim_ows(input: &[u8]) -> &[u8] {
    let mut input = input;

    for (i, b) in input.iter().enumerate() {
        if !is_whitespace(*b) {
            input = &input[i..];
            break;
        }
    }

    for (i, b) in input.iter().rev().enumerate() {
        if !is_whitespace(*b) {
            return &input[..input.len() - i];
        }
    }

    input
}

fn tag<'a, 'b>(input: &'a [u8], tag: &'b [u8]) -> Result<(&'a [u8], &'a [u8]), ParseError> {
    if input.len() < tag.len() {
        return Err(ParseError::Incomplete);
    }

    if input[..tag.len()] == tag[..] {
        return Ok((&input[tag.len()..], &input[..tag.len()]));
    }

    Err(ParseError::BadRequest)
}

fn tagb(input: &[u8], tag: u8) -> Result<(&[u8], u8), ParseError> {
    if input.len() < 1 {
        return Err(ParseError::Incomplete);
    }

    if input[0] == tag {
        return Ok((&input[1..], input[0]));
    }

    Err(ParseError::BadRequest)
}

fn ensure_n<F>(input: &[u8], n: usize, cond: F) -> Result<(&[u8], &[u8]), ParseError>
where
    F: Fn(u8) -> bool,
{
    if input.len() < n {
        return Err(ParseError::Incomplete);
    }

    for b in &input[..n] {
        if !cond(*b) {
            return Err(ParseError::BadRequest);
        }
    }

    return Ok((&input[n..], &input[..n]));
}

fn validate_until<F>(input: &[u8], end: u8, cond: F) -> Result<(&[u8], &[u8]), ParseError>
where
    F: Fn(u8) -> bool,
{
    for (i, b) in input.iter().enumerate() {
        if *b == end {
            return Ok((&input[i + 1..], &input[..i]));
        }
        if !cond(*b) {
            return Err(ParseError::BadData);
        }
    }

    Err(ParseError::Incomplete)
}

fn take_until<F>(input: &[u8], cond: F) -> Result<(&[u8], &[u8]), ParseError>
where
    F: Fn(u8) -> bool,
{
    for (i, b) in input.iter().enumerate() {
        if cond(*b) {
            return Ok((&input[i..], &input[..i]));
        }
    }

    Err(ParseError::Incomplete)
}

fn take_till<F>(input: &[u8], cond: F) -> Result<(&[u8], &[u8]), ParseError>
where
    F: Fn(u8) -> bool,
{
    let mut offset = 0;

    for (i, b) in input.iter().enumerate() {
        offset = i;
        if !cond(*b) {
            return Ok((&input[i..], &input[..i]));
        }
    }

    return Ok((&input[offset..], &input[..offset]));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_request() {
        let buf = b"POST /index.html HTTP/1.1\r\nHost: www.baidu.com\r\nContent-Length: 0\r\nConnection: Close\r\n\r\nbad data";
        let buf: &[u8] = b"\
GET /wp-content/uploads/2010/03/hello-kitty-darth-vader-pink.jpg HTTP/1.1\r\n\
Host: www.kittyhell.com\r\n\
User-Agent: Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10.6; ja-JP-mac; rv:1.9.2.3) Gecko/20100401 Firefox/3.6.3 Pathtraq/0.9\r\n\
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
Accept-Language: ja,en-us;q=0.7,en;q=0.3\r\n\
Accept-Encoding: gzip,deflate\r\n\
Accept-Charset: Shift_JIS,utf-8;q=0.7,*;q=0.7\r\n\
Keep-Alive: 115\r\n\
Connection: keep-alive\r\n\
Cookie: wp_ozh_wsa_visits=2; wp_ozh_wsa_visit_lasttime=xxxxxxxxxx; __utma=xxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.x; __utmz=xxxxxxxxx.xxxxxxxxxx.x.x.utmccn=(referral)|utmcsr=reader.livedoor.com|utmcct=/reader/|utmcmd=referral|padding=under256\r\n\r\n";

        let mut req = RawRequest::new();

        let ret = parse_request(buf, &mut req);

        println!(
            "body: {:?}, req {:?}",
            String::from_utf8_lossy(&buf[ret.unwrap()..]),
            &req
        );
    }

    #[test]
    fn test_parse_response() {
        let buf = b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: Close\r\n\r\nbad data";

        let mut rsp = RawResponse::new();

        let ret = parse_response(buf, &mut rsp);

        println!("ret: {:?}, rsp {:?}", ret, &rsp);

        println!(
            "body: {:?}, req {:?}",
            String::from_utf8_lossy(&buf[ret.unwrap()..]),
            &rsp
        );
    }

    #[test]
    fn print_tchar_table() {
        print!("[");

        for b in 0..127 {
            if "!#$%&'*+-.^_`|~".as_bytes().contains(&b) || b.is_ascii_alphanumeric() {
                print!("true, ");
            } else {
                print!("false, ");
            }
            if b % 10 == 9 {
                println!("");
            }
        }

        println!("]")
    }
}

// "!#$%&'*+-.^_`|~" / DIGIT / ALPHA
