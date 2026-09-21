// Copyright 2026 RenderKit contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use resvg::{tiny_skia, usvg};

const MAX_SOURCE: usize = 256 * 1024;
const MAX_FONT: usize = 16 * 1024 * 1024;
const MAX_DIMENSION: u32 = 2048;
const MAX_PIXELS: usize = 3 * 1024 * 1024;

fn required_output_len(width: u32, height: u32) -> Option<usize> {
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return None;
    }
    let pixels = (width as usize).checked_mul(height as usize)?;
    if pixels > MAX_PIXELS {
        return None;
    }
    pixels.checked_mul(4)
}

fn render(source: &[u8], font: &[u8], width: u32, height: u32) -> Result<tiny_skia::Pixmap, i32> {
    if source.len() > MAX_SOURCE || font.len() > MAX_FONT
        || required_output_len(width, height).is_none() { return Err(1); }
    let text = std::str::from_utf8(source).map_err(|_| 2)?;
    let doc = roxmltree::Document::parse(text).map_err(|_| 2)?;
    let mut count = 0;
    for node in doc.descendants() {
        count += 1;
        if count > 8192 || node.ancestors().take(66).count() > 64 { return Err(1); }
        // Initial candidate: no script, images, external resources or costly filter surfaces.
        if node.is_element() && matches!(node.tag_name().name(), "image" | "filter" | "script" | "foreignObject") {
            return Err(2);
        }
    }
    let mut options = usvg::Options::default();
    // Do not use default resolvers: the string resolver can read local files.
    options.image_href_resolver = usvg::ImageHrefResolver {
        resolve_data: Box::new(|_, _, _| None),
        resolve_string: Box::new(|_, _| None),
    };
    if !font.is_empty() { options.fontdb_mut().load_font_data(font.to_vec()); }
    let family = options.fontdb.faces().next().and_then(|face| face.families.first()).map(|f| f.0.clone());
    if let Some(family) = family {
        options.font_family = family.clone();
        options.fontdb_mut().set_sans_serif_family(family.clone());
        options.fontdb_mut().set_serif_family(family.clone());
        options.fontdb_mut().set_monospace_family(family);
    }
    // No system font discovery, disk access or bundled font licensing hidden in the SDK.
    if doc.descendants().any(|n| n.has_tag_name("text")) && options.fontdb.is_empty() { return Err(2); }
    let tree = usvg::Tree::from_str(text, &options).map_err(|_| 2)?;
    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or(1)?;
    let scale = (width as f32 / tree.size().width()).min(height as f32 / tree.size().height());
    let tx = (width as f32 - tree.size().width() * scale) / 2.0;
    let ty = (height as f32 - tree.size().height() * scale) / 2.0;
    resvg::render(&tree, tiny_skia::Transform::from_row(scale, 0.0, 0.0, scale, tx, ty), &mut pixmap.as_mut());
    Ok(pixmap)
}

#[no_mangle]
pub extern "C" fn rk_resvg_abi_version() -> u32 { 1 }

/// # Safety
/// All non-null pointers must be valid for their declared lengths, and output writable.
#[no_mangle]
pub unsafe extern "C" fn rk_resvg_render(source: *const u8, source_len: usize,
    font: *const u8, font_len: usize, width: u32, height: u32,
    output: *mut u8, output_len: usize) -> i32 {
    let Some(expected_output_len) = required_output_len(width, height) else { return 1; };
    if source.is_null() || output.is_null() || source_len > MAX_SOURCE || font_len > MAX_FONT
        || (font.is_null() && font_len != 0) || output_len != expected_output_len { return 1; }
    std::panic::catch_unwind(|| {
        let bytes = std::slice::from_raw_parts(source, source_len);
        let fonts = if font_len == 0 { &[] } else { std::slice::from_raw_parts(font, font_len) };
        #[cfg(test)]
        if bytes == b"__renderkit_test_panic__" { panic!("fault injection"); }
        match render(bytes, fonts, width, height) {
            Ok(pixmap) => { std::ptr::copy_nonoverlapping(pixmap.data().as_ptr(), output, output_len); 0 }
            Err(code) => code,
        }
    }).unwrap_or(3)
}

#[cfg(test)]
mod tests {
    use super::*;
    const MARKER: &[u8] = include_bytes!("../fixtures/defs-marker-end.svg");
    #[test]
    fn marker_has_arrowhead_pixels() {
        let pixmap = render(MARKER, &[], 200, 100).unwrap();
        assert!(pixmap.pixel(160, 42).unwrap().alpha() > 200, "arrowhead above shaft");
        assert_eq!(pixmap.pixel(30, 10).unwrap().alpha(), 0);
    }
    #[test]
    fn budgets_and_untrusted_resources_fail_closed() {
        assert!(render(MARKER, &[], 0, 100).is_err());
        assert!(render(MARKER, &[], 4096, 4096).is_err());
        assert_eq!(required_output_len(2048, 1536), Some(2048 * 1536 * 4));
        assert_eq!(required_output_len(2048, 1537), None);
        assert!(render(MARKER, &[], 2048, 2048).is_err());
        assert!(render(b"broken", &[], 100, 100).is_err());
        for tag in ["image", "filter", "script", "foreignObject"] {
            let xml = format!("<svg xmlns='http://www.w3.org/2000/svg'><{tag}/></svg>");
            assert!(render(xml.as_bytes(), &[], 100, 100).is_err());
        }
    }
    #[test]
    fn ffi_checks_buffer_and_renders_real_pixels() {
        let mut out = vec![0; 200 * 100 * 4];
        unsafe {
            assert_eq!(rk_resvg_render(MARKER.as_ptr(), MARKER.len(), std::ptr::null(), 0, 200, 100, out.as_mut_ptr(), 4), 1);
            assert_eq!(rk_resvg_render(MARKER.as_ptr(), MARKER.len(), std::ptr::null(), 0, 200, 100, out.as_mut_ptr(), out.len()), 0);
        }
        assert!(out.iter().any(|&v| v != 0));
    }
    #[test]
    fn ffi_turns_panics_into_an_error_code() {
        let source = b"__renderkit_test_panic__";
        let mut out = vec![0; 4];
        unsafe {
            assert_eq!(rk_resvg_render(source.as_ptr(), source.len(), std::ptr::null(), 0,
                1, 1, out.as_mut_ptr(), out.len()), 3);
        }
    }
}
