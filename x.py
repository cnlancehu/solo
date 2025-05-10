import os
import platform
import subprocess
import sys
import zipfile
import requests

app_name = "solo"

targets = {
    "Windows": {
        "x86_64-pc-windows-msvc": "windows-x64",
        "i686-pc-windows-msvc": "windows-x86",
        "aarch64-pc-windows-msvc": "windows-arm64"
    },
    "Linux": {
        "i686-linux-android": "android-x86",
        "aarch64-linux-android": "android-arm64",
        "x86_64-linux-android": "android-x64",
        "armv7-linux-androideabi": "android-armv7",
        
        "i686-unknown-linux-gnu": "linux-x86",
        "x86_64-unknown-linux-gnu": "linux-x64",
        "aarch64-unknown-linux-gnu": "linux-arm64"
    },
    "Darwin": {
        "x86_64-apple-darwin": "macos-intel",
        "aarch64-apple-darwin": "macos-silicon"
    }
}

def exec(cmd):
    if platform.system() == "Linux":
        ndk_home = os.environ.get("ANDROID_NDK_HOME")
        ndk_home = os.path.join(ndk_home, "toolchains", "llvm", "prebuilt", "linux-x86_64", "bin")
        os.environ["PATH"] = f"{ndk_home}:{os.environ['PATH']}"

        os.environ["CC_i686_linux_android"] = os.path.join(ndk_home, "i686-linux-android21-clang")
        os.environ["CC_x86_64_linux_android"] = os.path.join(ndk_home, "x86_64-linux-android21-clang")
        os.environ["CC_aarch64_linux_android"] = os.path.join(ndk_home, "aarch64-linux-android21-clang")
        os.environ["CC_armv7_linux_androideabi"] = os.path.join(ndk_home, "armv7a-linux-androideabi21-clang")

        result = subprocess.Popen(cmd, stdout=subprocess.PIPE, text=True, shell=True, env=os.environ).wait()
    else:
        result = subprocess.Popen(cmd, stdout=subprocess.PIPE, text=True, shell=True).wait()
    
    if result != 0:
        sys.exit(result)

def ci_build():
    os_type = platform.system()
    os.makedirs("dist", exist_ok=True)
    version = os.environ.get("VERSION")
    token = os.environ.get("TOKEN")

    if os_type == "Windows":
        os.environ["RUSTFLAGS"] = "-C target-feature=+crt-static -C link-arg=/DEBUG:NONE"
    else:
        os.environ["RUSTFLAGS"] = "-C target-feature=+crt-static"

    if os_type == "Linux":
        subprocess.Popen("sudo apt update", stdout=subprocess.PIPE, text=True, shell=True).wait()
        subprocess.Popen("sudo apt install -y gcc-aarch64-linux-gnu", stdout=subprocess.PIPE, text=True, shell=True).wait()
        subprocess.Popen("sudo apt install -y gcc-i686-linux-gnu", stdout=subprocess.PIPE, text=True, shell=True).wait()
    
    for target, alias in targets[os_type].items():
        exec(f"rustup target add {target}")
        exec(f"cargo build -r --target {target}")
        with zipfile.ZipFile(os.path.join("dist", f"{app_name}-{alias}.zip"), "w") as zipf:
            if os_type == "Windows":
                app_name_with_extension = f"{app_name}.exe"
            else:
                app_name_with_extension = app_name
            zipf.write(os.path.join("target", target, "release", app_name_with_extension), arcname=app_name_with_extension, compresslevel=3)
            os_name, arch = alias.split("-")
        headers = {
            'token': token,
            'user-agent': 'Lance Dev',
        }
        response1 = requests.request("POST", f"https://pkg.lance.fun/upload?{app_name}+{version}+{os_name}-{arch}+zip+{app_name}.zip", headers=headers, data=open(os.path.join("dist", f"{app_name}-{alias}.zip"), 'rb'))
        response2 = requests.request("POST", f"https://pkg.lance.fun/upload?{app_name}+{version}+{os_name}-{arch}+files+{app_name_with_extension}", headers=headers, data=open(os.path.join("target", target, "release", app_name_with_extension), 'rb'))
        if response1.ok and response2.ok:
            print("Successfully uploaded")
        else:
            print("Failed to upload")
            print(response1.text)
            print(response2.text)

if __name__ == "__main__":
    if sys.argv[1] == "ci":
        ci_build()