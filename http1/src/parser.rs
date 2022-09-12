use std::borrow::Cow;

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
            headers: Vec::new(),
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseStatus {
    Completed(usize),
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    Incomplete,
    BadRequest,
    UnsupportMethod,
}

impl<'a> RawRequest<'a> {
    pub fn parse(&mut self, buf: &'a [u8]) -> Result<usize, ParseError> {
        let mut input = buf;

        input = skip_first_crlf(input)?;

        input = parse_request_line(input, self)?;

        input = parse_headers(input, &mut self.headers)?;

        Ok(buf.len() - input.len())
    }

    pub fn parse2(&mut self, buf: &'a [u8]) -> Result<usize, ParseError> {
        let mut input = buf;

        input = parse_request_line2(input, self)?;

        input = parse_headers2(input, &mut self.headers)?;

        Ok(buf.len() - input.len())
    }
}

fn parse_request_line2<'a>(
    buf: &'a [u8],
    req: &mut RawRequest<'a>,
) -> Result<&'a [u8], ParseError> {
    let mut input = buf;

    // get method until space
    let (input, method) = find_and_skip(input, b" ")?;
    req.method = method;

    let (input, uri) = find_and_skip(input, b" ")?;
    req.uri = uri;

    // parse version
    let (input, h) = tag(input, b"HTTP/")?;
    if input.len() < 3 {
        return Err(ParseError::Incomplete);
    }

    let version = &input[..3];
    if version[0] < 48
        || version[0] > 57
        || version[1] != b'.'
        || version[2] < 48
        || version[2] > 57
    {
        return Err(ParseError::BadRequest);
    }
    req.version = version;
    let input = &input[3..];

    let (input, crlf) = tag(input, b"\r\n")?;

    Ok(input)
}

fn parse_headers2<'a>(
    buf: &'a [u8],
    headers: &mut Vec<Header<'a>>,
) -> Result<&'a [u8], ParseError> {
    let mut input = buf;

    loop {
        let (i, name) = find_and_skip(input, b":")?;
        let (i, _sp) = take_till(i, |b| b == b':' || b == b' ')?;
        let (i, value) = find_and_skip(i, b"\r\n")?;
        // let value = ltrim(value);
        headers.push(Header {
            ty: HeaderType::Keep,
            name: Cow::Borrowed(name),
            value: Cow::Borrowed(value),
        });

        if i.len() < 2 {
            return Err(ParseError::Incomplete)
        } 
        if &i[..2] == b"\r\n" {
            input = &i[2..];
            break;
        }

        input = i;
    }

    Ok(input)
}

fn find_and_skip<'a, 'b>(buf: &'a [u8], pat: &'b [u8]) -> Result<(&'a[u8], &'a [u8]), ParseError> {
    match twoway::find_bytes(buf, pat) {
        Some(p) => {
            Ok((&buf[p+pat.len()..], &buf[..p]))
        }
        None => {
            Err(ParseError::Incomplete)
        }
    }
}


fn skip_first_crlf(input: &[u8]) -> Result<&[u8], ParseError> {
    // skip first empty line (some clients add CRLF after POST content)
    if input.len() < 1 {
        return Err(ParseError::Incomplete);
    }
    if input[0] == b'\r' {
        let (input, crlf) = tag(input, b"\r\n")?;
        return Ok(input);
    }

    Ok(input)
}

fn parse_request_line<'a>(
    input: &'a [u8],
    req: &mut RawRequest<'a>,
) -> Result<&'a [u8], ParseError> {
    // get method until space
    let (input, method) = take_until(input, |b| b == b' ')?;
    req.method = method;

    let (input, sp) = take_till(input, |b| b == b' ')?;

    // get uri until space
    let (input, uri) = take_until(input, |b| b == b' ')?;
    let (input, sp) = take_till(input, |b| b == b' ')?;
    req.uri = uri;

    // parse version
    let (input, h) = tag(input, b"HTTP/")?;
    if input.len() < 3 {
        return Err(ParseError::Incomplete);
    }

    let version = &input[..3];
    if version[0] < 48
        || version[0] > 57
        || version[1] != b'.'
        || version[2] < 48
        || version[2] > 57
    {
        return Err(ParseError::BadRequest);
    }
    req.version = version;

    let (input, crlf) = tag(&input[3..], b"\r\n")?;

    Ok(input)
}

fn parse_headers<'a>(buf: &'a [u8], headers: &mut Vec<Header<'a>>) -> Result<&'a [u8], ParseError> {
    let mut input = buf;

    loop {
        if input.len() < 2 {
            return Err(ParseError::Incomplete);
        }
        // found CRLF finished
        if &input[..2] == b"\r\n" {
            return Ok(&input[2..]);
        }

        // take header name before colon
        let (i, name) = take_until(input, |b| b == b':')?;

        let (i, _sp) = take_till(i, |b| b == b':' || b == b' ')?;

        let (i, value) = take_until(i, |b| b == b'\r')?;
        let value = ltrim(value);

        headers.push(Header {
            ty: HeaderType::Keep,
            name: Cow::Borrowed(name),
            value: Cow::Borrowed(value),
        });

        let (i, lf) = tag(i, b"\r\n")?;

        input = i;
    }
}

fn ltrim(input: &[u8]) -> &[u8] {
    for (i, b) in input.iter().rev().enumerate() {
        if *b != b' ' {
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

        let ret = req.parse2(buf);

        println!(
            "ret: {:?}, req {:?}",
            String::from_utf8_lossy(&buf[ret.unwrap()..]),
            &req
        );
    }
}
