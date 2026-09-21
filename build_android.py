#!/usr/bin/env python3
"""Build Android JNI libraries from the pinned Rust and NDK toolchains."""

import hashlib
import json
import os
import pathlib
import shutil
import subprocess

ROOT = pathlib.Path(__file__).resolve().parent
OUT = ROOT / "build/android"
NDK_VERSION = "27.2.12479018"


def run(*args):
    subprocess.run([str(arg) for arg in args], cwd=ROOT, check=True)


def ndk_root():
    configured = os.environ.get("ANDROID_NDK_HOME")
    if configured:
        return pathlib.Path(configured)
    sdk = pathlib.Path(os.environ.get("ANDROID_HOME", pathlib.Path.home() / "Library/Android/sdk"))
    return sdk / "ndk" / NDK_VERSION


def main():
    ndk = ndk_root()
    if not ndk.is_dir():
        raise RuntimeError(f"Android NDK {NDK_VERSION} not found at {ndk}")
    host = "darwin-x86_64" if os.uname().sysname == "Darwin" else "linux-x86_64"
    toolchain = ndk / "toolchains/llvm/prebuilt" / host / "bin"
    if OUT.exists():
        shutil.rmtree(OUT)
    checksums = []
    for abi, rust_target in (("arm64-v8a", "aarch64-linux-android"), ("x86_64", "x86_64-linux-android")):
        run("cargo", "build", "--locked", "--release", "--target", rust_target)
        archive = ROOT / "target" / rust_target / "release/librenderkit_resvg.a"
        destination = OUT / "jni" / abi / "librenderkit_resvg_jni.so"
        destination.parent.mkdir(parents=True, exist_ok=True)
        run(toolchain / "clang++", f"--target={rust_target}26", f"--sysroot={toolchain.parent / 'sysroot'}",
            "-shared", "-fPIC", "-O2", "-fvisibility=hidden", "-static-libstdc++",
            "-Wl,--gc-sections", "-Wl,-z,max-page-size=16384", "-Wl,--exclude-libs,ALL",
            f"-I{ROOT / 'include'}", ROOT / "android/src/main/cpp/renderkit_resvg_jni.cpp",
            archive, "-ldl", "-llog", "-lm", "-o", destination)
        run(toolchain / "llvm-strip", "--strip-unneeded", destination)
        checksums.append({"path": str(destination.relative_to(OUT)), "bytes": destination.stat().st_size,
                          "sha256": hashlib.sha256(destination.read_bytes()).hexdigest()})
    notices = OUT / "notices"
    notices.mkdir(parents=True)
    shutil.copy2(ROOT / "Notices/THIRD-PARTY-NOTICES.txt", notices)
    shutil.copy2(ROOT / "Notices/RUST-STDLIB-NOTICES.html", notices)
    (notices / "ANDROID-NDK-NOTICES.txt").write_text(
        (ndk / "NOTICE").read_text() + "\n\n" + (ndk / "NOTICE.toolchain").read_text())
    (OUT / "checksums.json").write_text(json.dumps(checksums, indent=2) + "\n")


if __name__ == "__main__":
    main()
