use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("cargo::rustc-env=TARGET={target}");

    let profile = env::var("PROFILE").unwrap();
    println!("cargo::rustc-env=PROFILE={profile}");
}
