use std::borrow::Cow;

static REQ_NFA_STATES: &'static [StaticState; 23] = {
    &[
        StaticState {
            index: 0,
            bytes: ByteClass::Any,
            next_states: &[1],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 1,
            bytes: ByteClass::Invalid(&[32, 9, 11, 12]),
            next_states: &[1, 2],
            start_capture: CaptureAction::StartMethod,
            end_capture: CaptureAction::EndMethod,
            result: StateResult::Ok,
        },
        StaticState {
            index: 2,
            bytes: ByteClass::Valid(&[32, 9, 11, 12]),
            next_states: &[2, 3],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 3,
            bytes: ByteClass::Invalid(&[32, 9, 11, 12]),
            next_states: &[3, 4],
            start_capture: CaptureAction::StartRequestTaget,
            end_capture: CaptureAction::EndRequestTaget,
            result: StateResult::Ok,
        },
        StaticState {
            index: 4,
            bytes: ByteClass::Valid(&[32, 9, 11, 12]),
            next_states: &[4, 5],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 5,
            bytes: ByteClass::Valid(&[72]),
            next_states: &[6],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 6,
            bytes: ByteClass::Valid(&[84]),
            next_states: &[7],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 7,
            bytes: ByteClass::Valid(&[84]),
            next_states: &[8],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 8,
            bytes: ByteClass::Valid(&[80]),
            next_states: &[9],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 9,
            bytes: ByteClass::Valid(&[47]),
            next_states: &[10],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 10,
            bytes: ByteClass::Invalid(&[13]),
            next_states: &[10, 11],
            start_capture: CaptureAction::StartHttpVersion,
            end_capture: CaptureAction::EndHttpVersion,
            result: StateResult::Ok,
        },
        StaticState {
            index: 11,
            bytes: ByteClass::Valid(&[13]),
            next_states: &[12],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 12,
            bytes: ByteClass::Valid(&[10]),
            next_states: &[13],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        // StaticState {
        //     index: 13,
        //     bytes: ByteClass::Valid(&[
        //         48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 97, 98, 99, 100, 101, 102, 103, 104,
        //         105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120,
        //         121, 122, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82,
        //         83, 84, 85, 86, 87, 88, 89, 90, 45,
        //     ]),
        //     next_states: &[13, 14, 15],
        //     start_capture: CaptureAction::StartHeaderName,
        //     end_capture: CaptureAction::EndHeaderName,
        //     result: StateResult::Ok,
        // },
        StaticState {
            index: 13,
            bytes: ByteClass::ValidRange(&[(48, 57), (97, 122), (65, 90), (45, 45)]),
            next_states: &[13, 14, 15],
            start_capture: CaptureAction::StartHeaderName,
            end_capture: CaptureAction::EndHeaderName,
            result: StateResult::Ok,
        },
        StaticState {
            index: 14,
            bytes: ByteClass::Valid(&[32, 9, 11, 12]),
            next_states: &[],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Err(ParseError::BadRequest),
        },
        StaticState {
            index: 15,
            bytes: ByteClass::Valid(&[58]),
            next_states: &[16, 17],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 16,
            bytes: ByteClass::Valid(&[32, 9, 11, 12]),
            next_states: &[16, 17],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 17,
            bytes: ByteClass::Invalid(&[13]),
            next_states: &[17, 18, 19],
            start_capture: CaptureAction::StartHeaderValue,
            end_capture: CaptureAction::EndHeaderValue,
            result: StateResult::Ok,
        },
        StaticState {
            index: 18,
            bytes: ByteClass::Valid(&[32, 9, 11, 12]),
            next_states: &[18, 19],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 19,
            bytes: ByteClass::Valid(&[13]),
            next_states: &[20],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 20,
            bytes: ByteClass::Valid(&[10]),
            next_states: &[13, 21],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 21,
            bytes: ByteClass::Valid(&[13]),
            next_states: &[22],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Ok,
        },
        StaticState {
            index: 22,
            bytes: ByteClass::Valid(&[10]),
            next_states: &[],
            start_capture: CaptureAction::None,
            end_capture: CaptureAction::None,
            result: StateResult::Done,
        },
    ]
};




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
    Incompleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    BadRequest,
    UnsupportMethod,
    ErrState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ByteClass {
    Valid(&'static [u8]),
    ValidRange(&'static [(u8, u8)]),
    Invalid(&'static [u8]),
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CaptureAction {
    None,
    StartMethod,
    EndMethod,
    StartRequestTaget,
    EndRequestTaget,
    StartHttpVersion,
    EndHttpVersion,
    StartHeaderName,
    EndHeaderName,
    StartHeaderValue,
    EndHeaderValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StateResult {
    Ok,
    Done,
    Err(ParseError),
}

#[derive(Debug, Clone)]
struct StaticState {
    pub index: usize,
    pub bytes: ByteClass,
    pub next_states: &'static [usize],
    pub start_capture: CaptureAction,
    pub end_capture: CaptureAction,
    pub result: StateResult,
}

impl StaticState {
    pub fn matches(&self, b: &u8) -> bool {
        match &self.bytes {
            ByteClass::Valid(bs) => bs.contains(b),
            ByteClass::ValidRange(range) => {
                for r in *range {
                    if *b >= r.0 && *b <= r.1 {
                        return true;
                    }
                }
                return false;
            }
            ByteClass::Invalid(bs) => !bs.contains(b),
            ByteClass::Any => true,
        }
    }
}

#[derive(Debug, Clone)]
struct StaticNFA {
    states: &'static [StaticState; 23],
    current_state: usize,
}

impl StaticNFA {
    pub fn get(&self, index: usize) -> &StaticState {
        unsafe { &self.states.get_unchecked(index) }
    }

    pub fn process<F>(
        &mut self,
        buf: &[u8],
        pos: usize,
        mut callback: F,
    ) -> Result<ParseStatus, ParseError>
    where
        F: FnMut(CaptureAction, usize),
    {
        let mut curr_state =unsafe { REQ_NFA_STATES.get_unchecked(self.current_state) };

        for (i, b) in buf[pos..].iter().enumerate() {
            for next in curr_state.next_states {
                let next_state = unsafe { REQ_NFA_STATES.get_unchecked(*next) };
                if next_state.matches(b) {
                    let offset = pos + i;

                    match next_state.result {
                        StateResult::Ok => {
                            // it's ok to go next state
                        }
                        StateResult::Done => {
                            return Ok(ParseStatus::Completed(offset + 1));
                        }
                        StateResult::Err(err) => {
                            return Err(err);
                        }
                    }

                    if curr_state.index != next_state.index {
                        let end = curr_state.end_capture.clone();
                        if end != CaptureAction::None {
                            callback(end, offset);
                        }

                        let start = next_state.start_capture.clone();
                        if start != CaptureAction::None {
                            callback(start, offset);
                        }
                    }

                    self.current_state = next_state.index;
                    curr_state = next_state;
                }
            }
        }

        Ok(ParseStatus::Incompleted)
    }

    /// process byte, return next state when found
    pub fn process_byte(&self, state: usize, b: &u8) -> Option<usize> {
        for s in self.states[state].next_states {
            if self.states[*s].matches(b) {
                return Some(*s);
            }
        }

        None
    }
}


pub struct Parser {
    nfa: StaticNFA,
    state: (CaptureAction, usize),
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            nfa: StaticNFA {
                states: REQ_NFA_STATES,
                current_state: 0,
            },
            state: (CaptureAction::None, 0),
        }
    }

    pub fn parse<'a>(
        &mut self,
        buf: &'a [u8],
        req: &mut RawRequest<'a>,
    ) -> Result<ParseStatus, ParseError> {
        let mut ret = Ok(ParseStatus::Incompleted);

        let mut callback = |c, pos| {
            let err_bad_state = Err(ParseError::ErrState);

            match c {
                CaptureAction::EndMethod => {
                    if self.state.0 != CaptureAction::StartMethod {
                        return err_bad_state;
                    }
                    req.method = &buf[self.state.1..pos];
                }
                CaptureAction::EndRequestTaget => {
                    if self.state.0 != CaptureAction::StartRequestTaget {
                        return err_bad_state;
                    }

                    req.uri = &buf[self.state.1..pos];
                }
                CaptureAction::EndHttpVersion => {
                    if self.state.0 != CaptureAction::StartHttpVersion {
                        return err_bad_state;
                    }

                    req.version = &buf[self.state.1..pos];
                }
                CaptureAction::EndHeaderName => {
                    if self.state.0 != CaptureAction::StartHeaderName {
                        return err_bad_state;
                    }

                    req.add_header_name(&buf[self.state.1..pos]);
                }
                CaptureAction::EndHeaderValue => {
                    if self.state.0 != CaptureAction::StartHeaderValue {
                        return err_bad_state;
                    }

                    req.add_header_value(&buf[self.state.1..pos]);
                }
                _ => {
                    self.state = (c, pos);
                }
            };

            return Ok(ParseStatus::Incompleted);
        };

        self.nfa.process(buf, 0, |c, pos| ret = callback(c, pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let buf = b"POST \t/index.html HTTP/1.1\r\nHost: www.baidu.com\r\nContent-Length: 0\r\nConnection: Close\r\n\r\nbad data";

        let buf = b"\
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

        let mut parser = Parser::new();
        let mut req = RawRequest::new();

        let ret = parser.parse(buf, &mut req);

        println!("=> {:?} {:?}", ret, req);
    }
}

mod gen_nfa {
    use super::{ByteClass, CaptureAction, ParseError, ParseStatus, StateResult};

    const HEADER_NAME_BYTES: &'static [u8] =
        b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-";
    const WHITE_SPACE_BYTES: &'static [u8] = &[b' ', b'\t', 0x0B, 0x0C];
    const CR_BYTES: &'static [u8] = &[b'\r'];
    const LF_BYTES: &'static [u8] = &[b'\n'];
    const COLON_BYTES: &'static [u8] = &[b':'];

    #[derive(Debug, Clone)]
    struct State {
        pub index: usize,
        pub bytes: ByteClass,
        pub next_states: Vec<usize>,
        pub start_capture: CaptureAction,
        pub end_capture: CaptureAction,
        pub result: StateResult,
    }

    impl State {
        pub fn new(index: usize, bytes: ByteClass) -> State {
            State {
                index,
                bytes,
                next_states: Vec::new(),
                start_capture: CaptureAction::None,
                end_capture: CaptureAction::None,
                result: StateResult::Ok,
            }
        }

        pub fn matches(&self, b: &u8) -> bool {
            match &self.bytes {
                ByteClass::Valid(bs) => bs.contains(b),
                ByteClass::Invalid(bs) => !bs.contains(b),
                ByteClass::ValidRange(range) => {
                    for r in *range {
                        if *b >= r.0 && *b <= r.1 {
                            return true;
                        }
                    }
                    return false;
                }
                ByteClass::Any => true,
            }
        }
    }

    #[derive(Debug, Clone)]
    struct NFA {
        states: Vec<State>,
        current_state: usize,
    }

    impl NFA {
        pub fn new() -> NFA {
            let root = State::new(0, ByteClass::Any);

            NFA {
                states: vec![root],
                current_state: 0,
            }
        }

        pub fn get(&self, index: usize) -> &State {
            &self.states[index]
        }

        pub fn get_mut(&mut self, index: usize) -> &mut State {
            &mut self.states[index]
        }

        fn put(&mut self, index: usize, bytes: ByteClass) -> usize {
            let state = self.get(index);
            for &index in &state.next_states {
                let state = self.get(index);
                if state.bytes == bytes {
                    return index;
                }
            }

            let state = self.new_state(bytes);
            self.get_mut(index).next_states.push(state);
            state
        }

        fn new_state(&mut self, bytes: ByteClass) -> usize {
            let index = self.states.len();
            let state = State::new(index, bytes);
            self.states.push(state);

            index
        }

        fn put_state(&mut self, index: usize, child: usize) {
            if !self.states[index].next_states.contains(&child) {
                self.get_mut(index).next_states.push(child);
            }
        }

        fn start_capture(&mut self, index: usize, capture: CaptureAction) {
            self.get_mut(index).start_capture = capture;
        }

        fn end_capture(&mut self, index: usize, capture: CaptureAction) {
            self.get_mut(index).end_capture = capture;
        }

        fn state_result(&mut self, index: usize, result: StateResult) {
            self.get_mut(index).result = result;
        }

        pub fn process<F>(
            &mut self,
            buf: &[u8],
            pos: usize,
            mut callback: F,
        ) -> Result<ParseStatus, ParseError>
        where
            F: FnMut(CaptureAction, usize),
        {
            for (i, b) in buf[pos..].iter().enumerate() {
                match self.process_byte(self.current_state, b) {
                    Some(next) => {
                        let offset = pos + i;

                        match self.get(next).result {
                            StateResult::Ok => {
                                // it's ok to go next state
                            }
                            StateResult::Done => {
                                return Ok(ParseStatus::Completed(offset + 1));
                            }
                            StateResult::Err(err) => {
                                return Err(err);
                            }
                        }

                        if self.current_state != next {
                            let end = self.get(self.current_state).end_capture.clone();
                            if end != CaptureAction::None {
                                callback(end, offset);
                            }

                            let start = self.get(next).start_capture.clone();
                            if start != CaptureAction::None {
                                callback(start, offset);
                            }
                        }

                        self.current_state = next;
                    }
                    None => return Err(ParseError::BadRequest),
                }
            }

            Ok(ParseStatus::Incompleted)
        }

        /// process byte, return next state when found
        pub fn process_byte(&self, state: usize, b: &u8) -> Option<usize> {
            for s in &self.states[state].next_states {
                if self.states[*s].matches(b) {
                    return Some(*s);
                }
            }

            None
        }
    }

    #[test]
    fn test_nfa() {
        let mut nfa = NFA::new();

        let method = nfa.put(0, ByteClass::Invalid(WHITE_SPACE_BYTES));
        nfa.start_capture(method, CaptureAction::StartMethod);
        nfa.put_state(method, method);
        nfa.end_capture(method, CaptureAction::EndMethod);

        // eat sp
        let sp = nfa.put(method, ByteClass::Valid(WHITE_SPACE_BYTES));
        nfa.put_state(sp, sp);

        let uri = nfa.put(sp, ByteClass::Invalid(WHITE_SPACE_BYTES));
        nfa.start_capture(uri, CaptureAction::StartRequestTaget);
        nfa.put_state(uri, uri);
        nfa.end_capture(uri, CaptureAction::EndRequestTaget);

        let sp = nfa.put(uri, ByteClass::Valid(WHITE_SPACE_BYTES));
        nfa.put_state(sp, sp);

        // HTTP/
        let h = nfa.put(sp, ByteClass::Valid(&[b'H']));
        let h = nfa.put(h, ByteClass::Valid(&[b'T']));
        let h = nfa.put(h, ByteClass::Valid(&[b'T']));
        let h = nfa.put(h, ByteClass::Valid(&[b'P']));
        let h = nfa.put(h, ByteClass::Valid(&[b'/']));

        let version = nfa.put(h, ByteClass::Invalid(CR_BYTES));
        nfa.start_capture(version, CaptureAction::StartHttpVersion);
        nfa.put_state(version, version);
        nfa.end_capture(version, CaptureAction::EndHttpVersion);

        let cr = nfa.put(version, ByteClass::Valid(CR_BYTES));
        let lf = nfa.put(cr, ByteClass::Valid(LF_BYTES));

        let header_name = nfa.put(lf, ByteClass::Valid(HEADER_NAME_BYTES));
        nfa.start_capture(header_name, CaptureAction::StartHeaderName);
        nfa.put_state(header_name, header_name);
        nfa.end_capture(header_name, CaptureAction::EndHeaderName);

        let err = nfa.put(header_name, ByteClass::Valid(WHITE_SPACE_BYTES));
        nfa.state_result(err, StateResult::Err(ParseError::BadRequest));

        let colon = nfa.put(header_name, ByteClass::Valid(COLON_BYTES));

        let sp = nfa.put(colon, ByteClass::Valid(WHITE_SPACE_BYTES));
        nfa.put_state(sp, sp);

        let header_value = nfa.put(colon, ByteClass::Invalid(CR_BYTES));
        nfa.put_state(sp, header_value);

        nfa.start_capture(header_value, CaptureAction::StartHeaderValue);
        nfa.put_state(header_value, header_value);
        nfa.end_capture(header_value, CaptureAction::EndHeaderValue);

        let sp = nfa.put(header_value, ByteClass::Valid(WHITE_SPACE_BYTES));
        nfa.put_state(sp, sp);

        let cr = nfa.put(sp, ByteClass::Valid(CR_BYTES));

        nfa.put_state(header_value, cr);

        let lf = nfa.put(cr, ByteClass::Valid(LF_BYTES));

        nfa.put_state(lf, header_name);

        let cr = nfa.put(lf, ByteClass::Valid(CR_BYTES));
        let lf = nfa.put(cr, ByteClass::Valid(LF_BYTES));
        nfa.state_result(lf, StateResult::Done);

        // println!("{:?}", nfa);

        let buf = b"POST \t/index.html HTTP/1.1\r\nHost: www.baidu.com\r\nContent-Length: 0\r\nConnection: Close\r\n\r\n";
        // let buf = b"GET ";

        let ret = nfa.process(&buf[..], 0, |capture, pos| {
            println!("--- {capture:?} - {pos}");
        });

        println!("ret:====> {:?}", ret);

        match ret {
            Ok(ParseStatus::Completed(off)) => {
                println!(
                    "Completed =======> {}",
                    String::from_utf8_lossy(&buf[off..])
                );
            }
            err => {
                println!("=======> {:?}", err);
            }
        }

        for s in nfa.states {
            println!("StaticState{{ index: {}, bytes: ByteClass::{:?}, next_states: &{:?}, start_capture: CaptureAction::{:?}, end_capture: CaptureAction::{:?}, result: StateResult::{:?} }},", s.index, s.bytes, s.next_states, s.start_capture, s.end_capture, s.result);
        }

        // println!("-> {:?}", nfa.get(ret.unwrap()));
    }
}
