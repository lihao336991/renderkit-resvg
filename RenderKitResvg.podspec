Pod::Spec.new do |spec|
  spec.name = 'RenderKitResvg'
  spec.version = '0.1.0'
  spec.summary = 'A constrained C ABI wrapper around resvg for iOS.'
  spec.description = <<-DESC
    RenderKitResvg packages resvg behind a small, fail-closed C ABI. It renders
    in-memory SVG and optional font data into a caller-owned RGBA buffer without
    loading system fonts, local files, or remote resources.
  DESC
  spec.homepage = 'https://github.com/lihao336991/renderkit-resvg'
  spec.author = { 'LiHao' => 'lihao.zzz@bytedance.com' }
  spec.license = { type: 'MIT OR Apache-2.0', file: 'LICENSE-MIT' }
  spec.source = {
    http: 'https://github.com/lihao336991/renderkit-resvg/releases/download/v0.1.0/RenderKitResvg-0.1.0.zip',
    sha256: '8f0651b207c091b0a15bded9827fd4576fa3917ea57189f20b8928d083d41ffc'
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
