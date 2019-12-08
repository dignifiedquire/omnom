use omnom::prelude::*;
use std::collections::HashMap;
use std::io::{BufRead, Cursor};

fn main() {
    assert_eq!(
        parse_mime("text/html").unwrap(),
        Mime {
            base_type: "text".to_string(),
            sub_type: "html".to_string(),
            parameters: None,
        }
    );

    let mut parameters = HashMap::new();
    parameters.insert("charset".to_string(), "utf-8".to_string());
    assert_eq!(
        parse_mime("text/html; charset=utf-8;").unwrap(),
        Mime {
            base_type: "text".to_string(),
            sub_type: "html".to_string(),
            parameters: Some(parameters),
        }
    );
}

#[derive(Eq, PartialEq, Debug)]
pub struct Mime {
    base_type: String,
    sub_type: String,
    parameters: Option<HashMap<String, String>>,
}

fn take1<R: BufRead>(mut s: R) -> Option<u8> {
    let mut token = vec![0; 1];
    s.read_exact(&mut token).ok()?;

    Some(token[0])
}

fn take_str_while<R: BufRead, P>(mut s: R, predicate: P) -> Option<String>
where
    P: FnMut(u8) -> bool,
{
    let mut val = vec![];
    s.read_while(&mut val, predicate).ok()?;
    validate_code_points(&val)?;
    let mut val = String::from_utf8(val).ok()?;
    val.make_ascii_lowercase();

    Some(val)
}

fn take_str_until<R: BufRead>(mut s: R, delim: u8) -> Option<String> {
    let mut val = vec![];
    match s.read_until(delim, &mut val).ok()? {
        0 => return None,
        _ => {
            if let Some(last) = val.last() {
                if *last == delim {
                    val.pop();
                }
            }
        }
    };

    validate_code_points(&val)?;

    String::from_utf8(val).ok()
}

fn parse_mime(s: &str) -> Option<Mime> {
    // parse the "type"
    //
    // ```txt
    // text/html; charset=utf-8;
    // ^^^^^
    // ```
    let mut s = Cursor::new(s);
    let base_type = take_str_until(&mut s, b'/')?;

    // parse the "subtype"
    //
    // ```txt
    // text/html; charset=utf-8;
    //      ^^^^^
    // ```
    let sub_type = take_str_until(&mut s, b';')?;

    // parse parameters into a hashmap
    //
    // ```txt
    // text/html; charset=utf-8;
    //           ^^^^^^^^^^^^^^^
    // ```

    let mut parameters = None;
    loop {
        // Stop parsing if there's no more bytes to consume.
        if s.fill_buf().unwrap().len() == 0 {
            break;
        }

        // Trim any whitespace.
        //
        // ```txt
        // text/html; charset=utf-8;
        //           ^
        // ```
        s.skip_while(is_http_whitespace_char).ok()?;

        // Get the param name.
        //
        // ```txt
        // text/html; charset=utf-8;
        //            ^^^^^^^
        // ```
        let param_name = take_str_while(&mut s, |b| b != b';' && b != b'=')?;

        // Ignore param names without values.
        //
        // ```txt
        // text/html; charset=utf-8;
        //                   ^
        // ```
        let token = take1(&mut s)?;
        if token == b';' {
            continue;
        }

        // Get the param value.
        //
        // ```txt
        // text/html; charset=utf-8;
        //                    ^^^^^^
        // ```
        let param_value = take_str_until(&mut s, b';')?;

        // Insert attribute pair into hashmap.
        if let None = parameters {
            parameters = Some(HashMap::new());
        }
        parameters.as_mut()?.insert(param_name, param_value);
    }

    // Construct the actual Mime struct.

    Some(Mime {
        base_type,
        sub_type,
        parameters,
    })
}

fn validate_code_points(buf: &[u8]) -> Option<()> {
    let all = buf.iter().all(|b| match b {
        b'-' | b'!' | b'#' | b'$' | b'%' => true,
        b'&' | b'\'' | b'*' | b'+' | b'.' => true,
        b'^' | b'_' | b'`' | b'|' | b'~' => true,
        b'A'..=b'Z' => true,
        b'a'..=b'z' => true,
        b'0'..=b'9' => true,
        _ => false,
    });

    if all {
        Some(())
    } else {
        None
    }
}

fn is_http_whitespace_char(b: u8) -> bool {
    match b {
        b' ' | b'\t' | b'\n' | b'\r' => true,
        _ => false,
    }
}
