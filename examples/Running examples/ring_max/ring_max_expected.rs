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
        Noop,
        Incr,
    },
    try_session,
    predicate::*,
    ParamName,
    Param,
};

use std::collections::HashMap;
use std::error::Error;

type Channel = Bidirectional<UnboundedSender<Label>, UnboundedReceiver<Label>>;

type Name = char;
type Value = i32;

#[derive(Roles)]
#[allow(dead_code)]
struct Roles {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
}

#[derive(Role)]
#[message(Label)]
struct A {
    #[route(B)]
    b: Channel,
    #[route(C)]
    c: Channel,
    #[route(D)]
    d: Channel,
    #[route(E)]
    e: Channel,
    #[route(F)]
    f: Channel,
    #[route(G)]
    g: Channel,
}

#[derive(Role)]
#[message(Label)]
struct B {
    #[route(A)]
    a: Channel,
    #[route(C)]
    c: Channel,
    #[route(D)]
    d: Channel,
    #[route(E)]
    e: Channel,
    #[route(F)]
    f: Channel,
    #[route(G)]
    g: Channel,
}

#[derive(Role)]
#[message(Label)]
struct C {
    #[route(A)]
    a: Channel,
    #[route(B)]
    b: Channel,
    #[route(D)]
    d: Channel,
    #[route(E)]
    e: Channel,
    #[route(F)]
    f: Channel,
    #[route(G)]
    g: Channel,
}

#[derive(Role)]
#[message(Label)]
struct D {
    #[route(A)]
    a: Channel,
    #[route(B)]
    b: Channel,
    #[route(C)]
    c: Channel,
    #[route(E)]
    e: Channel,
    #[route(F)]
    f: Channel,
    #[route(G)]
    g: Channel,
}

#[derive(Role)]
#[message(Label)]
struct E {
    #[route(A)]
    a: Channel,
    #[route(B)]
    b: Channel,
    #[route(C)]
    c: Channel,
    #[route(D)]
    d: Channel,
    #[route(F)]
    f: Channel,
    #[route(G)]
    g: Channel,
}

#[derive(Role)]
#[message(Label)]
struct F {
    #[route(A)]
    a: Channel,
    #[route(B)]
    b: Channel,
    #[route(C)]
    c: Channel,
    #[route(D)]
    d: Channel,
    #[route(E)]
    e: Channel,
    #[route(G)]
    g: Channel,
}

#[derive(Role)]
#[message(Label)]
struct G {
    #[route(A)]
    a: Channel,
    #[route(B)]
    b: Channel,
    #[route(C)]
    c: Channel,
    #[route(D)]
    d: Channel,
    #[route(E)]
    e: Channel,
    #[route(F)]
    f: Channel,
}

#[derive(Message, Copy, Clone)]
enum Label {
    ProposalA(ProposalA),
    ProposalG(ProposalG),
    ProposalB(ProposalB),
    ProposalC(ProposalC),
    ProposalD(ProposalD),
    ProposalE(ProposalE),
    ProposalF(ProposalF),
}

impl From<Label> for Value {
    fn from(label: Label) -> Value {
        match label {
            Label::ProposalA(payload) => payload.into(),
            Label::ProposalG(payload) => payload.into(),
            Label::ProposalB(payload) => payload.into(),
            Label::ProposalC(payload) => payload.into(),
            Label::ProposalD(payload) => payload.into(),
            Label::ProposalE(payload) => payload.into(),
            Label::ProposalF(payload) => payload.into(),
        }
    }
}


#[derive(Copy, Clone)]
struct ProposalA(i32);

impl From<ProposalA> for Value {
    fn from(value: ProposalA) -> Value {
        let ProposalA(val) = value;
        val
    }
}

#[derive(Copy, Clone)]
struct ProposalG(i32);

impl From<ProposalG> for Value {
    fn from(value: ProposalG) -> Value {
        let ProposalG(val) = value;
        val
    }
}

#[derive(Copy, Clone)]
struct ProposalB(i32);

