#[path = "build_support/deployment_target.rs"]
mod deployment_target;

use std::{env, process};

use deployment_target::{Version, parse};

const MINIMUM_MACOS: Version = Version::new(26, 1, 0);

fn main() {
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }

    let value = match env::var("MACOSX_DEPLOYMENT_TARGET") {
        Ok(value) => value,
        Err(_) => fail(
            "MACOSX_DEPLOYMENT_TARGET is missing; libneo consumers must set it to at least 26.1 \
             in .cargo/config.toml",
        ),
    };
    let version = match parse(&value) {
        Ok(version) => version,
        Err(reason) => fail(&format!(
            "MACOSX_DEPLOYMENT_TARGET={value:?} is invalid ({reason}); libneo requires 26.1 or later"
        )),
    };
    if version < MINIMUM_MACOS {
        fail(&format!(
            "MACOSX_DEPLOYMENT_TARGET={value:?} is too low; libneo requires 26.1 or later"
        ));
    }
}

fn fail(message: &str) -> ! {
    eprintln!("error: {message}");
    process::exit(1);
}
