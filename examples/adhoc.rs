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
type Value = i32;

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
    Number(Number),
    Start(Start),
    Stop(Stop),
}

struct Number(i32);

struct Start;

struct Stop;

// role C
#[session(Name, Value)]
type ProtocolC = Receive<S, Number, Tautology<Name, Value, Label>, Constant<Name, Value>, Select<S, AdHocPred<Label, GTnConst<Stop, 'x', 11>, LTnConst<Start, 'x', 10>>, Constant<Name, Value>, ProtocolC1>>;

#[session(Name, Value)]
enum ProtocolC1 {
    Stop(Stop, End),
    Start(Start, End),
}

// role S
#[session(Name, Value)]
type ProtocolS = Send<C, Number, Tautology<Name, Value, Label>, Constant<Name, Value>, Branch<C, Tautology<Name, Value, Label>, Constant<Name, Value>, ProtocolS1>>;

#[session(Name, Value)]
enum ProtocolS1 {
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
                Stop => {
                    LHS::default().check(m, None).unwrap();
                    Ok(())
                }
                Start => {
                    RHS::default().check(m, None).unwrap();
                    Ok(())
                }
                _ => Err(())
            }
        } else {
            Ok(())
        }
    }
}


// global protocol Protocol(role C, role S)
// {    
//     number(i32) from S to C;                             
//     choice at C
//     {
//         start() from C to S; @x<10
//     }
//     or
//     {
//         stop() from C to S;@x>11
//     }
// }

async fn c(role: &mut C) -> Result<(), Box<dyn Error>> {
    let mut map = HashMap::new();

    try_session(role, map,
        |s: ProtocolC<'_, _>| async {
            let mut s = s;
            let (Number(n), s) = s.receive().await?;
            if n<=10 {
                let s = s.select(Start).await?;
                Ok(((), s))
            } else {
                let s = s.select(Stop).await?;
                Ok(((), s))
            };
        })
        .await
}

async fn s(role: &mut S) -> Result<(), Box<dyn Error>> {
    try_session(role, HashMap::new(),
        |s: ProtocolS<'_, _>| async {
            let mut s = s;
            let s = s.send(Number(1)).await?;
            match s.branch().await? {
                ProtocolS0::Start(_, end) => {
                    println!("Start");
                    Ok(((), end))
                },
                ProtocolS0::Stop(_, end) => {
                    println!("Terminated");
                    Ok(((), end))
                },
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