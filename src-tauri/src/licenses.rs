//! Toneweave 自身のライセンスと、配布物に含まれる依存ライブラリのライセンス一覧。
//! macOS のアプリメニュー (About の直下)、Settings ダイアログ、CLI の `--license` から見せる。
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// リポジトリ直下の LICENSE。.app / インストーラには LICENSE ファイルが入らないので埋め込む。
const LICENSE: &str = include_str!("../../LICENSE");
/// リポジトリ直下の THIRD-PARTY-NOTICES.txt (`pnpm notices` で生成する)。
const THIRD_PARTY_NOTICES: &str = include_str!("../../THIRD-PARTY-NOTICES.txt");

const WINDOW_LABEL: &str = "licenses";
pub const MENU_ID: &str = "third-party-licenses";

/// 表示する全文。先頭に Toneweave 自身の MIT ライセンス、その後に依存の一覧。
pub fn text() -> String {
    format!("Toneweave is released under the MIT License.\n\n{LICENSE}\n\n{THIRD_PARTY_NOTICES}")
}

#[tauri::command]
pub fn third_party_notices() -> String {
    text()
}

/// Third-Party Licenses ウインドウを出す。既に開いていれば前面に出すだけ。
/// 非表示のまま作り、画面が本文を埋めてから `licenses_window_ready` で出す
/// (先に出すと空のウインドウが一瞬見える)。
pub fn show_window(app: &AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(WINDOW_LABEL) {
        return present(&win);
    }
    WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::App("licenses.html".into()))
        .title("Third-Party Licenses")
        .inner_size(680.0, 600.0)
        .min_inner_size(420.0, 320.0)
        .visible(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn present(win: &tauri::WebviewWindow) -> Result<(), String> {
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())
}

/// Settings ダイアログから開く。Windows では同期コマンドの中でウインドウを作ると
/// デッドロックするので async にする。
#[tauri::command]
pub async fn show_licenses(app: AppHandle) -> Result<(), String> {
    show_window(&app)
}

/// Third-Party Licenses 画面の描画が終わった合図。ここで初めてウインドウを出す。
#[tauri::command]
pub fn licenses_window_ready(app: AppHandle) -> Result<(), String> {
    match app.get_webview_window(WINDOW_LABEL) {
        Some(win) => present(&win),
        None => Ok(()),
    }
}

