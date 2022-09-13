use crate::http::{Header, RawRequest, RawResponse};

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

pub fn parse_request<'a>(buf: &'a [u8], req: &mut RawRequest<'a>) -> Result<usize, ParseError> {
    let mut input = buf;

    // skip first empty line (some clients add CRLF after POST content)
    if input.len() < 2 {
        return Err(ParseError::Incomplete);
    }
    if &input[..2] == b"\r\n" {
        input = &input[2..];
    }

    input = parse_request_line(input, req)?;

    input = parse_headers(input, &mut req.headers)?;

    Ok(buf.len() - input.len())
}

fn parse_request_line<'a>(buf: &'a [u8], req: &mut RawRequest<'a>) -> Result<&'a [u8], ParseError> {
    let mut input = buf;

    // get method until space
    let (input, method) = find_and_skip_byte(input, b' ')?;
    req.method = method;

    let (input, uri) = find_and_skip_byte(input, b' ')?;
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

fn parse_headers<'a>(buf: &'a [u8], headers: &mut Vec<Header<'a>>) -> Result<&'a [u8], ParseError> {
    let mut input = buf;

    loop {
        let (i, name) = find_and_skip_byte(input, b':')?;
        let (i, _sp) = take_till(i, is_ws)?;
        let (i, value) = find_and_skip_2bytes(i, [b'\r', b'\n'])?;

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
    let mut input = buf;

    input = parse_status_line(input, rsp)?;

    input = parse_headers(input, &mut rsp.headers)?;

    Ok(buf.len() - input.len())
}

fn parse_status_line<'a>(buf: &'a [u8], rsp: &mut RawResponse<'a>) -> Result<&'a [u8], ParseError> {
    let input = buf;

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
    rsp.version = version;
    let input = &input[3..];

    let (input, sp) = tagb(input, b' ')?;

    let (input, status) = ensure_n(input, 3, is_digit)?;
    rsp.status_code = status;

    let (input, sp) = tagb(input, b' ')?;

    // get reason until line end
    let (input, reason) = find_and_skip_2bytes(input, [b'\r', b'\n'])?;
    rsp.reason = reason;

    Ok(input)
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
            Ok((&buf[p + 1..], &buf[..p]))},
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

fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | 0x0B | 0x0C)
}

fn is_digit(b: u8) -> bool {
    matches!(b, b'0'..=b'9')
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

        println!(
            "ret: {:?}, rsp {:?}",
            ret, &rsp
        );

        println!(
            "body: {:?}, req {:?}",
            String::from_utf8_lossy(&buf[ret.unwrap()..]),
            &rsp
        );
    }
}
