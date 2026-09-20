use std::{io::Write, path::Path};

const MAX_BYTES: usize = 2 * 1024 * 1024;

pub fn save_at(path: &Path, content: &str) -> Result<(), String> {
    if content.len() > MAX_BYTES {
        return Err("Draft is too large to save.".into());
    }
    if let Some(parent) = path.parent() {
        let mut builder = std::fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder
            .create(parent)
            .map_err(|_| "Could not create draft directory.".to_string())?;
    }
    // クラッシュ時に書きかけのファイルが残らないよう、一時ファイルに書いてから rename する
    let tmp = tmp_path(path);
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&tmp)
        .map_err(|_| "Could not write draft.".to_string())?;
    file.write_all(content.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|_| "Could not write draft.".to_string())?;
    std::fs::rename(&tmp, path).map_err(|_| "Could not save draft.".to_string())
}

fn tmp_path(path: &Path) -> std::path::PathBuf {
    path.with_extension("json.tmp")
}

pub fn load_at(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .filter(|s| s.len() <= MAX_BYTES)
}

pub fn clear_at(path: &Path) -> Result<(), String> {
    // 書き込み中のクラッシュで残った一時ファイルにも元メールが入っているので一緒に消す
    for path in [path.to_path_buf(), tmp_path(path)] {
        match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                return Err("Could not clear draft.".into())
            }
            _ => {}
        }
    }
    Ok(())
}

fn path() -> Result<std::path::PathBuf, String> {
    Ok(crate::config::directory()?.join("draft.json"))
}
pub fn save(content: &str) -> Result<(), String> {
    save_at(&path()?, content)
}
pub fn load() -> Option<String> {
    load_at(&path().ok()?)
}
pub fn clear() -> Result<(), String> {
    clear_at(&path()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_and_clear() {
        let dir = std::env::temp_dir().join(format!("toneweave-draft-{}", std::process::id()));
        let path = dir.join("draft.json");
        assert_eq!(load_at(&path), None);
        save_at(&path, r#"{"source":"a"}"#).unwrap();
        save_at(&path, r#"{"source":"b"}"#).unwrap();
        assert_eq!(load_at(&path).as_deref(), Some(r#"{"source":"b"}"#));
        assert!(!path.with_extension("json.tmp").exists());
        std::fs::write(tmp_path(&path), "leftover").unwrap();
        clear_at(&path).unwrap();
        clear_at(&path).unwrap();
        assert_eq!(load_at(&path), None);
        assert!(!tmp_path(&path).exists());
        assert!(save_at(&path, &"x".repeat(MAX_BYTES + 1)).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }
}