/// macOS の既定メニューの、アプリメニューの About の直後に "Third-Party Licenses" を足す。
/// Windows は元々メニューバーを持たないので、Settings ダイアログのボタンから開く。
#[cfg(target_os = "macos")]
pub fn app_menu(app: &AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{Menu, MenuItem, MenuItemKind};
    let menu = Menu::default(app)?;
    if let Some(MenuItemKind::Submenu(app_submenu)) = menu.items()?.into_iter().next() {
        let item = MenuItem::with_id(app, MENU_ID, "Third-Party Licenses", true, None::<&str>)?;
        // 既定のアプリメニューは About が先頭
        app_submenu.insert(&item, 1)?;
    }
    Ok(menu)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// svelte は devDependencies にあるが、その runtime は bundle に入るので notices に載せる
    /// (scripts/generate-third-party-notices.sh の BUNDLED_RUNTIME と揃える)。
    /// clsx は svelte の runtime が import する推移依存。
    const BUNDLED_DEV_DEPENDENCIES: &[&str] = &["svelte"];
    const BUNDLED_TRANSITIVE: &[&str] = &["clsx"];

    /// 配布物に入る直接依存の crate 名を Cargo.toml から拾う。`[dependencies]` と
    /// `[target.'cfg(..)'.dependencies]` の `name = ...` の 1 行書式だけを見る
    /// (`[dependencies.foo]` 形式は拾えない)。build / dev 依存は配布物に入らないので除く。
    /// 配布しないターゲット (Linux) 専用の target 依存を足すと、about.toml の targets の
    /// 外なので notices に載らず、このテストが落ちる。その時はここで除外する。
    fn direct_rust_dependencies(manifest: &str) -> Vec<String> {
        let mut section = String::new();
        let mut names = Vec::new();
        for line in manifest.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                section = line.to_string();
                continue;
            }
            let shipped = section == "[dependencies]"
                || (section.starts_with("[target.") && section.ends_with(".dependencies]"));
            if !shipped || line.starts_with('#') {
                continue;
            }
            if let Some((name, _)) = line.split_once('=') {
                names.push(name.trim().to_string());
            }
        }
        names
    }

    /// package.json の dependencies (vite が bundle する runtime 依存) の名前。
    fn direct_npm_dependencies() -> Vec<String> {
        let package: serde_json::Value =
            serde_json::from_str(include_str!("../../package.json")).expect("package.json parses");
        package["dependencies"]
            .as_object()
            .expect("package.json has dependencies")
            .keys()
            .cloned()
            .collect()
    }

    /// Windows の checkout は autocrlf で CRLF になるので、行末を揃えてから読む。
    fn lf(text: &str) -> String {
        text.replace("\r\n", "\n")
    }

    /// THIRD-PARTY-NOTICES.txt の "Used by:" ブロックに並ぶ (package 名, version)。
    /// 生成物のエントリは区切り線 → `License: ...` → 空行 → "Used by:" の並びなので、
    /// その並びだけをブロックとして読む (ライセンス本文に同じ字面があっても数えない)。
    /// npm 側と Rust 側は "# Rust crates" の見出しで分かれている。
    fn packages_in_notices(notices: &str, section: &str) -> Vec<(String, String)> {
        let text = match section {
            "npm" => notices.split("# Rust crates").next(),
            "rust" => notices.split("# Rust crates").nth(1),
            _ => None,
        }
        .expect("the notices file has a Rust crates heading");
        let separator = "=".repeat(80);
        let mut packages = Vec::new();
        let mut in_block = false;
        let mut after_separator = false;
        let mut after_license = false;
        for line in text.lines() {
            if in_block {
                let mut words = line.strip_prefix("  ").unwrap_or("").split(' ');
                match (words.next(), words.next()) {
                    (Some(name), Some(version)) if !name.is_empty() => {
                        packages.push((name.to_string(), version.to_string()));
                    }
                    _ => in_block = false,
                }
                continue;
            }
            in_block = after_license && line == "Used by:";
            after_license = (after_separator && line.starts_with("License: "))
                || (after_license && line.is_empty());
            after_separator = line == separator;
        }
        packages
    }

    /// Cargo.lock の [[package]] ブロック。(name, version, dependencies の行)。
    fn locked_rust_packages(lock: &str) -> Vec<(String, String, Vec<String>)> {
        lock.split("[[package]]")
            .skip(1)
            .map(|block| {
                let field = |key: &str| {
                    block
                        .lines()
                        .find_map(|line| line.strip_prefix(key))
                        .map(|rest| rest.trim().trim_matches('"').to_string())
                        .unwrap_or_default()
                };
                let deps = block
                    .lines()
                    .filter_map(|line| line.strip_prefix(" \""))
                    .map(|line| line.trim_end_matches("\",").to_string())
                    .collect();
                (field("name = "), field("version = "), deps)
            })
            .collect()
    }

    /// Cargo.lock が toneweave の直接依存 `name` に選んだ version。同じ crate が 2 つ以上の
    /// version で入っている時は、ルートの dependencies に `name version` の形で書かれる。
    fn resolved_rust_version(lock: &[(String, String, Vec<String>)], name: &str) -> String {
        let root = lock
            .iter()
            .find(|(crate_name, _, _)| crate_name == "toneweave")
            .expect("Cargo.lock has the toneweave package");
        let entry = root
            .2
            .iter()
            .find(|dep| *dep == name || dep.starts_with(&format!("{name} ")))
            .unwrap_or_else(|| panic!("{name} is not a dependency of toneweave in Cargo.lock"));
        // 同名同 version で source が違う時は `name version (source)` になるので 2 語目だけ
        match entry.split(' ').nth(1) {
            Some(version) => version.to_string(),
            None => {
                let mut versions = lock
                    .iter()
                    .filter(|(crate_name, _, _)| crate_name == name)
                    .map(|(_, version, _)| version.clone());
                let version = versions.next().expect("the crate is in Cargo.lock");
                assert!(
                    versions.next().is_none(),
                    "{name} has several versions in Cargo.lock"
                );
                version
            }
        }
    }

    /// pnpm-lock.yaml の importers の `.` (このプロジェクト) が `group`
    /// (dependencies / devDependencies) に選んだ (name, version)。
    /// `name:` → `specifier:` → `version:` の 3 行で並ぶ。
    fn resolved_npm_versions(lock: &str, group: &str) -> Vec<(String, String)> {
        let importer = lock
            .split("\nimporters:\n")
            .nth(1)
            .expect("pnpm-lock.yaml has an importers section")
            .split("\npackages:\n")
            .next()
            .expect("importers come before packages");
        let header = format!("    {group}:");
        let mut resolved = Vec::new();
        let mut name = String::new();
        let mut in_group = false;
        for line in importer.lines() {
            if line.starts_with("    ") && !line.starts_with("     ") {
                in_group = line == header;
                continue;
            }
            if !in_group {
                continue;
            }
            if let Some(key) = line
                .strip_prefix("      ")
                .filter(|rest| !rest.starts_with(' '))
            {
                name = key.trim_end_matches(':').trim_matches('\'').to_string();
            } else if let Some(version) = line.strip_prefix("        version: ") {
                // peer 依存の括弧は notices の version には無い
                let version = version.split('(').next().unwrap_or(version).trim();
                resolved.push((name.clone(), version.to_string()));
            }
        }
        resolved
    }

    /// 直接依存が、Cargo.lock が選んだ version で載っているか。名前だけだと、上げた依存の
    /// 旧 version が推移依存として残っている時に通ってしまう。
    #[test]
    fn third_party_notices_list_every_direct_rust_dependency() {
        // Arrange
        let deps = direct_rust_dependencies(&lf(include_str!("../Cargo.toml")));
        assert!(deps.contains(&"tauri".to_string()), "parsed deps: {deps:?}");
        let lock = locked_rust_packages(&lf(include_str!("../Cargo.lock")));
        let listed = packages_in_notices(&lf(THIRD_PARTY_NOTICES), "rust");
        assert!(listed.len() > 100, "parsed notices: {listed:?}");

        // Act
        let missing: Vec<(String, String)> = deps
            .iter()
            .map(|name| (name.clone(), resolved_rust_version(&lock, name)))
            .filter(|entry| !listed.contains(entry))
            .collect();

        // Assert
        assert!(
            missing.is_empty(),
            "not in THIRD-PARTY-NOTICES.txt (run `pnpm notices`): {missing:?}"
        );
    }

    /// 載っている crate の version が Cargo.lock と食い違えば、依存を上げたのに
    /// `pnpm notices` を流していない。
    #[test]
    fn third_party_notices_match_cargo_lock_versions() {
        // Arrange
        let lock = locked_rust_packages(&lf(include_str!("../Cargo.lock")));
        let listed = packages_in_notices(&lf(THIRD_PARTY_NOTICES), "rust");
        assert!(listed.len() > 100, "parsed notices: {listed:?}");

        // Act
        let stale: Vec<&(String, String)> = listed
            .iter()
            .filter(|(name, version)| !lock.iter().any(|(n, v, _)| n == name && v == version))
            .collect();

        // Assert
        assert!(
            stale.is_empty(),
            "not in Cargo.lock (run `pnpm notices`): {stale:?}"
        );
    }

    /// bundle される npm パッケージが、pnpm-lock.yaml が選んだ version で過不足なく載っているか。
    /// notices の npm 側は node_modules の package.json から書くので、lock を更新して
    /// install と再生成を忘れると古い version のまま残る。
    #[test]
    fn third_party_notices_list_every_bundled_npm_package() {
        // Arrange
        let deps = direct_npm_dependencies();
        assert!(
            deps.contains(&"@tauri-apps/api".to_string()),
            "parsed deps: {deps:?}"
        );
        let lock = lf(include_str!("../../pnpm-lock.yaml"));
        let resolved = resolved_npm_versions(&lock, "dependencies");
        assert_eq!(
            resolved.len(),
            deps.len(),
            "parsed lock importer: {resolved:?}"
        );
        let mut expected = resolved;
        expected.extend(
            resolved_npm_versions(&lock, "devDependencies")
                .into_iter()
                .filter(|(name, _)| BUNDLED_DEV_DEPENDENCIES.contains(&name.as_str())),
        );
        assert_eq!(
            expected.len(),
            deps.len() + BUNDLED_DEV_DEPENDENCIES.len(),
            "bundled devDependencies not found in the lock: {expected:?}"
        );
        let listed = packages_in_notices(&lf(THIRD_PARTY_NOTICES), "npm");

        // Act
        let missing: Vec<&(String, String)> = expected
            .iter()
            .filter(|entry| !listed.contains(entry))
            .collect();
        let unknown: Vec<&(String, String)> = listed
            .iter()
            .filter(|entry| !expected.contains(entry))
            .filter(|(name, version)| {
                // 推移依存は importers に出ないので、packages に同じ version があるかで見る
                !(BUNDLED_TRANSITIVE.contains(&name.as_str())
                    && lock.contains(&format!("\n  {name}@{version}:\n")))
            })
            .collect();
        let transitive_listed = BUNDLED_TRANSITIVE
            .iter()
            .all(|name| listed.iter().any(|(listed_name, _)| listed_name == name));

        // Assert
        assert!(
            missing.is_empty(),
            "not in THIRD-PARTY-NOTICES.txt (run `pnpm notices`): {missing:?}"
        );
        assert!(
            unknown.is_empty(),
            "not in pnpm-lock.yaml (run `pnpm notices`): {unknown:?}"
        );
        assert!(transitive_listed, "parsed notices: {listed:?}");
    }

    /// Windows の checkout (CRLF) でも同じ結果になること。
    #[test]
    fn notices_parsers_accept_crlf() {
        // Arrange
        let notices = "x\r\n# Rust crates\r\n\r\n".to_string()
            + &"=".repeat(80)
            + "\r\nLicense: MIT\r\n\r\nUsed by:\r\n  serde 1.0.0 (u)\r\n\r\ntext\r\n";
        let lock = "lockfileVersion: '9.0'\r\nimporters:\r\n  .:\r\n    dependencies:\r\n      clsx:\r\n        specifier: ^2\r\n        version: 2.1.1\r\npackages:\r\n";

        // Act
        let listed = packages_in_notices(&lf(&notices), "rust");
        let resolved = resolved_npm_versions(&lf(lock), "dependencies");

        // Assert
        assert_eq!(listed, vec![("serde".to_string(), "1.0.0".to_string())]);
        assert_eq!(resolved, vec![("clsx".to_string(), "2.1.1".to_string())]);
    }

    /// `--license` と Third-Party Licenses ウインドウが出す全文。自身の MIT が先頭にあり、
    /// 依存の一覧が続くこと。
    #[test]
    fn license_text_starts_with_own_license_then_notices() {
        // Act
        let text = lf(&text());

        // Assert
        assert!(text.starts_with("Toneweave is released under the MIT License."));
        assert!(text.contains("Copyright (c) 2026 Cyberneura"));
        let own = text
            .find("MIT License\n\nCopyright (c) 2026 Cyberneura")
            .unwrap();
        let notices = text.find("THIRD-PARTY NOTICES").unwrap();
        assert!(own < notices);
        assert!(text.contains("# Rust crates (compiled into the binary)"));
    }
}
