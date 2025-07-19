fn main() {
    let time = chrono::Utc::now()
        .format("%Y/%m/%d %H:%M:%S UTC")
        .to_string();
    println!("cargo:rustc-env=SOLO_BUILD_TIME={time}");

    println!(
        "cargo:rustc-env=SOLO_TARGET_OS={}",
        std::env::var("CARGO_CFG_TARGET_OS").unwrap()
    );
    println!(
        "cargo:rustc-env=SOLO_TARGET_ARCH={}",
        std::env::var("CARGO_CFG_TARGET_ARCH").unwrap()
    );

    let (os_display, arch_display) = {
        let target = std::env::var("TARGET").unwrap();
        match target.as_str() {
            "x86_64-pc-windows-msvc" => ("Windows", "x64"),
            "i686-pc-windows-msvc" => ("Windows", "x86"),
            "aarch64-pc-windows-msvc" => ("Windows", "ARM64"),

            "i686-linux-android" => ("Android", "x86"),
            "aarch64-linux-android" => ("Android", "ARM64"),
            "x86_64-linux-android" => ("Android", "x64"),
            "armv7-linux-androideabi" => ("Android", "ARMv7"),
            "i686-unknown-linux-gnu" => ("Linux", "x86"),
            "x86_64-unknown-linux-gnu" => ("Linux", "x64"),
            "aarch64-unknown-linux-gnu" => ("Linux", "ARM64"),

            "x86_64-apple-darwin" => ("Darwin", "Intel"),
            "aarch64-apple-darwin" => ("Darwin", "Apple Silicon"),

            _ => ("Unknown", "unknown"),
        }
    };
    println!("cargo:rustc-env=SOLO_TARGET_OS_DISPLAY={os_display}");
    println!("cargo:rustc-env=SOLO_TARGET_ARCH_DISPLAY={arch_display}");
}
