// global protocol SleepAdvisor(role C, role A)
// {
//     Order(S) from C to A;
//     quote(i32) from A to C;
                                                   
//     choice at C
//     {
//         accept() from C to A;
//         address(String) from C to A;
//         date(String) from A to C;
//     }
//     or
//     {
//         reject() from C to A;
//     }
// }


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
    a: A,
}

#[derive(Role)]
#[message(Label)]
struct C {
    #[route(A)]
    a: Channel,
}

#[derive(Role)]
#[message(Label)]
struct A {
    #[route(C)]
    c: Channel,
}

#[derive(Message)]
enum Label {
    Order(Order),
    Quote(Quote),
    Accept(Accept),
    Reject(Reject),
    Address(Address),
    Date(Date),
}

struct Order(String);

struct Quote(i32);

struct Accept;

struct Reject;

struct Address(String);

struct Date(String);

#[session]
type TravelAgencyC = Send<A, Order, Receive<A, Quote, Select<A, TravelAgencyC2>>>;

#[session]
enum TravelAgencyC2 {
    Reject(Reject, End),
    Accept(Accept, Send<A, Address, Receive<A, Date, End>>),
}

#[session]
type TravelAgencyA = Receive<C, Order, Send<C, Quote, Branch<C, TravelAgencyA2>>>;

#[session]
enum TravelAgencyA2 {
    Reject(Reject, End),
    Accept(Accept, Receive<C, Address, Send<C, Date, End>>),
}

// global protocol SleepAdvisor(role C, role A)
// {
//     Order(S) from C to A;
//     quote(i32) from A to C;
                                                   
//     choice at C
//     {
//         accept() from C to A;
//         address(String) from C to A;
//         date(String) from A to C;
//     }
//     or
//     {
//         reject() from C to A;
//     }
// }

// script compare
// 1st normal
// 2nd with refinement
// 

// generation 
// unroll

// execution -- more important

async fn c(role: &mut C) -> Result<(), Box<dyn Error>> {
    try_session(role, |s: TravelAgencyC<'_, _>| async {
        let s = s.send(Order("XX".to_string())).await?;
        let (Quote(n), s) = s.receive().await?;
        if n < 80 {
            // Accept command if both prices are the same
            let s = s.select(Accept).await?;
            let s = s.send(Address("detailed_address".to_string())).await?;
            let (Date(date), s) = s.receive().await?;
            println!("Client: Accept order (price {}, Date {})", n, date);
            Ok(((), s))
        } else {
            let s = s.select(Reject).await?;
            println!("Client: Reject order (price inconsistency {} vs {})", n, 10);
            Ok(((), s))
        }
    })
    .await
}

async fn a(role: &mut A) -> Result<(), Box<dyn Error>> {
    try_session(role, |s: TravelAgencyA<'_, _>| async {
        let (Order(order), s) = s.receive().await?;
        let s = s.send(Quote(70)).await?;
        match s.branch().await? {
            TravelAgencyA2::Accept(_, s) => {
                let (Address(addr), s) = s.receive().await?;
                let s = s.send(Date("June 7th".to_string())).await?;
                println!("Agency: Reveive order (place {}, Customer address {})", order, addr);
                Ok(((), s))
            }
            TravelAgencyA2::Reject(_, end) => Ok(((), end)),
        }
    })
    .await
}

fn main() {
    let mut roles = Roles::default();
    executor::block_on(async {
        try_join!(c(&mut roles.c), a(&mut roles.a)).unwrap();
    });
}
