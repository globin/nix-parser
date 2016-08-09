#[macro_use]
extern crate nom;
extern crate regex;

use std::str;

use nom::{IResult, Needed};

static EASY_FILE: &'static str = r#"
{
  name = "promo";

  buildInputs = [
    nodejs
    env
  ];

  buildPhase = ''
    SECRET_KEY_BASE=tempfnord rake assets:precompile
    mv config config.dist
  '';

  installPhase = ''
    mkdir -p $out/share
    cp -r . $out/share/promo
    ln -sf /run/promo/config $out/share/promo/config
  '';
}
"#;

#[derive(Debug, PartialEq)]
pub enum Type {
    String(String),
    IndentedString(String),
    Identifier(String),
    AttrSet(AttrSet)
}

#[derive(Debug, PartialEq)]
pub struct AttrSet(Vec<Type>);

fn is_not_string_delim(i: &[u8]) -> IResult<&[u8], &[u8]> {
    let pos = i.len() - 1;

    if i[pos] == b'"' && &i[pos - 1..] != b"\"" {
        return IResult::Done(&i[pos..], &i[..pos])
    }

    IResult::Incomplete(Needed::Unknown)
}

named!(
    pub string<Type>,
    delimited!(
        char!('"'),
        map_res!(
            is_not_string_delim,
            |val| str::from_utf8(val).map(|s| Type::String(s.into()))
        ),
        char!('"')
    )
);

fn is_not_indented_string_delim(i: &[u8]) -> IResult<&[u8], &[u8]> {
    let pos = i.len() - 2;

    if &i[pos..] == b"''" && &i[pos - 1..] != b"'''" {
        return IResult::Done(&i[pos..], &i[..pos])
    }

    IResult::Incomplete(Needed::Unknown)
}

named!(
    pub indented_string<Type>,
    delimited!(
        tag!("''"),
        map_res!(
            is_not_indented_string_delim,
            |val| str::from_utf8(val).map(|s| Type::IndentedString(s.into()))
        ),
        tag!("''")
    )
);

named!(
    pub identifier<Type>,
    map_res!(
        re_bytes_find!("^([a-zA-Z_][a-zA-Z0-9_-]*)"),
        |val| str::from_utf8(val).map(|s| Type::Identifier(s.into()))
    )
);

named!(
    pub attr_set<Type>,
    chain!(
        char!('{') ~
        id: identifier ~
        char!('}'),
        || Type::AttrSet(AttrSet(vec!(id)))
    )
);

#[cfg(test)]
mod tests {
    use nom::{ErrorKind, IResult};
    use nom::Err::Code;
    use super::*;

    fn s(val: &str) -> Type {
        Type::String(val.into())
    }
    fn is(val: &str) -> Type {
        Type::IndentedString(val.into())
    }
    fn id(val: &str) -> Type {
        Type::Identifier(val.into())
    }
    fn a(val: &str) -> Type {
        Type::AttrSet(AttrSet(vec!(Type::Identifier(val.into()))))
    }

    #[test]
    fn string_parse() {
        assert_eq!(string(br#""test""#), IResult::Done(&b""[..], s("test")));
        assert_eq!(string(br#""test foo""#), IResult::Done(&b""[..], s("test foo")));
        assert_eq!(string(b"\"test\nfoo\""), IResult::Done(&b""[..], s("test\nfoo")));
        // assert_eq!(string(b"\"test\nfo\\\"o\""), IResult::Done(&b""[..], "test\nfo\"o"));
    }

    #[test]
    fn indented_string_parse() {
        assert_eq!(indented_string(b"''test''"), IResult::Done(&b""[..], is("test")));
        assert_eq!(indented_string(b"''test foo''"), IResult::Done(&b""[..], is("test foo")));
        assert_eq!(indented_string(b"''test\nfoo''"), IResult::Done(&b""[..], is("test\nfoo")));
        // assert_eq!(string(b"''test\nfo'''o''"), IResult::Done(&b""[..], "test\nfo''o"));
    }

    #[test]
    fn identifier_parse() {
        assert_eq!(identifier(b"test"), IResult::Done(&b""[..], id("test")));
        assert_eq!(identifier(b"test-foo"), IResult::Done(&b""[..], id("test-foo")));
        assert_eq!(identifier(b"_test-foo"), IResult::Done(&b""[..], id("_test-foo")));
        assert_eq!(identifier(b"_test9-foo"), IResult::Done(&b""[..], id("_test9-foo")));
        assert_eq!(identifier(b"9foo"), IResult::Error(Code(ErrorKind::RegexpFind)));
        assert_eq!(identifier(b"-foo"), IResult::Error(Code(ErrorKind::RegexpFind)));
    }

    #[test]
    fn attr_set_parse() {
        assert_eq!(attr_set(b"{test}"), IResult::Done(&b""[..], a("test")));
    }
}
