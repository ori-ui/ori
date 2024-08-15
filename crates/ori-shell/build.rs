fn main() {
    println!("cargo::rustc-check-cfg=cfg(x11_platform)");
    println!("cargo::rustc-check-cfg=cfg(wayland_platform)");

    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
    ))]
    {
        #[cfg(feature = "x11")]
        println!("cargo:rustc-cfg=x11_platform");

        #[cfg(feature = "wayland")]
        println!("cargo:rustc-cfg=wayland_platform");
    }
}
