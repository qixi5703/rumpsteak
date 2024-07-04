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
    g: G,
}

#[derive(Role)]
#[message(Label)]
struct A {
    #[route(B)]
    b: Channel,
    #[route(G)]
    g: Channel,
}

#[derive(Role)]
#[message(Label)]
struct B {
    #[route(A)]
    a: Channel,
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
}

#[derive(Message, Copy, Clone)]
enum Label {
    GeneratorA(GeneratorA),
    GeneratorB(GeneratorB),
    PrimeA(PrimeA),
    PrimeB(PrimeB),
    PrivateKeyA(PrivateKeyA),
    SharedA(SharedA),
    SharedB(SharedB),
    PrivateKeyB(PrivateKeyB),
}

impl From<Label> for Value {
    fn from(label: Label) -> Value {
        match label {
            Label::GeneratorA(payload) => payload.into(),
            Label::GeneratorB(payload) => payload.into(),
            Label::PrimeA(payload) => payload.into(),
            Label::PrimeB(payload) => payload.into(),
            Label::PrivateKeyA(payload) => payload.into(),
            Label::SharedA(payload) => payload.into(),
            Label::SharedB(payload) => payload.into(),
            Label::PrivateKeyB(payload) => payload.into(),
        }
    }
}


#[derive(Copy, Clone)]
struct GeneratorA(i32);

impl From<GeneratorA> for Value {
    fn from(value: GeneratorA) -> Value {
        let GeneratorA(val) = value;
        val
    }
}

#[derive(Copy, Clone)]
struct GeneratorB(i32);

impl From<GeneratorB> for Value {
    fn from(value: GeneratorB) -> Value {
        let GeneratorB(val) = value;
        val
    }
}

#[derive(Copy, Clone)]
struct PrimeA(i32);

impl From<PrimeA> for Value {
    fn from(value: PrimeA) -> Value {
        let PrimeA(val) = value;
        val
    }
}

#[derive(Copy, Clone)]
struct PrimeB(i32);

impl From<PrimeB> for Value {
    fn from(value: PrimeB) -> Value {
        let PrimeB(val) = value;
        val
    }
}

#[derive(Copy, Clone)]
struct PrivateKeyA(i32);

impl From<PrivateKeyA> for Value {
    fn from(value: PrivateKeyA) -> Value {
        let PrivateKeyA(val) = value;
        val
    }
}

#[derive(Copy, Clone)]
struct SharedA(i32);

impl From<SharedA> for Value {
    fn from(value: SharedA) -> Value {
        let SharedA(val) = value;
        val
    }
}

#[derive(Copy, Clone)]
struct SharedB(i32);

impl From<SharedB> for Value {
    fn from(value: SharedB) -> Value {
        let SharedB(val) = value;
        val
    }
}

#[derive(Copy, Clone)]
struct PrivateKeyB(i32);

impl From<PrivateKeyB> for Value {
    fn from(value: PrivateKeyB) -> Value {
        let PrivateKeyB(val) = value;
        val
    }
}

#[session(Name, Value)]
type DiffieHellmanA = Receive<G, 'g', GeneratorA, Tautology::<Name, Value, GeneratorA>, Constant<Name, Value>, Send<B, 'h', GeneratorB, Equal::<Value, char, GeneratorB, Variable<'h'>, Variable<'g'>>, Constant<Name, Value>, Receive<G, 'p', PrimeA, Tautology::<Name, Value, PrimeA>, Constant<Name, Value>, Send<B, 'q', PrimeB, Equal::<Value, char, PrimeB, Variable<'q'>, Variable<'p'>>, Constant<Name, Value>, Receive<G, 'a', PrivateKeyA, Tautology::<Name, Value, PrivateKeyA>, Constant<Name, Value>, Send<B, 'A', SharedA, Equal::<Value, char, SharedA, Variable<'A'>, Modulo<Exponent<Variable<'g'>, Variable<'a'>>, Variable<'p'>>>, Constant<Name, Value>, Receive<B, 'B', SharedB, Tautology::<Name, Value, SharedB>, Constant<Name, Value>, End>>>>>>>;

#[session(Name, Value)]
type DiffieHellmanB = Receive<A, 'h', GeneratorB, Tautology::<Name, Value, GeneratorB>, Constant<Name, Value>, Receive<A, 'q', PrimeB, Tautology::<Name, Value, PrimeB>, Constant<Name, Value>, Receive<G, 'b', PrivateKeyB, Tautology::<Name, Value, PrivateKeyB>, Constant<Name, Value>, Receive<A, 'A', SharedA, Tautology::<Name, Value, SharedA>, Constant<Name, Value>, Send<A, 'B', SharedB, Equal::<Value, char, SharedB, Variable<'B'>, Modulo<Exponent<Variable<'h'>, Variable<'b'>>, Variable<'q'>>>, Constant<Name, Value>, End>>>>>;

#[session(Name, Value)]
type DiffieHellmanG = Send<A, 'g', GeneratorA, Tautology::<Name, Value, GeneratorA>, Constant<Name, Value>, Send<A, 'p', PrimeA, Tautology::<Name, Value, PrimeA>, Constant<Name, Value>, Send<A, 'a', PrivateKeyA, Tautology::<Name, Value, PrivateKeyA>, Constant<Name, Value>, Send<B, 'b', PrivateKeyB, Tautology::<Name, Value, PrivateKeyB>, Constant<Name, Value>, End>>>>;
