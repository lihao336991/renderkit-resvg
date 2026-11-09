use super::{Face, FaceInfo, Font};
use std::{
    ffi::{c_char, c_void, CStr},
    path::{Path, PathBuf},
    ptr::{self, NonNull},
};

type CFRef = *const c_void;

#[repr(C)]
struct CFRange {
    location: isize,
    length: isize,
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFStringCreateWithCharacters(allocator: CFRef, chars: *const u16, length: isize) -> CFRef;
    fn CFStringGetCString(value: CFRef, buffer: *mut c_char, size: isize, encoding: u32) -> u8;
    fn CFGetTypeID(value: CFRef) -> usize;
    fn CFURLGetTypeID() -> usize;
    fn CFURLCopyFileSystemPath(url: CFRef, style: isize) -> CFRef;
    fn CFRelease(value: CFRef);
}

#[link(name = "CoreText", kind = "framework")]
extern "C" {
    static kCTFontURLAttribute: CFRef;
    fn CTFontCreateUIFontForLanguage(font_type: u32, size: f64, language: CFRef) -> CFRef;
    fn CTFontCreateForStringWithLanguage(
        font: CFRef,
        text: CFRef,
        range: CFRange,
        language: CFRef,
    ) -> CFRef;
    fn CTFontCopyAttribute(font: CFRef, attribute: CFRef) -> CFRef;
    fn CTFontCopyPostScriptName(font: CFRef) -> CFRef;
}

struct Owned(NonNull<c_void>);
impl Owned {
    fn new(reference: CFRef) -> Option<Self> {
        NonNull::new(reference.cast_mut()).map(Self)
    }

    fn as_ref(&self) -> CFRef {
        self.0.as_ptr()
    }

    fn from_utf16(chars: &[u16]) -> Option<Self> {
        Self::new(unsafe {
            CFStringCreateWithCharacters(ptr::null(), chars.as_ptr(), chars.len() as isize)
        })
    }

    fn utf8(&self) -> Option<String> {
        const UTF8_ENCODING: u32 = 0x0800_0100;
        let mut buffer = [0 as c_char; 4096];
        let ok = unsafe {
            CFStringGetCString(
                self.as_ref(),
                buffer.as_mut_ptr(),
                buffer.len() as isize,
                UTF8_ENCODING,
            )
        };
        if ok == 0 {
            return None;
        }
        unsafe { CStr::from_ptr(buffer.as_ptr()) }
            .to_str()
            .ok()
            .map(str::to_owned)
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        unsafe { CFRelease(self.as_ref()) }
    }
}

pub(super) fn find(
    character: char,
    language: Option<&str>,
    resolve: impl Fn(char, Font) -> Option<FaceInfo>,
) -> Option<FaceInfo> {
    let language =
        language.and_then(|tag| Owned::from_utf16(&tag.encode_utf16().collect::<Vec<_>>()));
    let language = language.as_ref().map_or(ptr::null(), Owned::as_ref);
    let mut storage = [0; 2];
    let utf16 = character.encode_utf16(&mut storage);
    let text = Owned::from_utf16(utf16)?;
    // kCTFontUIFontSystem is 2. Core Text supplies the fallback for the scalar,
    // and resvg still performs all shaping and positioning.
    let base = Owned::new(unsafe { CTFontCreateUIFontForLanguage(2, 16.0, language) })?;
    let range = CFRange {
        location: 0,
        length: utf16.len() as isize,
    };
    let font = Owned::new(unsafe {
        CTFontCreateForStringWithLanguage(base.as_ref(), text.as_ref(), range, language)
    })?;
    let name = Owned::new(unsafe { CTFontCopyPostScriptName(font.as_ref()) })?.utf8()?;
    let url = Owned::new(unsafe { CTFontCopyAttribute(font.as_ref(), kCTFontURLAttribute) })?;
    if unsafe { CFGetTypeID(url.as_ref()) != CFURLGetTypeID() } {
        return None;
    }
    let path = Owned::new(unsafe { CFURLCopyFileSystemPath(url.as_ref(), 0) })?.utf8()?;
    resolve(
        character,
        Font {
            path: PathBuf::from(path),
            face: Face::PostScriptName(name),
        },
    )
}

pub(super) fn is_system_path(path: &Path) -> bool {
    path.starts_with("/System/Library/Fonts")
        || (cfg!(target_abi = "sim")
            && path.starts_with("/Library/Developer/CoreSimulator")
            && path
                .to_str()
                .is_some_and(|p| p.contains("/RuntimeRoot/System/Library/Fonts/")))
}
