fn main() {
    let time = chrono::Utc::now()
        .format("%Y/%m/%d %H:%M:%S UTC")
        .to_string();
    println!("cargo:rustc-env=SOLO_BUILD_TIME={time}");

    let target = std::env::var("TARGET").unwrap();

    let (os_display, arch_display, target_name) = match target.as_str() {
        "x86_64-pc-windows-msvc" => ("Windows", "x64", "windows-x64"),
        "i686-pc-windows-msvc" => ("Windows", "x86", "windows-x86"),
        "aarch64-pc-windows-msvc" => ("Windows", "ARM64", "windows-arm64"),

        "i686-linux-android" => ("Android", "x86", "android-x86"),
        "aarch64-linux-android" => ("Android", "ARM64", "android-arm64"),
        "x86_64-linux-android" => ("Android", "x64", "android-x64"),
        "armv7-linux-androideabi" => ("Android", "ARMv7", "android-armv7"),
        "i686-unknown-linux-gnu" => ("Linux", "x86", "linux-x86"),
        "x86_64-unknown-linux-gnu" => ("Linux", "x64", "linux-x64"),
        "aarch64-unknown-linux-gnu" => ("Linux", "ARM64", "linux-arm64"),

        "x86_64-apple-darwin" => ("Darwin", "Intel", "macos-intel"),
        "aarch64-apple-darwin" => ("Darwin", "Apple Silicon", "macos-silicon"),

        _ => ("Unknown", "unknown", "unknown"),
    };

    println!("cargo:rustc-env=SOLO_TARGET_OS_DISPLAY={os_display}");
    println!("cargo:rustc-env=SOLO_TARGET_ARCH_DISPLAY={arch_display}");
    println!("cargo:rustc-env=SOLO_TARGET={target_name}");
}
