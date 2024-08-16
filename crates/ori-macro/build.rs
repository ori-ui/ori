use std::env;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(release_profile)");

    if dbg!(env::var("PROFILE")) == Ok(String::from("release")) {
        println!("cargo:rustc-cfg=release_profile");
    }
}
