use std::{env, path::PathBuf};

fn main() {
    if env::var("CARGO_CFG_TARGET_VENDOR").expect("failed to get target vendor") != "apple" {
        println!("cargo:warning=tracing-oslog is only available for Apple platforms, it will not log anything on other platforms!");
        return;
    }

    let mut args = Vec::<String>::new();
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("failed to get target os");
    if target_os == "ios" {
        let version = if env::var("CARGO_CFG_TARGET_ABI").unwrap_or_default() == "macabi" {
            "14.0"
        } else {
            "10.0"
        };
        args.push(format!("-miphoneos-version-min={version}"));
    } else if target_os == "macos" {
        args.push("-mmacosx-version-min=10.12".to_owned());
    }

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type("os_log_.*")
        .allowlist_function("os_log_create")
        .allowlist_function("os_release")
        .allowlist_function("wrapped_.*")
        .clang_args(&args)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Couldn't write bindings!");

    cc::Build::new().file("wrapper.c").compile("wrapper");
}
