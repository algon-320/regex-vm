/*
<PrefixSuffix> = `^`? <Branch> `$`?
<Branch> = <Connect> (`|` <Connect>)*
<Connect> = (<Group> / <RepeatStar> / <RepeatPlus> / <Maybe>)+
<RepeatStar> = <Group> `*`
<RepeatPlus> = <Group> `+`
<Maybe> = <Group> `?`
<Group> = `(` <regex> `)` / <Literal> / <AnyChar> / <CharClass>
<Literal> = not <special_chars> / `\` <special_chars>
<AnyChar> = `.`
<CharClass> = `[` `^`? (`\`と`]`以外 これらは`\`でエスケープ) `]`
<special_chars> = [|*+?.\\()]
*/

#[derive(Debug, PartialEq, Clone)]
pub enum RegExp {
    PrefixSuffix(bool, Box<RegExp>, bool),
    Branch(Vec<RegExp>),
    Connect(Vec<RegExp>),
    RepeatStar(Box<RegExp>),
    RepeatPlus(Box<RegExp>),
    RepeatRange(Box<RegExp>, usize, usize),
    Maybe(Box<RegExp>),
    Group(Box<RegExp>),
    AnyChar,
    CharClass(bool, Vec<char>),
    Literal(char),
}

pub fn parse(pat: &str) -> Result<RegExp, String> {
    parse_regex(&mut pat.chars().peekable())
}

use std::iter::Peekable;
use std::str::Chars;

fn parse_regex(pat: &mut Peekable<Chars>) -> Result<RegExp, String> {
    let prefix = if pat.peek() == Some(&'^') {
        pat.next();
        true
    } else {
        false
    };
    let body = parse_branch(pat)?;
    let suffix = pat.next() == Some('$');
    Ok(RegExp::PrefixSuffix(prefix, Box::new(body), suffix))
}
fn parse_branch(pat: &mut Peekable<Chars>) -> Result<RegExp, String> {
    let mut ret = vec![parse_connect(pat)?];
    while pat.peek() == Some(&'|') {
        expect_char('|', pat)?;
        ret.push(parse_connect(pat)?);
    }
    Ok(RegExp::Branch(ret))
}
fn parse_connect(pat: &mut Peekable<Chars>) -> Result<RegExp, String> {
    let mut ret = vec![parse_repeat(pat)?];
    fn is_end_of_connect(c: Option<&char>) -> bool {
        c == None || c == Some(&'|') || c == Some(&')') || c == Some(&'$')
    }
    while !is_end_of_connect(pat.peek()) {
        ret.push(parse_repeat(pat)?);
    }
    Ok(RegExp::Connect(ret))
}
fn parse_repeat(pat: &mut Peekable<Chars>) -> Result<RegExp, String> {
    let group = parse_group(pat)?;
    match pat.peek() {
        Some(&'*') => {
            pat.next();
            Ok(RegExp::RepeatStar(Box::new(group)))
        }
        Some(&'+') => {
            pat.next();
            Ok(RegExp::RepeatPlus(Box::new(group)))
        }
        Some(&'?') => {
            pat.next();
            Ok(RegExp::Maybe(Box::new(group)))
        }
        Some(&'{') => {
            pat.next();
            let low = get_number(pat)? as usize;
            expect_char(',', pat)?;
            let hight = get_number(pat)? as usize;
            expect_char('}', pat)?;
            Ok(RegExp::RepeatRange(Box::new(group), low, hight))
        }
        _ => Ok(group),
    }
}
fn parse_group(pat: &mut Peekable<Chars>) -> Result<RegExp, String> {
    match pat.peek() {
        Some(&'(') => {
            expect_char('(', pat)?;
            let ret = parse_branch(pat)?;
            expect_char(')', pat)?;
            Ok(RegExp::Group(Box::new(ret)))
        }
        _ => parse_char(pat),
    }
}
fn parse_char(pat: &mut Peekable<Chars>) -> Result<RegExp, String> {
    match pat.peek() {
        Some(&'[') => parse_charclass(pat),
        Some(&'.') => {
            expect_char('.', pat)?;
            Ok(RegExp::AnyChar)
        }
        _ => parse_literal(pat),
    }
}
fn parse_charclass(pat: &mut Peekable<Chars>) -> Result<RegExp, String> {
    expect_char('[', pat)?;

    let flipped = if pat.peek() == Some(&'^') {
        pat.next();
        true
    } else {
        false
    };

    let mut char_set = vec![];
    loop {
        let c = match pat.peek() {
            Some(&']') => break,
            _ => {
                pat.peek()
                    .ok_or("Syntax Error: unclosed char-class, `]` not found")?;
                get_char(pat)?
            }
        };
        char_set.push(c);
    }

    expect_char(']', pat)?;
    Ok(RegExp::CharClass(flipped, char_set))
}
fn parse_literal(pat: &mut Peekable<Chars>) -> Result<RegExp, String> {
    Ok(RegExp::Literal(get_char(pat)?))
}

fn get_char(pat: &mut Peekable<Chars>) -> Result<char, String> {
    let tmp = pat
        .peek()
        .ok_or("Syntax Error: expect a character but found End-Of-String")?;
    if tmp == &'\\' {
        get_escaped_char(pat)
    } else {
        Ok(pat.next().unwrap())
    }
}

fn expect_char(expect: char, pat: &mut Peekable<Chars>) -> Result<char, String> {
    let tmp = pat.next().ok_or(format!(
        "Syntax Error: expect `{}` but found End-Of-String",
        expect
    ))?;
    if tmp != expect {
        return Err(format!(
            "Syntax Error: expect `{}` but found `{}`",
            expect, tmp
        ));
    }
    Ok(tmp)
}

