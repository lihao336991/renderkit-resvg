# RenderKit Resvg

`RenderKitResvg` is a constrained C ABI wrapper around
[resvg](https://github.com/linebender/resvg). It accepts in-memory SVG and an
optional in-memory primary font and renders into a caller-owned premultiplied
RGBA buffer. Current mobile source resolves missing glyphs from read-only
system fonts selected by Core Text or Android's system font configuration.
SVG input cannot name local resources or fetch remote resources. The published
iOS 0.1.1 and Android 0.2.1 versions include system font fallback.

The initial API intentionally rejects `image`, `filter`, `script`, and
`foreignObject`, limits SVG/font input sizes and output dimensions, and catches
Rust panics at the ABI boundary.

## CocoaPods

```ruby
pod 'RenderKitResvg', '0.1.1'
```

The Pod contains a static `RenderKitResvg.xcframework` for iOS 15 or later,
including arm64 device and arm64/x86_64 simulator slices. Its public C API is
declared in `renderkit_resvg.h`. When linking an XCFramework built from
current source directly, link Core Text and Core Foundation.

## Android

```kotlin
implementation("io.github.lihao336991:renderkit-resvg-android:0.2.1")
```

The public AAR contains a small Java binding, arm64-v8a and x86_64 JNI
libraries, the wrapper licenses, Rust dependency notices and Android NDK
notices. The shared objects are built from the pinned source during release;
they are never committed to this source repository.

## Font fallback limitations

- usvg may reshape an entire text run with the fallback font, including Latin
  characters that the supplied primary font already covers.
- Language selection uses the root `xml:lang` and document-wide script hints.
  Language declarations on individual `text` or `tspan` elements are not used.
- Each render permits 256 system font lookups. Repeated unsupported characters
  consume this budget and can prevent later characters from using system fonts.

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
