use super::compiler::{Char, Ins, Instruction, Position};

type PC = usize;
type SP = usize;
type Context = (PC, SP, usize);

// substring match
pub fn search(ins: &Ins, text: String) -> Option<Vec<(usize, usize)>> {
    let text = text.chars().collect::<Vec<_>>();
    for p in 0..text.len() {
        let ret = search_impl(ins, p, text.as_slice());
        if ret != None {
            return ret;
        }
    }
    None
}

fn search_impl(ins: &Ins, from: usize, text: &[char]) -> Option<Vec<(usize, usize)>> {
    let mut thread_stack: Vec<Context> = Vec::new();
    let mut ret: Vec<(usize, usize)> = Vec::new();
    let mut group_paren_l = Vec::new();

    let mut pc: PC = 0;
    let mut sp: SP = from;

    use Instruction::*;
    loop {
        let inst = ins.get(pc)?;
        match inst {
            MatchChar(c) => {
                let ok = match c {
                    Char::Literal(l) => text.len() > sp && l == text.get(sp).unwrap(),
                    Char::CharClass(f, cs) => {
                        fn check(flip: bool, cs: &Vec<char>, c: char) -> bool {
                            for k in cs.iter() {
                                if k == &c {
                                    return !flip;
                                }
                            }
                            flip
                        }
                        text.len() > sp && check(*f, cs, *text.get(sp).unwrap())
                    }
                    Char::Any => text.len() > sp,
                };
                if ok {
                    pc += 1;
                    sp += 1;
                } else {
                    let context = thread_stack.pop()?;
                    pc = context.0;
                    sp = context.1;
                    group_paren_l.truncate(context.2);
                }
            }
            MatchPos(p) => match p {
                Position::Front => {
                    if sp == 0 {
                        pc += 1;
                    } else {
                        let context = thread_stack.pop()?;
                        pc = context.0;
                        sp = context.1;
                        group_paren_l.truncate(context.2);
                    }
                }
                Position::Back => {
                    if sp == text.len() {
                        pc += 1;
                    } else {
                        let context = thread_stack.pop()?;
                        pc = context.0;
                        sp = context.1;
                        group_paren_l.truncate(context.2);
                    }
                }
            },
            Branch(x, y) => {
                thread_stack.push((*y as usize, sp, group_paren_l.len()));
                pc = *x as usize;
            }
            Jump(x) => {
                pc = *x as usize;
            }
            GroupParenL => {
                group_paren_l.push(sp);
                pc += 1;
            }
            GroupParenR => {
                let left = group_paren_l.pop()?;
                ret.push((left, sp));
                pc += 1;
            }
            Finish => {
                ret.insert(0, (from, sp));
                return Some(ret);
            }
        }
    }
}
