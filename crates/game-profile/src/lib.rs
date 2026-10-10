//! Read-only access to the local Minecraft: Java Edition installation. The bite caption
//! text comes from the player's own game files at runtime, so the app ships no game text
//! (ADR-008), and every installed version contributes its wording (ADR-009).
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Caption keys of the fishing bobber's sounds.
pub const BITE_KEY: &str = "subtitles.entity.fishing_bobber.splash";
pub const THROW_KEY: &str = "subtitles.entity.fishing_bobber.throw";
pub const RETRIEVE_KEY: &str = "subtitles.entity.fishing_bobber.retrieve";

pub struct JavaProfile {
    root: PathBuf,
}

impl JavaProfile {
    /// `%APPDATA%\.minecraft`, when it exists.
    pub fn default_location() -> Option<Self> {
        let root = PathBuf::from(std::env::var_os("APPDATA")?).join(".minecraft");
        root.is_dir().then_some(Self { root })
    }

    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The game language from `options.txt`, e.g. "zh_tw". Only the `lang:` line is read.
    pub fn language(&self) -> Option<String> {
        let options = fs::read_to_string(self.root.join("options.txt")).ok()?;
        options
            .lines()
            .find_map(|l| l.strip_prefix("lang:"))
            .map(|l| l.trim().to_owned())
    }

    /// The bite caption in `lang` across every installed version (usually one string).
    pub fn bite_captions(&self, lang: &str) -> Vec<String> {
        self.captions(BITE_KEY, lang)
    }

    /// The caption for `key` in `lang` across every installed version.
    /// Falls back to English when the language has no entry.
    pub fn captions(&self, key: &str, lang: &str) -> Vec<String> {
        let mut found = self.captions_from_assets(key, lang);
        if lang == "en_us" || found.is_empty() {
            found.extend(self.captions_from_jars(key));
        }
        found.into_iter().collect()
    }

    /// Non-English language files live in the shared asset store, referenced by hash.
    fn captions_from_assets(&self, key: &str, lang: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let assets = self.root.join("assets");
        let Ok(indexes) = fs::read_dir(assets.join("indexes")) else {
            return out;
        };
        let object = format!("minecraft/lang/{lang}.json");
        for index in indexes.flatten() {
            let Some(hash) = read_json(&index.path())
                .and_then(|i| i["objects"][&object]["hash"].as_str().map(str::to_owned))
            else {
                continue;
            };
            if hash.len() < 2 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                continue; // never build a path from anything but a plain hash
            }
            let file = assets.join("objects").join(&hash[..2]).join(&hash);
            if let Some(text) = read_json(&file).and_then(|j| j[key].as_str().map(str::to_owned)) {
                out.insert(text);
            }
        }
        out
    }

    /// English ships inside each version's client jar.
    fn captions_from_jars(&self, key: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let Ok(versions) = fs::read_dir(self.root.join("versions")) else {
            return out;
        };
        for version in versions.flatten() {
            let jar = version
                .path()
                .join(format!("{}.jar", version.file_name().to_string_lossy()));
            if let Some(text) = english_caption_in_jar(&jar, key) {
                out.insert(text);
            }
        }
        out
    }
}

fn read_json(path: &Path) -> Option<serde_json::Value> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

fn english_caption_in_jar(jar: &Path, key: &str) -> Option<String> {
    let mut archive = zip::ZipArchive::new(fs::File::open(jar).ok()?).ok()?;
    let mut entry = archive.by_name("assets/minecraft/lang/en_us.json").ok()?;
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    json[key].as_str().map(str::to_owned)
}

/// Windows OCR language tag for a Java language code: "en_us" -> "en-US",
/// "zh_tw" -> "zh-Hant-TW", "zh_cn" -> "zh-Hans-CN".
pub fn ocr_language_tag(game_lang: &str) -> String {
    match game_lang {
        "zh_tw" | "zh_hk" => format!("zh-Hant-{}", game_lang[3..].to_ascii_uppercase()),
        "zh_cn" => "zh-Hans-CN".to_owned(),
        other => match other.split_once('_') {
            Some((lang, region)) => format!("{lang}-{}", region.to_ascii_uppercase()),
            None => other.to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// A tiny fake `.minecraft` with one asset index, one object and one client jar.
    fn fake_install() -> PathBuf {
        let root = std::env::temp_dir().join(format!("maf-profile-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let hash = "ab12cd";
        fs::create_dir_all(root.join("assets/indexes")).unwrap();
        fs::create_dir_all(root.join("assets/objects/ab")).unwrap();
        fs::create_dir_all(root.join("versions/1.0")).unwrap();
        fs::write(
            root.join("options.txt"),
            "version:1\nlang:zh_tw\nguiScale:4\n",
        )
        .unwrap();
        fs::write(
            root.join("assets/indexes/1.json"),
            format!(
                r#"{{"objects":{{"minecraft/lang/zh_tw.json":{{"hash":"{hash}","size":1}}}}}}"#
            ),
        )
        .unwrap();
        fs::write(
            root.join("assets/objects/ab").join(hash),
            format!(r#"{{"{BITE_KEY}":"TW caption"}}"#),
        )
        .unwrap();
        let mut jar =
            zip::ZipWriter::new(fs::File::create(root.join("versions/1.0/1.0.jar")).unwrap());
        jar.start_file(
            "assets/minecraft/lang/en_us.json",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        jar.write_all(format!(r#"{{"{BITE_KEY}":"EN caption"}}"#).as_bytes())
            .unwrap();
        jar.finish().unwrap();
        root
    }

    #[test]
    fn reads_language_and_captions() {
        let root = fake_install();
        let profile = JavaProfile::at(&root);
        assert_eq!(profile.language().as_deref(), Some("zh_tw"));
        assert_eq!(profile.bite_captions("zh_tw"), ["TW caption"]);
        assert_eq!(profile.bite_captions("en_us"), ["EN caption"]);
        assert_eq!(profile.bite_captions("fr_fr"), ["EN caption"]); // fallback
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn maps_game_languages_to_ocr_tags() {
        assert_eq!(ocr_language_tag("en_us"), "en-US");
        assert_eq!(ocr_language_tag("zh_tw"), "zh-Hant-TW");
        assert_eq!(ocr_language_tag("zh_cn"), "zh-Hans-CN");
        assert_eq!(ocr_language_tag("ja_jp"), "ja-JP");
    }
}
