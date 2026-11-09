use super::{Face, FaceInfo, Font};
use std::{
    ffi::{c_char, c_int, c_void, CStr, OsStr},
    fs::File,
    io::Read,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::OnceLock,
};

type Handle = *mut c_void;
type Close = unsafe extern "C" fn(Handle);

#[link(name = "dl")]
extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> Handle;
    fn dlsym(handle: Handle, symbol: *const c_char) -> Handle;
}

struct SystemFont {
    languages: String,
    path: PathBuf,
    index: u32,
}

impl SystemFont {
    fn resolve(
        &self,
        character: char,
        load: &impl Fn(char, Font) -> Option<FaceInfo>,
    ) -> Option<FaceInfo> {
        load(
            character,
            Font {
                path: self.path.clone(),
                face: Face::Index(self.index),
            },
        )
    }

    fn matches(&self, language: &str) -> bool {
        self.languages
            .split(|c: char| c == ',' || c.is_ascii_whitespace())
            .any(|tag| {
                let base = tag.split('-').next().unwrap_or_default();
                base.eq_ignore_ascii_case(language)
                    || (language == "th" && tag.eq_ignore_ascii_case("und-Thai"))
            })
    }
}

fn enumerate_fonts() -> Option<Vec<SystemFont>> {
    // The iterator can also read system configuration in processes without
    // Android's initialized font collection. Symbols are present from API 29.
    let library = unsafe { dlopen(c"libandroid.so".as_ptr(), 2) };
    if library.is_null() {
        return None;
    }
    unsafe fn symbol<T: Copy>(library: Handle, name: &CStr) -> Option<T> {
        let address = dlsym(library, name.as_ptr());
        if address.is_null() {
            return None;
        }
        Some(std::mem::transmute_copy(&address))
    }
    unsafe {
        let open: unsafe extern "C" fn() -> Handle = symbol(library, c"ASystemFontIterator_open")?;
        let next: unsafe extern "C" fn(Handle) -> Handle =
            symbol(library, c"ASystemFontIterator_next")?;
        let close: Close = symbol(library, c"ASystemFontIterator_close")?;
        let font_close: Close = symbol(library, c"AFont_close")?;
        let path: unsafe extern "C" fn(Handle) -> *const c_char =
            symbol(library, c"AFont_getFontFilePath")?;
        let locale: unsafe extern "C" fn(Handle) -> *const c_char =
            symbol(library, c"AFont_getLocale")?;
        let index: unsafe extern "C" fn(Handle) -> usize =
            symbol(library, c"AFont_getCollectionIndex")?;
        let weight: unsafe extern "C" fn(Handle) -> u16 = symbol(library, c"AFont_getWeight")?;
        let italic: unsafe extern "C" fn(Handle) -> bool = symbol(library, c"AFont_isItalic")?;
        let iterator = open();
        if iterator.is_null() {
            return None;
        }
        let mut fonts = Vec::new();
        for _ in 0..4096 {
            let font = next(iterator);
            if font.is_null() {
                break;
            }
            let filename = path(font);
            if !filename.is_null() && weight(font) == 400 && !italic(font) {
                if let Ok(index) = u32::try_from(index(font)) {
                    let lang = locale(font);
                    let languages = if lang.is_null() {
                        String::new()
                    } else {
                        CStr::from_ptr(lang).to_string_lossy().into_owned()
                    };
                    let path =
                        PathBuf::from(OsStr::from_bytes(CStr::from_ptr(filename).to_bytes()));
                    fonts.push(SystemFont {
                        languages,
                        path,
                        index,
                    });
                }
            }
            font_close(font);
        }
        close(iterator);
        fonts.sort_by(|a, b| {
            (&a.path, a.index, &a.languages).cmp(&(&b.path, b.index, &b.languages))
        });
        fonts.dedup_by(|a, b| a.path == b.path && a.index == b.index && a.languages == b.languages);
        Some(fonts)
    }
}

pub(super) fn find(
    character: char,
    language: Option<&str>,
    resolve: impl Fn(char, Font) -> Option<FaceInfo>,
) -> Option<FaceInfo> {
    static FONTS: OnceLock<Vec<SystemFont>> = OnceLock::new();
    static CONFIG: OnceLock<Vec<SystemFont>> = OnceLock::new();
    let language = language.and_then(|tag| tag.split('-').next());
    let system = FONTS.get_or_init(|| enumerate_fonts().unwrap_or_default());
    let config = CONFIG.get_or_init(|| read_legacy_fonts().unwrap_or_default());
    system
        .iter()
        .chain(config)
        .filter(|font| language.is_some_and(|language| font.matches(language)))
        .take(8)
        .chain(config.iter().filter(|font| font.languages.is_empty()))
        .chain(
            system
                .iter()
                .filter(|font| font.languages.is_empty())
                .take(12),
        )
        .find_map(|font| font.resolve(character, &resolve))
}

fn read_legacy_fonts() -> Option<Vec<SystemFont>> {
    const MAX_XML: u64 = 1024 * 1024;
    let mut xml = String::new();
    File::open("/system/etc/fonts.xml")
        .ok()?
        .take(MAX_XML + 1)
        .read_to_string(&mut xml)
        .ok()?;
    if xml.len() as u64 > MAX_XML {
        return None;
    }
    let doc = roxmltree::Document::parse(&xml).ok()?;
    Some(
        doc.root_element()
            .children()
            .filter(|node| node.has_tag_name("family"))
            .filter_map(|family| {
                let languages = family.attribute("lang").unwrap_or("");
                if languages.is_empty() && family.attribute("name") != Some("sans-serif") {
                    return None;
                }
                let font = family.children().find(|node| {
                    node.has_tag_name("font")
                        && node.attribute("weight").unwrap_or("400") == "400"
                        && node.attribute("style").unwrap_or("normal") == "normal"
                        && node.attribute("fallbackFor").is_none()
                })?;
                let filename = font.text()?.trim();
                let path = Path::new(filename);
                if path.file_name()?.to_str()? != filename {
                    return None;
                }
                Some(SystemFont {
                    languages: languages.to_owned(),
                    path: Path::new("/system/fonts").join(path),
                    index: font.attribute("index").unwrap_or("0").parse().ok()?,
                })
            })
            .collect(),
    )
}

pub(super) fn is_system_path(path: &Path) -> bool {
    [
        "/system/fonts",
        "/product/fonts",
        "/system_ext/fonts",
        "/vendor/fonts",
        "/odm/fonts",
    ]
    .iter()
    .any(|directory| path.starts_with(directory))
}