impl From<ProposalB> for Value {
    fn from(value: ProposalB) -> Value {
        let ProposalB(val) = value;
        val
    }
}

#[derive(Copy, Clone)]
struct ProposalC(i32);

impl From<ProposalC> for Value {
    fn from(value: ProposalC) -> Value {
        let ProposalC(val) = value;
        val
    }
}

#[derive(Copy, Clone)]
struct ProposalD(i32);

impl From<ProposalD> for Value {
    fn from(value: ProposalD) -> Value {
        let ProposalD(val) = value;
        val
    }
}

#[derive(Copy, Clone)]
struct ProposalE(i32);

impl From<ProposalE> for Value {
    fn from(value: ProposalE) -> Value {
        let ProposalE(val) = value;
        val
    }
}

#[derive(Copy, Clone)]
struct ProposalF(i32);

impl From<ProposalF> for Value {
    fn from(value: ProposalF) -> Value {
        let ProposalF(val) = value;
        val
    }
}

#[session(Name, Value)]
type RingMaxA = Send<B, 'a', ProposalA, Tautology::<Name, Value, ProposalA>, Noop<Name, Value>, Receive<G, 'g', ProposalG, Tautology::<Name, Value, ProposalG>, Noop<Name, Value>, End>>;

#[session(Name, Value)]
type RingMaxB = Receive<A, 'a', ProposalA, Tautology::<Name, Value, ProposalA>, Noop<Name, Value>, Send<C, 'b', ProposalB, Or<ProposalB, Equal::<Value, char, ProposalB, Variable<'b'>, Variable<'a'>>, GTn::<Value, char, ProposalB, Variable<'b'>, Variable<'a'>>, Name, Value>, Noop<Name, Value>, End>>;

#[session(Name, Value)]
type RingMaxC = Receive<B, 'b', ProposalB, Tautology::<Name, Value, ProposalB>, Noop<Name, Value>, Send<D, 'c', ProposalC, Or<ProposalC, Equal::<Value, char, ProposalC, Variable<'c'>, Variable<'b'>>, GTn::<Value, char, ProposalC, Variable<'c'>, Variable<'b'>>, Name, Value>, Noop<Name, Value>, End>>;

#[session(Name, Value)]
type RingMaxD = Receive<C, 'c', ProposalC, Tautology::<Name, Value, ProposalC>, Noop<Name, Value>, Send<E, 'd', ProposalD, Or<ProposalD, Equal::<Value, char, ProposalD, Variable<'d'>, Variable<'c'>>, GTn::<Value, char, ProposalD, Variable<'d'>, Variable<'c'>>, Name, Value>, Noop<Name, Value>, End>>;

#[session(Name, Value)]
type RingMaxE = Receive<D, 'd', ProposalD, Tautology::<Name, Value, ProposalD>, Noop<Name, Value>, Send<F, 'e', ProposalE, Or<ProposalE, Equal::<Value, char, ProposalE, Variable<'e'>, Variable<'d'>>, GTn::<Value, char, ProposalE, Variable<'e'>, Variable<'d'>>, Name, Value>, Noop<Name, Value>, End>>;

#[session(Name, Value)]
type RingMaxF = Receive<E, 'e', ProposalE, Tautology::<Name, Value, ProposalE>, Noop<Name, Value>, Send<G, 'f', ProposalF, Or<ProposalF, Equal::<Value, char, ProposalF, Variable<'f'>, Variable<'e'>>, GTn::<Value, char, ProposalF, Variable<'f'>, Variable<'e'>>, Name, Value>, Noop<Name, Value>, End>>;

#[session(Name, Value)]
type RingMaxG = Receive<F, 'f', ProposalF, Tautology::<Name, Value, ProposalF>, Noop<Name, Value>, Send<A, 'g', ProposalG, Or<ProposalG, Equal::<Value, char, ProposalG, Variable<'g'>, Variable<'f'>>, GTn::<Value, char, ProposalG, Variable<'g'>, Variable<'f'>>, Name, Value>, Noop<Name, Value>, End>>;
