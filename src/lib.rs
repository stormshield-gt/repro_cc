#[uniffi::export]
fn repro() {
    use ring::agreement;
    let rng = ring::rand::SystemRandom::new();
    agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng).unwrap();
}

// #[uniffi::export]
// fn repro2() {
//     use aws_lc_rs::agreement;
//     let rng = aws_lc_rs::rand::SystemRandom::new();
//     agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng).unwrap();
// }

uniffi::setup_scaffolding!();
