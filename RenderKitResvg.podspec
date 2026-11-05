Pod::Spec.new do |spec|
  spec.name = 'RenderKitResvg'
  spec.version = '0.1.1'
  spec.summary = 'A constrained C ABI wrapper around resvg for iOS.'
  spec.description = <<-DESC
    RenderKitResvg packages resvg behind a small, fail-closed C ABI. It renders
    in-memory SVG and optional primary font data into a caller-owned RGBA buffer.
    Missing glyphs can use read-only system fonts selected by Core Text.
  DESC
  spec.homepage = 'https://github.com/lihao336991/renderkit-resvg'
  spec.author = { 'LiHao' => 'lihao.zzz@bytedance.com' }
  spec.license = { type: 'MIT OR Apache-2.0', file: 'LICENSE-MIT' }
  spec.source = {
    http: 'https://github.com/lihao336991/renderkit-resvg/releases/download/v0.1.1/RenderKitResvg-0.1.1.zip',
    sha256: '56674b2ac3927a8cbc8b3391eefa78f8b18d89817d8edfc738b14ccc348095d0'
  }

  spec.ios.deployment_target = '15.0'
  spec.cocoapods_version = '>= 1.10'
  spec.vendored_frameworks = 'RenderKitResvg.xcframework'
  spec.frameworks = 'CoreFoundation', 'CoreText'
  spec.resource_bundles = {
    'RenderKitResvgNotices' => [
      'Notices/THIRD-PARTY-NOTICES.txt',
      'Notices/RUST-STDLIB-NOTICES.html'
    ]
  }
  spec.preserve_paths = ['LICENSE-APACHE', 'LICENSE-MIT', 'NOTICE']
end
