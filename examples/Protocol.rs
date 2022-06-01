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
        Tautology,
        LTnVar,
        GTnVar,
        LTnConst
    },
    predicate::Predicate,
};

use std::collections::HashMap;
use std::error::Error;

type Channel = Bidirectional<UnboundedSender<Label>, UnboundedReceiver<Label>>;
type Name = char;
type Value = i32;

#[derive(Roles)]
#[allow(dead_code)]
struct Roles {
    c: C,
    s: S,
    a: A,
}

#[derive(Role)]
#[message(Label)]
struct C {
    #[route(S)]
    s: Channel,
    #[route(A)]
    a: Channel,
}

#[derive(Role)]
#[message(Label)]
struct S {
    #[route(C)]
    c: Channel,
    #[route(A)]
    a: Channel,
}

#[derive(Role)]
#[message(Label)]
struct A {
    #[route(C)]
    c: Channel,
    #[route(S)]
    s: Channel,
}

#[derive(Message)]
enum Label {
    Password(Password),
    Failure(Failure),
    Success(Success),
}

struct Password(u32);

struct Failure;

struct Success;

#[session(Name, Value)]
type AuthenticationC = Send<A, Password, Tautology<Name, Value, Label>, Constant<Name, Value>, Branch<A, Tautology<Name, Value, Label>, Constant<Name, Value>, AuthenticationC2>>;

#[session(Name, Value)]
enum AuthenticationC2 {
    Success(Success, End),
    Failure(Failure, AuthenticationC),
}

#[session(Name, Value)]
type AuthenticationS = Branch<A,Tautology<Name, Value, Label>, Constant<Name, Value>, AuthenticationS0>;

#[session(Name, Value)]
enum AuthenticationS0 {
    Success(Success, End),
    Failure(Failure, Branch<A,Tautology<Name, Value, Label>, Constant<Name, Value>, AuthenticationS0>),
}

#[session(Name, Value)]
type AuthenticationA = Receive<C, Password, Tautology<Name, Value, Label>, Constant<Name, Value>, Select<C, AdHocPred<Label, LTnConst<Success, 'x', 10>, LTnConst<Failure, 'x', 5>>, Constant<Name, Value>, AuthenticationA2>>;

#[session(Name, Value)]
enum AuthenticationA2 {
    Failure(Failure, Send<S, Failure, Tautology<Name, Value, Failure>, Constant<Name, Value>, AuthenticationA>),
    Success(Success, Send<S, Success, Tautology<Name, Value, Success>, Constant<Name, Value>, End>),
}


pub struct AdHocPred<L, LHS: Predicate, RHS: Predicate> {
    _p: PhantomData<(L, LHS, RHS)>
}

impl<L, LHS: Predicate, RHS: Predicate> Default for AdHocPred<L, LHS, RHS> {
    fn default() -> Self { Self { _p: PhantomData} }
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
                Success => LHS::default().check(m, None),
                Failure => RHS::default().check(m, None),
                _ => Err(())
            }
        } else {
            Ok(())
        }
    }
}

async fn s(role: &mut S) -> Result<(), Box<dyn Error>> {
    let mut map = HashMap::new();
    map.insert('x', 0);
    map.insert('y', 10);

    try_session(role, map,
        |s: AuthenticationS<'_, _>| async {
            
            let mut i = 0;
            let (Password(password), s) = s.receive().await?;
            let s = loop {
                // let (Password(password), s) = s.receive().await?;
                
                let mut s = s;
                s = if i <= 9 {
                    s.select(Failure(i)).await?
                } else {
                    break s;
                };
                // let (Password(password), s) = s.receive().await?;
                i += 1;
            };
            
            // let s = s.select(Abort(i)).await?;
            Ok(((), s))
        })
        .await
}


async fn a(role: &mut A) -> Result<(), Box<dyn Error>> {
    let mut map = HashMap::new();
    map.insert('x', 0);
    map.insert('y', 10);

    try_session(role, map,
        |s: AuthenticationS<'_, _>| async {
            let mut i = 0;
            let (Password(password), s) = s.receive().await?;
            loop {
                let mut s = s;
                if password = 67 {
                    s = s.select(Success(i)).await?;
                    return Ok(((), s))
                } else if i<=20 {
                    s = s.select(Failure(i)).await?
                } else {
                    // let s = s.select(Abort(i)).await?;
                    return Ok(((), s))
                };
                i += 1;
            };
            
        })
        .await
}

async fn c(role: &mut C) -> Result<(), Box<dyn Error>> {
    try_session(role, HashMap::new(),
        |s: AuthenticationC<'_, _>| async {
            // let s = s.send(Password(1)).await?;
            let mut s = s;
            loop{
                let s = s.send(Password(1)).await?;
                
                match s.branch().await? {
                    AuthenticationC2::Failure(_, s2) => {
                        s = s2 
                    },
                    // AuthenticationC2::Abort(_, end) => {
                    //     println!("Terminated");
                    //     return Ok(((), end))
                    // },
                    AuthenticationC2::Success(_, end) => {
                        println!("Sucess log in");
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
        try_join!(s(&mut roles.s), c(&mut roles.c),  a(&mut roles.a)).unwrap();
    });
}

// global garantee 
// local type checking
//projection

// example could be -image- here is the example