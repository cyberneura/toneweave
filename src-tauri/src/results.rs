use crate::ai::Replies;
use std::{
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
pub fn save_at(directory: &Path, replies: &Replies) -> Result<String, String> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(directory)
        .map_err(|_| "Could not create results directory.".to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Invalid system clock.")?
        .as_nanos();
    let path = directory.join(format!("reply-{stamp}-{}.txt", std::process::id()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&path)
        .map_err(|_| "Could not create result file.".to_string())?;
    for (i, r) in replies.variants.iter().enumerate() {
        writeln!(
            file,
            "=== Variant {} · {} ===\n{}\n",
            i + 1,
            r.tone_used,
            r.text()
        )
        .map_err(|_| "Could not write result file.".to_string())?;
    }
    file.sync_all()
        .map_err(|_| "Could not finish saving result file.".to_string())?;
    Ok(path.to_string_lossy().into_owned())
}
pub fn save(replies: &Replies) -> Result<String, String> {
    save_at(&crate::config::directory()?.join("results"), replies)
}
