use std::marker::PhantomData;
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    executor, try_join,
};
#[allow(unused_imports)]
use ::rumpsteak::{
    channel::Bidirectional,
    session,
    Branch,
    End,
    Message,
    Receive,
    Role,
    Roles,
    Select,
    Send,
    effect::{
        SideEffect,
        Constant,
        Incr,
    },
    try_session,
    predicate::{
        Predicate,
        Tautology,
        LTnVar,
        GTnVar,
        LTnConst,
        GTnConst
    },
};

use std::collections::HashMap;
use std::error::Error;

type Channel = Bidirectional<UnboundedSender<Label>, UnboundedReceiver<Label>>;
type Name = char;
type Value = u32;

pub struct AdHocPred<L, LHS: Predicate, RHS: Predicate> {
    _p: PhantomData<(L, LHS, RHS)>
}

impl<L, LHS: Predicate, RHS: Predicate> Default for AdHocPred<L, LHS, RHS> {
    fn default() -> Self { Self { _p: PhantomData} }
}

#[derive(Roles)]
#[allow(dead_code)]
struct Roles {
    c: C,
    s: S,
}

#[derive(Role)]
#[message(Label)]
struct C {
    #[route(S)]
    s: Channel,
}

#[derive(Role)]
#[message(Label)]
struct S {
    #[route(C)]
    c: Channel,
}

#[derive(Message)]
enum Label {
    Start(Start),
    Stop(Stop),
}

struct Start;

struct Stop;

// role C
#[session(Name, Value)]
type ProtocolC = Select<S, AdHocPred<Label, GTnConst<Stop, 'x', 11>, LTnConst<Start, 'x', 10>>, Constant<Name, Value>, ProtocolC0>;

#[session(Name, Value)]
enum ProtocolC0 {
    Stop(Stop, End),
    Start(Start, End),
}

// role S
#[session(Name, Value)]
type ProtocolS = Branch<C, AdHocPred<Label, Tautology<Name, Value, Label>, Tautology<Name, Value, Label>, >, Constant<Name, Value>, ProtocolS0>;

#[session(Name, Value)]
enum ProtocolS0 {
    Stop(Stop, End),
    Start(Start, End),
}

impl<L, LHS: Predicate, RHS: Predicate> Predicate for AdHocPred<L, LHS, RHS>
{
    type Name = char;
    type Value = i32;
    type Label = L;
    type Error = ();

    fn check(&self, m: &HashMap<Self::Name, Self::Value>, l: Option<&Self::Label>) -> Result<(), Self::Error> {
        
        if let Some(l) = l {
            match l {
                Stop(_) => LHS::default().check(m, None),
                Start(_) => RHS::default().check(m, None),
                _ => Err(())
            }
        } else {
            Ok(())
        }
    }
}


async fn c(role: &mut C) -> Result<(), Box<dyn Error>> {
    let mut map = HashMap::new();
    map.insert('x', 0);
    map.insert('y', 10);

    try_session(role, map,
        |s: ProtocolC<'_, _>| async {
            let mut i = 0;
            loop {
                let mut s = s;
                if i<=10 {
                    s = s.select(Start(i)).await?
                } else {
                    let s = s.select(Stop(i)).await?;
                    return Ok(((), s))
                };
                i += 1;
            };
            
        })
        .await
}

async fn s(role: &mut S) -> Result<(), Box<dyn Error>> {
    try_session(role, HashMap::new(),
        |s: ProtocolS<'_, _>| async {
            // let s = s.send(Password(1)).await?;
            let mut s = s;
            loop{                
                match s.branch().await? {
                    ProtocolS0::Start(_, s2) => {
                        s = s2 
                    },
                    ProtocolS0::Stop(_, end) => {
                        println!("Terminated");
                        return Ok(((), end))
                    },
                }
            }
        })
    .await
}

fn main() {
    let mut roles = Roles::default();
    executor::block_on(async {
        try_join!(s(&mut roles.s), c(&mut roles.c)).unwrap();
    });
}