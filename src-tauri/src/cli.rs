use std::io::{IsTerminal, Read};
use tauri_plugin_clipboard_manager::ClipboardExt;
pub const HELP: &str = "Toneweave — thoughtful email replies\n\nUsage: toneweave [reply OPTIONS]\n\n  reply                 Compose from stdin (or --clipboard)\n  --clipboard           Read source from the system clipboard\n  --direction, -d TEXT  How to reply\n  --preset, -p NAME     Configured tone preset\n  --greeting, -g on|off Override greeting / keigo mode\n  --decoration, -D TITLE  Apply a configured decoration (repeatable)\n  --help, -h            Show help\n\nNo arguments opens the desktop app. Results go to ~/.config/toneweave/results/.\nExample: toneweave reply --clipboard -d \"やんわり断る\" -g on";
#[derive(Default, Debug, PartialEq)]
pub struct Args {
    pub clipboard: bool,
    pub direction: String,
    pub preset: String,
    pub greeting: Option<bool>,
    pub decorations: Vec<String>,
}
pub fn parse(args: &[String]) -> Result<Args, String> {
    if args.first().map(String::as_str) != Some("reply") {
        return Err("Expected 'reply'. Use --help for usage.".into());
    }
    let mut result = Args::default();
    let mut args = args[1..].iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--clipboard" => result.clipboard = true,
            "--direction" | "-d" => {
                result.direction = args.next().ok_or("Missing direction value.")?.clone()
            }
            "--preset" | "-p" => {
                result.preset = args.next().ok_or("Missing preset value.")?.clone()
            }
            "--greeting" | "-g" => {
                result.greeting = Some(match args.next().map(String::as_str) {
                    Some("on" | "true") => true,
                    Some("off" | "false") => false,
                    _ => return Err("Greeting must be on or off.".into()),
                })
            }
            "--decoration" | "-D" => result
                .decorations
                .push(args.next().ok_or("Missing decoration title.")?.clone()),
            _ => return Err(format!("Unknown option: {arg}")),
        }
    }
    Ok(result)
}
pub async fn run(app: &tauri::App, args: Args) -> Result<(), String> {
    let source = if args.clipboard {
        app.clipboard()
            .read_text()
            .map_err(|_| "Could not read clipboard text.")?
    } else {
        if std::io::stdin().is_terminal() {
            return Err("Pipe email content into stdin, or use --clipboard.".into());
        }
        let mut text = String::new();
        std::io::stdin()
            .take(100_001)
            .read_to_string(&mut text)
            .map_err(|_| "Could not read UTF-8 email from stdin.")?;
        text
    };
    let config = crate::config::load().await?;
    let replies = crate::ai::generate(
        &config,
        &source,
        &args.direction,
        &args.preset,
        args.greeting.unwrap_or(config.greeting.enabled),
        &args.decorations,
    )
    .await?;
    // Print drafts even if local persistence or clipboard fails; never lose a successful response.
    for (i, reply) in replies.variants.iter().enumerate() {
        println!(
            "=== Variant {} · {} ===\n{}\n",
            i + 1,
            reply.tone_used,
            reply.text()
        );
    }
    let path = crate::results::save(&replies)?;
    eprintln!("Saved: {path}");
    if config.general.auto_copy_result {
        app.clipboard()
            .write_text(replies.variants[0].text())
            .map_err(|_| "Drafts saved, but automatic clipboard copy failed.")?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flags() {
        let a = parse(
            &[
                "reply",
                "--clipboard",
                "-d",
                "やんわり断る",
                "-p",
                "丁寧に",
                "-g",
                "off",
            ]
            .map(String::from),
        )
        .unwrap();
        assert!(a.clipboard);
        assert_eq!(a.greeting, Some(false));
        assert_eq!(a.direction, "やんわり断る");
        assert!(parse(&["reply", "-g", "maybe"].map(String::from)).is_err());
        assert!(parse(&["reply", "-d"].map(String::from)).is_err());
    }
}
