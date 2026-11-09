# Changelog

## iOS 0.1.1 / Android 0.2.1

- Resolve missing glyphs from eligible system fonts through the existing C and
  JNI interfaces.
- Bound system font file size, cached mappings, and fallback lookups.
- Include the new transitive dependency notices.
- Publish Android 0.2.1 on 2026-11-05 and iOS 0.1.1 on 2026-11-09.

## 0.2.0 - 2026-09-21

- Add a public Android AAR with arm64-v8a and x86_64 JNI libraries, a stable
  Java binding, Maven license/SCM metadata and bundled redistribution notices.
- Enforce the 3,145,728 total-pixel budget and convert Rust panics into ABI
  error code 3.
- Keep all generated native binaries out of the source repository.

## 0.1.0 - 2026-09-21

- Add the initial iOS XCFramework with arm64 device and arm64/x86_64 simulator slices.
- Expose ABI version 1 and constrained in-memory SVG rendering.
- Pin resvg 0.48.1 and include complete redistribution notices.
