async fn a(role: &mut A) -> Result<(), Box<dyn Error>> {
    try_session(role, HashMap::new(), |s: DiffieHellmanA<'_, _>| async {
        use std::convert::TryInto;
        let (GeneratorA(g), s) = s.receive().await?;
        let s = s.send(GeneratorB(g)).await?;
        let (PrimeA(p), s) = s.receive().await?;
        let s = s.send(PrimeB(p)).await?;
        let (PrivateKeyA(a), s) = s.receive().await?;
        let shared_a = (g.pow(a.try_into()?))%p;
        println!("Sending partial shared key A {}", shared_a);
        let s = s.send(SharedA(shared_a)).await?;
        let (SharedB(shared_b), s) = s.receive().await?;
        println!("Participant A: Received partial shared key {} from B", shared_b);
        let secret = shared_b.pow(a);
        println!("Participant A: Secret shared key: {}", secret);
        Ok(((), s))
    })
    .await
}

async fn b(role: &mut B) -> Result<(), Box<dyn Error>> {
    try_session(role, HashMap::new(), |s: DiffieHellmanB<'_, _>| async {
        use std::convert::TryInto;
        let (GeneratorB(h), s) = s.receive().await?;
        let (PrimeB(q), s) = s.receive().await?;
        let (PrivateKeyB(b), s) = s.receive().await?;
        let (SharedA(A), s) = s.receive().await?;
        let s = s.send(SharedB((h.pow(b.try_into()?))%q)).await?;
        Ok(((), s))
    })
    .await
}

async fn g(role: &mut G) -> Result<(), Box<dyn Error>> {
    try_session(role, HashMap::new(), |s: DiffieHellmanG<'_, _>| async {
        // We take p = 7, g = 5
        // The secret keys are random values between 0 and 99;
        let p = 7;
        let g = 5;
        let a: i32 = (rand::random::<u8>()%10) as i32;
        let b: i32 = (rand::random::<u8>()%10) as i32;
        let s = s.send(GeneratorA(g)).await?;
        let s = s.send(PrimeA(p)).await?;
        let s = s.send(PrivateKeyA(a)).await?;
        let s = s.send(PrivateKeyB(b)).await?;
        Ok(((), s))
    })
    .await
}

fn main() {
    let mut roles = Roles::default();
    executor::block_on(async {
        try_join!(a(&mut roles.a), b(&mut roles.b), g(&mut roles.g)).unwrap();
    });
}
