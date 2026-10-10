//! The Bite Caption resource packs are built into the app (sources: `resource-packs/`).
//! They contain only our own files: no Mojang sounds or text.
use std::fs;
use std::io::{Cursor, Write};
use std::path::PathBuf;

use maf_platform_win::shell;

macro_rules! files {
    ($base:literal: $($name:literal),+ $(,)?) => {
        &[$(($name, include_bytes!(concat!("../../../resource-packs/", $base, "/", $name)) as &[u8])),+]
    };
}

const BEDROCK: &[(&str, &[u8])] = files!("bedrock/AMF_Bite_Caption":
    "manifest.json", "pack_icon.png", "sounds.json", "sounds/sound_definitions.json",
    "texts/en_US.lang", "texts/languages.json",
);
const JAVA: &[(&str, &[u8])] = files!("java/AMF_Bite_Caption":
    "pack.mcmeta", "pack.png", "assets/minecraft/sounds.json", "assets/minecraft/lang/en_us.json",
);
const FILE_NAME: &str = "AMF_Bite_Caption";

fn zip(files: &[(&str, &[u8])]) -> Result<Vec<u8>, String> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in files {
        writer
            .start_file(*name, zip::write::SimpleFileOptions::default())
            .map_err(|e| e.to_string())?;
        writer.write_all(bytes).map_err(|e| e.to_string())?;
    }
    Ok(writer.finish().map_err(|e| e.to_string())?.into_inner())
}

/// Writes the `.mcpack` to the temp folder and opens it; Minecraft then imports it.
pub fn install_bedrock() -> Result<String, String> {
    let dir = std::env::temp_dir().join("Minecraft Auto Fishing");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{FILE_NAME}.mcpack"));
    fs::write(&path, zip(BEDROCK)?).map_err(|e| e.to_string())?;
    shell::open_path(&path)?;
    Ok(
        "Minecraft is importing the pack. Then turn it on in Settings > Global Resources."
            .to_owned(),
    )
}

/// Copies the pack into `%APPDATA%\.minecraft\resourcepacks` (replacing an older copy).
pub fn install_java() -> Result<String, String> {
    let root =
        PathBuf::from(std::env::var_os("APPDATA").ok_or("APPDATA is not set")?).join(".minecraft");
    if !root.is_dir() {
        return Err("Minecraft: Java Edition was not found (no .minecraft folder).".to_owned());
    }
    let dir = root.join("resourcepacks");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join(format!("{FILE_NAME}.zip")), zip(JAVA)?).map_err(|e| e.to_string())?;
    Ok("Copied. In the game, turn it on in Options > Resource Packs.".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn names_in(bytes: Vec<u8>) -> Vec<String> {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_owned())
            .collect()
    }

    #[test]
    fn packs_zip_every_file_at_the_archive_root() {
        assert_eq!(names_in(zip(BEDROCK).unwrap()).len(), BEDROCK.len());
        let java = zip(JAVA).unwrap();
        assert!(names_in(java.clone()).contains(&"pack.mcmeta".to_owned()));
        let mut archive = zip::ZipArchive::new(Cursor::new(java)).unwrap();
        let mut lang = String::new();
        archive
            .by_name("assets/minecraft/lang/en_us.json")
            .unwrap()
            .read_to_string(&mut lang)
            .unwrap();
        assert!(lang.contains("[AMF] BITE"));
    }
}
