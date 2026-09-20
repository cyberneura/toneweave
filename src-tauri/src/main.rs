mod ai;
mod cli;
mod config;
mod draft;
mod results;
use serde::Serialize;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Serialize)]
struct PublicConfig {
    #[serde(flatten)]
    config: config::Config,
    configured: bool,
    config_path: String,
}
#[tauri::command]
async fn get_config() -> Result<PublicConfig, String> {
    let config = config::load().await?;
    let configured =
        !config.ai.api_key.trim().is_empty() || config.ai.provider == "openai-compatible";
    Ok(PublicConfig {
        config,
        configured,
        config_path: config::directory()?
            .join("config.yaml")
            .to_string_lossy()
            .into_owned(),
    })
}
#[derive(Serialize)]
struct Generation {
    variants: Vec<ai::Reply>,
    saved_to: Option<String>,
    copied: bool,
    warnings: Vec<String>,
}
#[tauri::command]
async fn generate_reply(
    app: tauri::AppHandle,
    source: String,
    direction: String,
    preset: String,
    greeting: bool,
    n: Option<usize>,
    decorations: Option<Vec<String>>,
) -> Result<Generation, String> {
    let mut config = config::load().await?;
    if let Some(n) = n {
        config.general.default_n = n;
        config.validate()?;
    }
    let replies = ai::generate(
        &config,
        &source,
        &direction,
        &preset,
        greeting,
        &decorations.unwrap_or_default(),
    )
    .await?;
    let mut warnings = Vec::new();
    let saved_to = match results::save(&replies) {
        Ok(path) => Some(path),
        Err(error) => {
            warnings.push(error);
            None
        }
    };
    let copied = if config.general.auto_copy_result {
        match app.clipboard().write_text(replies.variants[0].text()) {
            Ok(()) => true,
            Err(_) => {
                warnings.push("Replies generated, but automatic clipboard copy failed.".into());
                false
            }
        }
    } else {
        false
    };
    Ok(Generation {
        variants: replies.variants,
        saved_to,
        copied,
        warnings,
    })
}
#[tauri::command]
fn copy_to_clipboard(app: tauri::AppHandle, text: String) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|_| "Could not copy text to the clipboard.".into())
}
#[tauri::command]
fn save_draft(content: String) -> Result<(), String> {
    draft::save(&content)
}
#[tauri::command]
fn load_draft() -> Option<String> {
    draft::load()
}
#[tauri::command]
fn clear_draft() -> Result<(), String> {
    draft::clear()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{}", cli::HELP);
        return;
    }

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            generate_reply,
            get_config,
            copy_to_clipboard,
            save_draft,
            load_draft,
            clear_draft
        ]);

    let app = builder.build(tauri::generate_context!()).unwrap_or_else(|e| {
        eprintln!("Failed to build Toneweave app: {e}");
        std::process::exit(1);
    });

    if !args.is_empty() {
        let cli_args = match cli::parse(&args) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(2);
            }
        };
        if let Err(e) = tauri::async_runtime::block_on(cli::run(&app, cli_args)) {
            eprintln!("{e}");
            std::process::exit(1);
        }
    } else {
        if let Err(e) = config::create_example() {
            eprintln!("{e}");
        }
        app.run(|_app, _event| {});
    }
}