#!/usr/bin/env python3
"""Build the RenderKitResvg XCFramework from the pinned Cargo.lock."""

import pathlib
import plistlib
import shutil
import subprocess

ROOT = pathlib.Path(__file__).resolve().parent
OUT = ROOT / "artifacts"
VERSION = "0.1.0"
MINIMUM_IOS = "15.0"


def run(*args):
    subprocess.run([str(arg) for arg in args], cwd=ROOT, check=True)


def cargo(target):
    run("cargo", "build", "--locked", "--release", "--target", target)
    return ROOT / "target" / target / "release" / "librenderkit_resvg.a"


def framework(folder, library, platform):
    destination = folder / "RenderKitResvg.framework"
    (destination / "Headers").mkdir(parents=True, exist_ok=True)
    (destination / "Modules").mkdir(exist_ok=True)
    shutil.copy2(library, destination / "RenderKitResvg")
    run("xcrun", "strip", "-S", destination / "RenderKitResvg")
    shutil.copy2(ROOT / "include/renderkit_resvg.h", destination / "Headers")
    (destination / "Modules/module.modulemap").write_text(
        'framework module RenderKitResvg {\n'
        '  umbrella header "renderkit_resvg.h"\n'
        '  export *\n'
        '}\n'
    )
    (destination / "Info.plist").write_bytes(plistlib.dumps({
        "CFBundleIdentifier": "com.renderkit.resvg",
        "CFBundleName": "RenderKitResvg",
        "CFBundleExecutable": "RenderKitResvg",
        "CFBundlePackageType": "FMWK",
        "CFBundleVersion": "1",
        "CFBundleShortVersionString": VERSION,
        "CFBundleSupportedPlatforms": [platform],
        "MinimumOSVersion": MINIMUM_IOS,
    }))
    return destination


def main():
    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir()

    device = cargo("aarch64-apple-ios")
    simulator_arm64 = cargo("aarch64-apple-ios-sim")
    simulator_x86_64 = cargo("x86_64-apple-ios")
    simulator = OUT / "simulator.a"
    run("lipo", "-create", simulator_arm64, simulator_x86_64, "-output", simulator)

    device_framework = framework(OUT / "device", device, "iPhoneOS")
    simulator_framework = framework(OUT / "simulator", simulator, "iPhoneSimulator")
    run(
        "xcodebuild", "-create-xcframework",
        "-framework", device_framework,
        "-framework", simulator_framework,
        "-output", OUT / "RenderKitResvg.xcframework",
    )


if __name__ == "__main__":
    main()
