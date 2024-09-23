pub extern "C" fn repro() {
    use ring::agreement;
    let rng = ring::rand::SystemRandom::new();
    agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng).unwrap();
}

pub extern "C" fn repro2() {
    use aws_lc_rs::agreement;
    let rng = aws_lc_rs::rand::SystemRandom::new();
    agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng).unwrap();
}
