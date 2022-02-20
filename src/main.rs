use std::{vec};

use ariadne::{Label, Report, ReportKind, Source};
use chumsky::prelude::*;

#[derive(Debug, Clone)]
enum Instr {
    StackOp(Box<Instr>),
    Push,
    Pop,
    Bring,
    Send,
    Decr,
    Incr,
    In,
    Out,
    Loop(Vec<Instr>),
}

fn parser() -> impl Parser<char, Vec<Instr>, Error = Simple<char>> {
    let comment = just("{").then(take_until(just('}'))).padded();

    let stack_instr = choice((
        just('#').to(Instr::Push),
        just('*').to(Instr::Pop),
        just('<').to(Instr::Bring),
        just('>').to(Instr::Send),
    ));
    let instruction = recursive(|instr| {
        choice((
            just('~')
                .then(stack_instr.clone())
                .map(|(_, instr)| Instr::StackOp(Box::new(instr))),
            stack_instr,
            just('-').to(Instr::Decr),
            just('+').to(Instr::Incr),
            just(',').to(Instr::In),
            just('.').to(Instr::Out),
            instr
                .repeated()
                .delimited_by(just("["), just("]"))
                .map(|v| Instr::Loop(v)),
        ))
        .padded_by(comment.repeated())
        .padded()
    });

    instruction.repeated().then_ignore(end())
}

fn kardz(instr: Vec<Instr>, kard_stacks: &mut Vec<Vec<u8>>) {
    let mut instr_iter = instr.iter();
    let term = console::Term::stdout();
    while let Some(instr) = instr_iter.next() {
        match instr {
            Instr::StackOp(instr) => match **instr {
                Instr::Push => kard_stacks.push(vec![0]),
                Instr::Pop => {
                    kard_stacks.pop().unwrap();
                }
                Instr::Bring => {
                    let pop = kard_stacks.remove(0);
                    kard_stacks.push(pop);
                }
                Instr::Send => {
                    let pop = kard_stacks.pop().unwrap();
                    kard_stacks.insert(0, pop);
                }
                _ => todo!(),
            },
            Instr::Push => kard_stacks.last_mut().unwrap().push(0),
            Instr::Pop => {
                kard_stacks.last_mut().unwrap().pop().unwrap();
            }
            Instr::Bring => {
                let pop = kard_stacks.last_mut().unwrap().remove(0);
                kard_stacks.last_mut().unwrap().push(pop);
            }
            Instr::Send => {
                let pop = kard_stacks.last_mut().unwrap().pop().unwrap();
                kard_stacks.last_mut().unwrap().insert(0, pop);
            }
            Instr::Decr => {
                let kard = kard_stacks.last_mut().unwrap().last_mut().unwrap();
                *kard -= 1;
            }
            Instr::Incr => {
                let kard = kard_stacks.last_mut().unwrap().last_mut().unwrap();
                *kard += 1;
            }
            Instr::Loop(lp) => loop {
                kardz(lp.to_vec(), kard_stacks);
                if *kard_stacks.last_mut().unwrap().last_mut().unwrap() == 0 {
                    break;
                }
            },
            Instr::In => {
                let kard = kard_stacks.last_mut().unwrap().last_mut().unwrap();
                *kard = term.read_char().unwrap() as u8;
            }
            Instr::Out => {
                let kard = kard_stacks.last_mut().unwrap().last_mut().unwrap();
                print!("{}", *kard as char);
            }
        }
    }
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    match parser().parse(src.clone()) {
        Ok(ast) => {
            let mut kard_stacks: Vec<Vec<u8>> = vec![vec![0]];
            kardz(ast, &mut kard_stacks);
        }
        Err(parse_errs) => {
            parse_errs.into_iter().for_each(|e| {
                Report::build(ReportKind::Error, (), e.span().end + 1)
                    .with_message("Unexpected token")
                    .with_label(Label::new(e.span()).with_message(format!("{}", e)))
                    .finish()
                    .print(Source::from(src.clone()))
                    .unwrap();
            });
        }
    }
}
