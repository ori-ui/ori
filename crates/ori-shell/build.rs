fn main() {
    println!("cargo::rustc-check-cfg=cfg(x11_platform)");
    println!("cargo::rustc-check-cfg=cfg(wayland_platform)");
    println!("cargo::rustc-check-cfg=cfg(android_platform)");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if matches!(
        target_os.as_str(),
        "linux" | "freebsd" | "dragonfly" | "netbsd" | "openbsd"
    ) {
        #[cfg(feature = "x11")]
        println!("cargo:rustc-cfg=x11_platform");

        #[cfg(feature = "wayland")]
        println!("cargo:rustc-cfg=wayland_platform");
    }

    if target_os == "android" {
        println!("cargo:rustc-cfg=android_platform");
    }
}
