#!/usr/bin/env python3
"""Create a signed Maven Central deployment bundle from the local repository."""

import hashlib
import os
import pathlib
import shutil
import subprocess
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent
VERSION = "0.2.0"
COORDINATE = pathlib.Path("io/github/lihao336991/renderkit-resvg-android") / VERSION
SOURCE = ROOT / "build/repository" / COORDINATE
STAGING = ROOT / "build/central-bundle" / COORDINATE
OUTPUT = ROOT / f"build/renderkit-resvg-android-{VERSION}-central-bundle.zip"


def digest(path, algorithm):
    value = hashlib.new(algorithm)
    value.update(path.read_bytes())
    return value.hexdigest()


def main():
    if not SOURCE.is_dir():
        raise RuntimeError("Run ./gradlew publishReleasePublicationToLocalReleaseRepository first")
    staging_root = ROOT / "build/central-bundle"
    if staging_root.exists():
        shutil.rmtree(staging_root)
    STAGING.mkdir(parents=True)
    names = [
        f"renderkit-resvg-android-{VERSION}.aar",
        f"renderkit-resvg-android-{VERSION}.pom",
        f"renderkit-resvg-android-{VERSION}-sources.jar",
        f"renderkit-resvg-android-{VERSION}-javadoc.jar",
    ]
    key = os.environ.get("RENDERKIT_SIGNING_KEY")
    for name in names:
        source = SOURCE / name
        destination = STAGING / name
        shutil.copy2(source, destination)
        command = ["gpg", "--batch", "--yes", "--armor", "--detach-sign"]
        if key:
            command.extend(["--local-user", key])
        subprocess.run([*command, str(destination)], check=True)
        for algorithm in ("md5", "sha1", "sha256", "sha512"):
            destination.with_name(destination.name + f".{algorithm}").write_text(digest(destination, algorithm))
    if OUTPUT.exists():
        OUTPUT.unlink()
    with zipfile.ZipFile(OUTPUT, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(staging_root.rglob("*")):
            if path.is_file():
                archive.write(path, path.relative_to(staging_root))
    print(OUTPUT)


if __name__ == "__main__":
    main()
