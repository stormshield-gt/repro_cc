#![cfg(any(target_os = "macos", target_os = "ios"))]

use tracing_os_log::OsLogLayer;
use tracing_subscriber::layer::SubscriberExt;

pub extern "C" fn repro() {
    let collector = tracing_subscriber::registry().with(OsLogLayer::new(c"com.bar.foo", c"foo"));

    tracing::subscriber::set_global_default(collector).expect("failed to set global subscriber");
    tracing::info!("foo");
}
