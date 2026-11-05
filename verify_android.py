#!/usr/bin/env python3
"""Run the published AAR binding and arm64 JNI library on a connected device."""

import argparse
import os
import pathlib
import shutil
import subprocess
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent
OUT = ROOT / "build/android-device-smoke"
VERSION = "0.2.1"


def run(*args):
    subprocess.run([str(arg) for arg in args], check=True)


def verify(sdk, serial):
    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir(parents=True)
    aar = ROOT / f"build/outputs/aar/renderkit-resvg-android-{VERSION}.aar"
    with zipfile.ZipFile(aar) as archive:
        archive.extract("classes.jar", OUT)
        so = OUT / "librenderkit_resvg_jni.so"
        so.write_bytes(archive.read("jni/arm64-v8a/librenderkit_resvg_jni.so"))
    classes = OUT / "classes"
    classes.mkdir()
    run("javac", "-source", "8", "-target", "8", "-cp", OUT / "classes.jar",
        "-d", classes, ROOT / "tests/JniSmoke.java")
    run("jar", "cf", OUT / "smoke.jar", "-C", classes, ".")
    run(sdk / "build-tools/35.0.0/d8", "--min-api", "26",
        "--lib", sdk / "platforms/android-35/android.jar", "--output", OUT / "classes.zip",
        OUT / "classes.jar", OUT / "smoke.jar")
    adb = [sdk / "platform-tools/adb", "-s", serial]
    remote = "/data/local/tmp/renderkit-resvg-public-smoke"
    run(*adb, "shell", "mkdir", "-p", remote)
    for source, name in ((OUT / "classes.zip", "classes.zip"),
                         (OUT / "librenderkit_resvg_jni.so", "librenderkit_resvg_jni.so"),
                         (ROOT / "fixtures/defs-marker-end.svg", "marker.svg")):
        run(*adb, "push", source, f"{remote}/{name}")
    run(*adb, "shell", f"CLASSPATH={remote}/classes.zip app_process "
        f"-Djava.library.path={remote} /system/bin "
        f"io.github.lihao336991.renderkit.resvg.JniSmoke {remote}/marker.svg")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--sdk", type=pathlib.Path,
                        default=os.environ.get("ANDROID_HOME", pathlib.Path.home() / "Library/Android/sdk"))
    parser.add_argument("--device", required=True)
    arguments = parser.parse_args()
    verify(arguments.sdk, arguments.device)
