// global protocol Protocol(role Sensor, role Client, role Server)
// {
//     sensor_reading(i32) from Sensor to Server;
//     date(i32) from Sensor to Server;
                                           
//     choice at Server
//     {
//         alarm(i32) from Server to Client;
//         register_highBP(i32) from Server to Sensor;
//     }
//     or
//     {
//         normal(i32) from Server to Client;
//         normal(i32) from Server to Sensor;
//         do Protocol(Sensor, Client, Server);
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
    server: Server,
    client: Client,
    sensor: Sensor,
}

#[derive(Role)]
#[message(Label)]
struct Server {
    #[route(Client)]
    client: Channel,
    #[route(Sensor)]
    sensor: Channel,
}

#[derive(Role)]
#[message(Label)]
struct Client {
    #[route(Server)]
    server: Channel,
    #[route(Sensor)]
    sensor: Channel,
}

#[derive(Role)]
#[message(Label)]
struct Sensor {
    #[route(Server)]
    server: Channel,
    #[route(Client)]
    client: Channel,
}

#[derive(Message)]
enum Label {
    SensorReading(SensorReading),
    Date(Date),
    Alarm(Alarm),
    Normal(Normal),
    RegisterHighBp(RegisterHighBp),
}

struct SensorReading(i32);

struct Date(i32);

struct Alarm(i32);

struct Normal(i32);

struct RegisterHighBp(i32);

#[session]
type DiabetesServer = Receive<Sensor, SensorReading, Receive<Sensor, Date, Select<Client, DiabetesServer3>>>;

#[session]
enum DiabetesServer3 {
    Normal(Normal, Send<Sensor, Normal, DiabetesServer>),
    Alarm(Alarm, Send<Sensor, RegisterHighBp, End>),
}

#[session]
type DiabetesClient = Branch<Server, DiabetesClient0>;

#[session]
enum DiabetesClient0 {
    Alarm(Alarm, End),
    Normal(Normal, Branch<Server, DiabetesClient0>),
}

#[session]
type DiabetesSensor = Send<Server, SensorReading, Send<Server, Date, Branch<Server, DiabetesSensor3>>>;

#[session]
enum DiabetesSensor3 {
    RegisterHighBp(RegisterHighBp, End),
    Normal(Normal, DiabetesSensor),
}


async fn Server(role: &mut Server) -> Result<(), Box<dyn Error>> {
    try_session(role, |mut s: DiabetesServer<'_, _>| async {
        loop {
            let (SensorReading(n), s_rec) = s.receive().await?;
            let (Date(d), s_rec1) = s_rec.receive().await?;
            if n > 130 { // When blood presure > 130, register highBP.
                let s_en = s_rec1.select(Alarm(d)).await?;
                let s_end = s_en.send(RegisterHighBp(d)).await?;
                return Ok(((), s_end));
            } else {
                let s_sel = s_rec1.select(Normal(d)).await?;
                s = s_sel.send(Normal(d)).await?;
            }
        }
    }).await
}

async fn Sensor(role: &mut Sensor) -> Result<(), Box<dyn Error>> {
    let mut date = 0;
    try_session(role, |mut s: DiabetesSensor<'_, _>| async {
        loop {
            date += 1;
            let s_send = if date == 10 { // Testing for the benchmarks. Day 1-9 (normal). Day 10 (highBP).
                 s.send(SensorReading(140)).await?
            } else {
                s.send(SensorReading(100)).await?
            }; 
            let s_send = s_send.send(Date(date)).await?;
            match s_send.branch().await? {
                DiabetesSensor3::RegisterHighBp(_, s_bra) => {
                    println!("Sensor register high blood pressure!");
                    return Ok(((), s_bra));
                }
                DiabetesSensor3::Normal(_, s_bra) => {
                    println!("Sensor read normal.");
                    s = s_bra;
                }
            }
        }
    }).await
}

async fn Client(role: &mut Client) -> Result<(), Box<dyn Error>> {
    try_session(role, |mut s: DiabetesClient<'_, _>| async {
        loop {
            match s.branch().await? {
                DiabetesClient0::Alarm(_, s_bra) => {
                    println!("Alarm high blood pressure!");
                    return Ok(((), s_bra));
                }
                DiabetesClient0::Normal(_, s_bra) => {
                    println!("Client has normal blood pressure.");
                    s = s_bra;
                }
            }
        }
    }).await
}
            

fn main() {
    let mut roles = Roles::default();
    executor::block_on(async{
        try_join!(Client(&mut roles.client), Server(&mut roles.server), Sensor(&mut roles.sensor)).unwrap();
    });
}
