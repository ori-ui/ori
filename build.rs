fn main() {
    // if we're targeting mobile, enable the mobile cfg flag
    #[cfg(any(target_os = "android", target_os = "ios"))]
    println!("cargo:rustc-cfg=mobile");

    // if we're targeting desktop, enable the desktop cfg flag
    #[cfg(any(target_family = "windows", target_family = "unix"))]
    println!("cargo:rustc-cfg=desktop");

    // if we're targeting web, enable the web cfg flag
    #[cfg(target_family = "wasm")]
    println!("cargo:rustc-cfg=web");
}
