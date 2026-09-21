# RenderKit Resvg

`RenderKitResvg` is a constrained C ABI wrapper around
[resvg](https://github.com/linebender/resvg). It accepts in-memory SVG and an
optional in-memory font, renders into a caller-owned premultiplied RGBA buffer,
and never loads system fonts, local files, or remote resources.

The initial API intentionally rejects `image`, `filter`, `script`, and
`foreignObject`, limits SVG/font input sizes and output dimensions, and catches
Rust panics at the ABI boundary.

## CocoaPods

```ruby
pod 'RenderKitResvg', '0.1.0'
```

The Pod contains a static `RenderKitResvg.xcframework` for iOS 15 or later,
including arm64 device and arm64/x86_64 simulator slices. Its public C API is
declared in `renderkit_resvg.h`.

## Android

```kotlin
implementation("io.github.lihao336991:renderkit-resvg-android:0.2.0")
```

The public AAR contains a small Java binding, arm64-v8a and x86_64 JNI
libraries, the wrapper licenses, Rust dependency notices and Android NDK
notices. The shared objects are built from the pinned source during release;
they are never committed to this source repository.

## Build from source

Install Rust 1.89.0 and the Apple targets, then run:

```sh
rustup target add --toolchain 1.89.0 \
  aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
python3 build_ios.py
python3 build_android.py
./gradlew assemble
```

The generated XCFramework is written to
`artifacts/RenderKitResvg.xcframework`. Build outputs are not committed.
Android outputs are written to `build/` and are likewise ignored.

## License

The wrapper is licensed under either Apache-2.0 or MIT, at your option. resvg
and all statically linked dependencies retain their own licenses; the complete
redistribution notices are included under `Notices/` and in the CocoaPod.
