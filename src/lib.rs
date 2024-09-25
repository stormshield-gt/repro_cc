use tracing_oslog::OsLogger;
use tracing_subscriber::layer::SubscriberExt;

#[uniffi::export]
fn repro() {
    use ring::agreement;
    let collector =
        tracing_subscriber::registry().with(OsLogger::new("moe.absolucy.test", "default"));
    tracing::subscriber::set_global_default(collector).expect("failed to set global subscriber");
    tracing::info!("foo");
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
