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
        GTnVar
    },
};

use std::collections::HashMap;
use std::error::Error;

type Channel = Bidirectional<UnboundedSender<Label>, UnboundedReceiver<Label>>;
type Name = char;
type Value = u32;

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
type ProtocolC = Send<A, Password, Tautology<Name, Value>, Constant<Name, Value>, Branch<A, Tautology<Name, Value>, Constant<Name, Value>, ProtocolC2>>;

#[session(Name, Value)]
enum ProtocolC2 {
    Success(Success, End),
    Failure(Failure, ProtocolC),
}

struct AdHocPred {}
impl Predicate for AdHocPred {
    fn check(...) {
        match label{
            Success => {
                x<10 .check()
            }
            Failure => {

            }
        }
    }
}

#[session(Name, Value)]
type ProtocolS = Branch<A, AdHocPred, Constant<Name, Value>, ProtocolS0>;

#[session(Name, Value)]
enum ProtocolS0 {
    Success(Success, End),
    Failure(Failure, Branch<A, Tautology<Name, Value>, Constant<Name, Value>, ProtocolS0>),
}

#[session(Name, Value)]
type ProtocolA = Receive<C, Password, Tautology<Name, Value>, Constant<Name, Value>, Select<C, Tautology<Name, Value>, Constant<Name, Value>, ProtocolA2>>;

#[session(Name, Value)]
enum ProtocolA2 {
    Failure(Failure, Send<S, Failure, Tautology<Name, Value>, Constant<Name, Value>, ProtocolA>),
    Success(Success, Send<S, Success, Tautology<Name, Value>, Constant<Name, Value>, End>),
}
