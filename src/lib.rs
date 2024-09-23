pub extern "C" fn repro() {
    use ring::agreement;
    let rng = ring::rand::SystemRandom::new();
    agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng).unwrap();
}
