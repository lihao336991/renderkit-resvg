// Gallery SVG-04: usvg's missing-character resolver loads only OS-selected system faces.
use memmap2::MmapOptions;
use resvg::usvg;
use skrifa::MetadataProvider;
use std::{
    collections::VecDeque,
    fs::File,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};
use unicode_script::{Script, UnicodeScript};
use usvg::fontdb::{Database, FaceInfo, Source};

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "android")]
use android as platform;
#[cfg(target_os = "ios")]
use ios as platform;

const MAX_SYSTEM_FILE: u64 = 64 * 1024 * 1024;
const MAX_CACHED_FILES: usize = 4;
const MAX_LOOKUPS_PER_RENDER: u16 = 256;

struct Font {
    path: PathBuf,
    face: Face,
}

enum Face {
    #[cfg(target_os = "android")]
    Index(u32),
    #[cfg(target_os = "ios")]
    PostScriptName(String),
}

impl Face {
    fn matches(&self, face: &FaceInfo) -> bool {
        match self {
            #[cfg(target_os = "android")]
            Self::Index(index) => face.index == *index,
            #[cfg(target_os = "ios")]
            Self::PostScriptName(name) => face.post_script_name == *name,
        }
    }
}

static FACES: Mutex<VecDeque<(PathBuf, Arc<[FaceInfo]>)>> = Mutex::new(VecDeque::new());

fn faces(path: &Path) -> Option<Arc<[FaceInfo]>> {
    let path = path.canonicalize().ok()?;
    if !platform::is_system_path(&path) {
        return None;
    }
    let mut cache = FACES.lock().ok()?;
    if let Some(position) = cache.iter().position(|(cached, _)| *cached == path) {
        let entry = cache.remove(position)?;
        let faces = Arc::clone(&entry.1);
        cache.push_back(entry);
        return Some(faces);
    }
    let file = File::open(&path).ok()?;
    let metadata = file.metadata().ok()?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_SYSTEM_FILE {
        return None;
    }
    // Only map canonical font files on OS-owned read-only partitions. The mapping
    // outlives the descriptor and may be shared by several active SVG trees.
    let bytes = unsafe { MmapOptions::new().map(&file).ok()? };
    let mut db = Database::new();
    db.load_font_source(Source::Binary(Arc::new(bytes)));
    let faces: Arc<[FaceInfo]> = db.faces().cloned().collect();
    if cache.len() == MAX_CACHED_FILES {
        cache.pop_front();
    }
    cache.push_back((path, Arc::clone(&faces)));
    Some(faces)
}

fn resolve(character: char, font: Font) -> Option<FaceInfo> {
    let candidates = faces(&font.path)?;
    let face = candidates.iter().find(|face| font.face.matches(face))?;
    let Source::Binary(bytes) = &face.source;
    let font_ref = skrifa::FontRef::from_index(bytes.as_ref().as_ref(), face.index).ok()?;
    font_ref.charmap().map(character)?;
    Some(face.clone())
}

pub(super) fn install<'a>(options: &mut usvg::Options<'a>, doc: &'a roxmltree::Document<'_>) {
    let root_language = doc
        .root_element()
        .attribute(("http://www.w3.org/XML/1998/namespace", "lang"))
        .filter(|tag| {
            !tag.is_empty()
                && tag.len() <= 35
                && tag.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        });
    let han_language = OnceLock::new();

    let default = usvg::FontResolver::default_fallback_selector();
    let lookups = Mutex::new(0u16);
    options.font_resolver.select_fallback = Box::new(move |character, excluded, db| {
        if let Some(id) = default(character, excluded, db) {
            return Some(id);
        }
        let mut remaining = lookups.lock().ok()?;
        if *remaining >= MAX_LOOKUPS_PER_RENDER {
            return None;
        }
        *remaining += 1;
        drop(remaining);
        let language = match character.script() {
            Script::Hiragana | Script::Katakana => Some("ja"),
            Script::Hangul => Some("ko"),
            Script::Thai => Some("th"),
            Script::Han => Some(*han_language.get_or_init(|| {
                if let Some(tag) = root_language.filter(|tag| {
                    tag.split('-').next().is_some_and(|base| {
                        ["ja", "ko", "zh"]
                            .iter()
                            .any(|language| base.eq_ignore_ascii_case(language))
                    })
                }) {
                    return tag;
                }
                let mut saw_hangul = false;
                for script in doc
                    .descendants()
                    .filter(|node| {
                        node.is_text() && node.ancestors().any(|parent| parent.has_tag_name("text"))
                    })
                    .filter_map(|node| node.text())
                    .flat_map(str::chars)
                    .map(|c| c.script())
                {
                    if matches!(script, Script::Hiragana | Script::Katakana) {
                        return "ja";
                    }
                    saw_hangul |= script == Script::Hangul;
                }
                if saw_hangul {
                    "ko"
                } else {
                    "ja"
                }
            })),
            _ => root_language,
        };
        let face = platform::find(character, language, resolve)?;
        let Source::Binary(source) = &face.source;
        if let Some(existing) = db.faces().find(|existing| {
            let Source::Binary(existing_source) = &existing.source;
            existing.index == face.index && Arc::ptr_eq(source, existing_source)
        }) {
            return (!excluded.contains(&existing.id)).then_some(existing.id);
        }
        Some(Arc::make_mut(db).push_face_info(face))
    });
}
