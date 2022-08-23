// global protocol Authentication(role C, role S)
// {
//     password(i32) from C to S;
                                                   
//     choice at S
//     {
//         success(i32, i32) from S to C;
//     }
//     or
//     {
//         choice at S 
//         {
//             failure(i32) from S to C; if retry > 0
//             do Authentication(C, S);
//         }
//         or
//         {
//             abort(i32) from S to C;
//         }
        
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
    Retry(Retry),
    Password(Password),
    Fail(Fail),
    Succeed(Succeed),
    Abort(Abort),
}

struct Retry(i32);

struct Password(i32);

struct Fail(i32);

struct Succeed(i32);

struct Abort(i32);

#[session]
type AuthC = Receive<S, Retry, Send<S, Password, Branch<S, AuthC3>>>;

#[session]
enum AuthC3 {
    Abort(Abort, End),
    Succeed(Succeed, End),
    Fail(Fail, AuthC),
}

#[session]
type AuthS = Send<C, Retry, Receive<C, Password, Select<C, AuthS3>>>;

#[session]
enum AuthS3 {
    Abort(Abort, End),
    Succeed(Succeed, End),
    Fail(Fail, AuthS),
}

async fn C(role: &mut C) -> Result<(), Box<dyn Error>> {
    try_session(role, |s: AuthC<'_, _>| async {
        let mut s = s;
        let mut times = 10;
        loop {
            let (_, s_rec) = s.receive().await?;
            times = times -1;
            let s_send = if times > 0 {
                s_rec.send(Password(1)).await?
            } else {
                s_rec.send(Password(42)).await?
            };
            match s_send.branch().await? {
                AuthC3::Abort(_, s_bra) => {
                    println!("Login aborted");
                    return Ok(((), s_bra));
                }
                AuthC3::Succeed(_, s_bra) => {
                    println!("Login succeeded");
                    return Ok(((), s_bra));
                }
                AuthC3::Fail(_, s_bra) => {
                    println!("Login failed");
                    s = s_bra;
                }
            }
        }
    }).await
}
async fn S(role: &mut S) -> Result<(), Box<dyn Error>> {
    try_session(role, |mut s: AuthS<'_, _>| async {
        loop {
            let s_send = s.send(Retry(10)).await?;
            let (Password(n), s_rec) = s_send.receive().await?;
            if n == 42 {
                let s_end = s_rec.select(Succeed(0)).await?;
                return Ok(((), s_end));
            } else {
                s = s_rec.select(Fail(-1)).await?;
            }
        }
    }).await
}

fn main() {
    let mut roles = Roles::default();
    executor::block_on(async{
        try_join!(C(&mut roles.c), S(&mut roles.s)).unwrap();
    });
}