fn get_escaped_char(pat: &mut Peekable<Chars>) -> Result<char, String> {
    expect_char('\\', pat)?;
    let c = pat
        .next()
        .ok_or("Syntax Error: expect char but found End-Of-String")?;
    Ok(c)
}

fn get_number(pat: &mut Peekable<Chars>) -> Result<u64, String> {
    let mut ret = 0;
    loop {
        let next = pat.peek();
        if let Some(c) = next {
            if c.is_ascii_digit() {
                ret *= 10;
                ret += *c as u64 - 48;
            } else {
                break;
            }
        } else {
            break;
        }
        pat.next();
        continue;
    }
    Ok(ret)
}

#[test]
fn test_parse() {
    use RegExp::*;
    assert_eq!(
        parse(r#"a\)"#).unwrap(),
        Branch(vec![Connect(vec![Literal('a'), Literal(')')]),])
    )
}
#[test]
fn test_parse_branch() {
    use RegExp::*;
    assert_eq!(
        parse("a|b").unwrap(),
        Branch(vec![
            Connect(vec![Literal('a')]),
            Connect(vec![Literal('b')]),
        ])
    );
    assert_eq!(
        parse("a|b|c").unwrap(),
        Branch(vec![
            Connect(vec![Literal('a')]),
            Connect(vec![Literal('b')]),
            Connect(vec![Literal('c')]),
        ])
    );
    assert_eq!(
        parse("ab|cd").unwrap(),
        Branch(vec![
            Connect(vec![Literal('a'), Literal('b')]),
            Connect(vec![Literal('c'), Literal('d')]),
        ])
    );
}
#[test]
fn test_parse_connect() {
    use RegExp::*;
    assert_eq!(
        parse("ab").unwrap(),
        Branch(vec![Connect(vec![Literal('a'), Literal('b'),])])
    );
    assert_eq!(
        parse("abcd").unwrap(),
        Branch(vec![Connect(vec![
            Literal('a'),
            Literal('b'),
            Literal('c'),
            Literal('d'),
        ])])
    );
}
#[test]
fn test_parse_repeat() {
    use RegExp::*;
    assert_eq!(
        parse("a*b*").unwrap(),
        Branch(vec![Connect(vec![
            RepeatStar(Box::new(Literal('a'))),
            RepeatStar(Box::new(Literal('b'))),
        ])])
    );
    assert_eq!(
        parse("ab+").unwrap(),
        Branch(vec![Connect(vec![
            Literal('a'),
            RepeatPlus(Box::new(Literal('b'))),
        ])])
    );
    assert_eq!(
        parse("a?").unwrap(),
        Branch(vec![Connect(vec![Maybe(Box::new(Literal('a'))),])])
    );
    assert_eq!(
        parse(".*").unwrap(),
        Branch(vec![Connect(vec![RepeatStar(Box::new(AnyChar)),])])
    );
}

#[test]
fn test_parse_group() {
    use RegExp::*;
    assert_eq!(
        parse("(ab)").unwrap(),
        Branch(vec![Connect(vec![Group(Box::new(Branch(vec![Connect(
            vec![Literal('a'), Literal('b'),]
        )])))])])
    );
    assert_eq!(
        parse("(ab)(cd)").unwrap(),
        Branch(vec![Connect(vec![
            Group(Box::new(Branch(vec![Connect(vec![
                Literal('a'),
                Literal('b'),
            ])]))),
            Group(Box::new(Branch(vec![Connect(vec![
                Literal('c'),
                Literal('d'),
            ])])))
        ])])
    );
    assert_eq!(
        parse("(ab)+(cd)").unwrap(),
        Branch(vec![Connect(vec![
            RepeatPlus(Box::new(Group(Box::new(Branch(vec![Connect(vec![
                Literal('a'),
                Literal('b'),
            ])]))))),
            Group(Box::new(Branch(vec![Connect(vec![
                Literal('c'),
                Literal('d'),
            ])])))
        ])])
    );
    assert_eq!(
        parse("(ab)?").unwrap(),
        Branch(vec![Connect(vec![Maybe(Box::new(Group(Box::new(
            Branch(vec![Connect(vec![Literal('a'), Literal('b'),])])
        ))))])])
    );
}

#[test]
fn test_parse_charclass() {
    use RegExp::*;
    assert_eq!(
        parse("[abcd]").unwrap(),
        Branch(vec![Connect(vec![CharClass(
            false,
            vec!['a', 'b', 'c', 'd']
        )])])
    );
    assert_eq!(
        parse("[^abcd]").unwrap(),
        Branch(vec![Connect(vec![CharClass(
            true,
            vec!['a', 'b', 'c', 'd']
        )])])
    );
    assert_eq!(
        parse("[^^]").unwrap(),
        Branch(vec![Connect(vec![CharClass(true, vec!['^'])])])
    );
    assert_eq!(
        parse(r#"[^\]\)]"#).unwrap(),
        Branch(vec![Connect(vec![CharClass(true, vec![']', ')'])])])
    );
    assert_eq!(
        parse(r#"[^|]"#).unwrap(),
        Branch(vec![Connect(vec![CharClass(true, vec!['|'])])])
    );
    assert_eq!(
        parse(r#"[^\\]"#).unwrap(),
        Branch(vec![Connect(vec![CharClass(true, vec!['\\'])])])
    );
    assert_eq!(
        parse("["),
        Err("Syntax Error: unclosed char-class, `]` not found".to_string())
    )
}
