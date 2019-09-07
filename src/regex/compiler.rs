use super::parser::RegExp;

pub type Ins = Vec<Instruction>;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    MatchChar(Char),
    Jump(isize),
    Branch(isize, isize),
    MatchPos(Position),
    GroupParenL,
    GroupParenR,
    Finish,
}

#[derive(Debug, PartialEq)]
pub enum Char {
    Literal(char),
    Any,
    CharClass(bool, Vec<char>),
}

#[derive(Debug, PartialEq)]
pub enum Position {
    Front,
    Back,
}

pub fn compile(ast: RegExp) -> Result<Ins, String> {
    let mut ins = compile_impl(ast)?;
    ins.push(Instruction::Finish);

    // fix instruction index
    Ok(ins
        .into_iter()
        .enumerate()
        .map(|(idx, instruction)| {
            let idx = idx as isize;
            match instruction {
                Instruction::Jump(p) => Instruction::Jump(p + idx),
                Instruction::Branch(x, y) => Instruction::Branch(x + idx, y + idx),
                other => other,
            }
        })
        .collect())
}

fn compile_impl(exp: RegExp) -> Result<Ins, String> {
    match exp {
        RegExp::PrefixSuffix(pre, body, suf) => {
            let mut ins = vec![];
            if pre {
                ins.push(Instruction::MatchPos(Position::Front));
            }
            ins.append(&mut compile_impl(*body)?);
            if suf {
                ins.push(Instruction::MatchPos(Position::Back));
            }
            Ok(ins)
        }
        RegExp::Branch(mut v) => {
            if v.len() == 1 {
                let exp = v.pop().unwrap();
                compile_impl(exp)
            } else {
                assert!(v.len() >= 2);
                let exp = v.pop().unwrap();
                compile_branch(RegExp::Branch(v), exp)
            }
        }
        RegExp::Connect(mut v) => {
            if v.len() == 1 {
                let exp = v.pop().unwrap();
                compile_impl(exp)
            } else {
                assert!(v.len() >= 2);
                let exp = v.pop().unwrap();
                compile_connection(RegExp::Connect(v), exp)
            }
        }
        RegExp::RepeatStar(r) => compile_repeat_star(*r),
        RegExp::RepeatPlus(r) => compile_repeat_plus(*r),
        RegExp::RepeatRange(r, l, h) => {
            let mut tmp = vec![*r.clone(); l];
            let mut maybe = vec![RegExp::Maybe(Box::new(*r)); h - l];
            tmp.append(&mut maybe);
            compile_impl(RegExp::Connect(tmp))
        }
        RegExp::Maybe(r) => compile_maybe(*r),
        RegExp::Group(r) => compile_group(*r),
        // char
        RegExp::AnyChar => compile_match_char(Char::Any),
        RegExp::CharClass(f, r) => compile_match_char(Char::CharClass(f, r)),
        RegExp::Literal(c) => compile_match_char(Char::Literal(c)),
    }
}

fn compile_match_char(c: Char) -> Result<Ins, String> {
    Ok(vec![Instruction::MatchChar(c)])
}

fn compile_branch(x: RegExp, y: RegExp) -> Result<Ins, String> {
    let mut x_ins = compile_impl(x)?;
    let mut y_ins = compile_impl(y)?;
    let mut ins = vec![Instruction::Branch(1, 2 + x_ins.len() as isize)];
    ins.append(&mut x_ins);
    ins.push(Instruction::Jump(1 + y_ins.len() as isize));
    ins.append(&mut y_ins);
    Ok(ins)
}

fn compile_connection(x: RegExp, y: RegExp) -> Result<Ins, String> {
    let mut ins = vec![];
    ins.append(&mut compile_impl(x)?);
    ins.append(&mut compile_impl(y)?);
    Ok(ins)
}

fn compile_maybe(exp: RegExp) -> Result<Ins, String> {
    let mut exp_ins = compile_impl(exp)?;
    let mut ins = vec![Instruction::Branch(1, 1 + exp_ins.len() as isize)];
    ins.append(&mut exp_ins);
    Ok(ins)
}

fn compile_repeat_star(exp: RegExp) -> Result<Ins, String> {
    let mut exp_ins = compile_impl(exp)?;
    let len = exp_ins.len() as isize;
    let mut ins = vec![Instruction::Branch(1, 2 + len)];
    ins.append(&mut exp_ins);
    ins.push(Instruction::Jump(-(1 + len)));
    Ok(ins)
}

fn compile_repeat_plus(exp: RegExp) -> Result<Ins, String> {
    let mut exp_ins = compile_impl(exp)?;
    let len = exp_ins.len() as isize;
    let mut ins = vec![];
    ins.append(&mut exp_ins);
    ins.push(Instruction::Branch(-len, 1));
    Ok(ins)
}

fn compile_group(exp: RegExp) -> Result<Ins, String> {
    let mut exp_ins = compile_impl(exp)?;
    let mut ins = vec![];
    ins.push(Instruction::GroupParenL);
    ins.append(&mut exp_ins);
    ins.push(Instruction::GroupParenR);
    Ok(ins)
}

#[test]
fn test_compile() {
    use Char::*;
    use Instruction::*;

    // pat: `ab`
    assert_eq!(
        compile(RegExp::Branch(vec![RegExp::Connect(vec![
            RegExp::Literal('a'),
            RegExp::Literal('b')
        ])])),
        Ok(vec![
            MatchChar(Literal('a')),
            MatchChar(Literal('b')),
            Finish
        ])
    );

    // pat: `a?`
    assert_eq!(
        compile(RegExp::Branch(vec![RegExp::Connect(vec![RegExp::Maybe(
            Box::new(RegExp::Literal('a'))
        ),])])),
        Ok(vec![Branch(1, 2), MatchChar(Literal('a')), Finish])
    );

    // pat: `a*`
    assert_eq!(
        compile(RegExp::Branch(vec![RegExp::Connect(vec![
            RegExp::RepeatStar(Box::new(RegExp::Literal('a'))),
        ])])),
        Ok(vec![Branch(1, 3), MatchChar(Literal('a')), Jump(1), Finish])
    );

    // pat: `a|b*`
    assert_eq!(
        compile(RegExp::Branch(vec![
            RegExp::Connect(vec![RegExp::Literal('a'),]),
            RegExp::Connect(vec![RegExp::RepeatStar(Box::new(RegExp::Literal('b'))),])
        ])),
        Ok(vec![
            Branch(1, 3),
            MatchChar(Literal('a')),
            Jump(6),
            Branch(4, 6),
            MatchChar(Literal('b')),
            Jump(4),
            Finish
        ])
    );
}
