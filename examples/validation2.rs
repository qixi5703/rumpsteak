#[allow(unused_imports)]
use ::rumpsteak::{
    channel::Bidirectional, session, try_session, Branch, End, Message, Receive, Role, Roles,
    Select, Send,
};
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    executor, try_join,
};

use std::error::Error;

type Channel = Bidirectional<UnboundedSender<Label>, UnboundedReceiver<Label>>;

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

#[session]
type AuthenticationC = Send<A, Password, Branch<A, AuthenticationC2>>;

#[session]
enum AuthenticationC2 {
    Success(Success, End),
    Failure(Failure, AuthenticationC),
}

#[session]
type AuthenticationS = Branch<A, AuthenticationS0>;

#[session]
enum AuthenticationS0 {
    Success(Success, End),
    Failure(Failure, Branch<A, AuthenticationS0>),
}

#[session]
type AuthenticationA = Receive<C, Password, Select<C, AuthenticationA2>>;

#[session]
enum AuthenticationA2 {
    Failure(Failure, Send<S, Failure, AuthenticationA>),
    Success(Success, Send<S, Success, End>),
}


async fn a(role: &mut A) -> Result<(), Box<dyn Error>> {
    try_session(role, |s: AuthenticationA<'_, _>| async {
        let mut i = 0;
        let s = loop {
            let (Password(msg), s) = s.receive().await?;
            let mut s = s;
            s = if msg == 10 {
                s.select(Success).await?

            } else {
                s.select(Failure).await?
            };
            i += 1;
        };
        Ok(((), s))
    })
    .await
}


async fn s(role: &mut S) -> Result<(), Box<dyn Error>> {
    try_session(role, |s: AuthenticationS<'_, _>| async {
        let mut s = s;
            loop{
                
                match s.branch().await? {
                    AuthenticationS0::Failure(_, s2) => {
                        s = s2 
                    },
                    AuthenticationS0::Success(_, end) => {
                        println!("Sucess log in");
                        return Ok(((), end))
                    },
                }
            }
    })
    .await
}

async fn c(role: &mut C) -> Result<(), Box<dyn Error>> {
    try_session(role, |s: AuthenticationC<'_, _>| async {
        let mut s = s;
            loop{
                let s = s.send(Password(1)).await?;
                
                match s.branch().await? {
                    AuthenticationC2::Failure(_, s2) => {
                        s = s2 
                    },
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
