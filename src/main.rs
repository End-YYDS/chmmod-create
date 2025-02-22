//! # CHM Plugin Scaffold
//!
//! 這是一個用於快速建立 CHM 插件模組的命令列工具。
//!
//! ## 功能
//!
//! - 自動建立新的 Rust 函式庫專案
//! - 配置 Cargo.toml 以支援動態函式庫編譯
//! - 自動整合 plugin_lib 依賴
//! - 自動生成預設插件代碼
//! - 生成 GitHub Actions 工作流程
//!
//! ## 使用方式
//!
//! ```bash
//! chmmod-create --name my_module
//! ```
//! ## 幫助
//! ```bash
//! chmmod-create --help
//! ```

mod workflow;
use clap::{crate_authors, crate_version, Arg, Command};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
// #[cfg(target_os = "linux")]
// use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command as ProcessCommand;
use toml_edit::{value, Array, DocumentMut, Item};
use workflow::create_build_workflow;
// const EXECUTE_FILE: &str = "chm_cli";
const PLUGIN_FRONTEND_DIR: &str = "src";
/// 主程式入口點
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let matches = Command::new("CHM Plugin Scaffold")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Generates a new CHM plugin module")
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .action(clap::ArgAction::Set)
                .help("Name of the module"),
        )
        .arg(
            Arg::new("description")
                .short('d')
                .long("description")
                .action(clap::ArgAction::Set)
                .help("Description of the plugin"),
        )
        .arg(
            Arg::new("pversion")
                .short('v')
                .long("plugin-version")
                .action(clap::ArgAction::Set)
                .help("Plugin version"),
        )
        .arg(
            Arg::new("scope")
                .short('s')
                .long("scope")
                .action(clap::ArgAction::Set)
                .help("Plugin scope"),
        )
        .arg(
            Arg::new("frontend")
                .short('f')
                .long("frontend")
                .action(clap::ArgAction::SetTrue)
                .help("Whether to include frontend pages"),
        )
        .get_matches();
    let module_name = matches
        .get_one::<String>("name")
        .cloned()
        .unwrap_or_else(|| {
            println!("Please enter a module name:");
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => input.trim().to_string(),
                Err(_) => {
                    eprintln!("Failed to read name exiting...");
                    std::process::exit(1);
                }
            }
        });
    let description = matches
        .get_one::<String>("description")
        .cloned()
        .unwrap_or_else(|| {
            println!("Please enter a plugin description:");
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => input.trim().to_string(),
                Err(_) => {
                    eprintln!("Failed to read description, exiting...");
                    std::process::exit(1);
                }
            }
        });
    let version = matches
        .get_one::<String>("pversion")
        .cloned()
        .unwrap_or_else(|| {
            println!("Please enter the plugin version (default: 0.1.0):");
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let version = input.trim();
                    if version.is_empty() {
                        "0.1.0".to_string()
                    } else {
                        version.to_string()
                    }
                }
                Err(_) => {
                    eprintln!("Failed to read version, using default: 0.1.0");
                    "0.1.0".to_string()
                }
            }
        });
    let scope = matches
        .get_one::<String>("scope")
        .cloned()
        .unwrap_or_else(|| {
            println!("Please enter the plugin scope (default: public):");
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let scope = input.trim();
                    if scope.is_empty() {
                        "public".to_string()
                    } else {
                        scope.to_string()
                    }
                }
                Err(_) => {
                    eprintln!("Failed to read scope, using default: public");
                    "public".to_string()
                }
            }
        });
    let include_frontend = matches.get_flag("frontend");

    if let Err(e) = scaffold_module(
        &module_name,
        &version,
        &description,
        &scope,
        include_frontend,
    )
    .await
    {
        eprintln!(
            "Error: Failed to scaffold the module '{}'. Reason: {}",
            module_name, e
        );
        std::process::exit(1);
    }
    println!("Module '{}' has been successfully scaffolded!", module_name);
    if include_frontend {
        println!("Frontend pages have been included in the scaffold.");
    }
}

/// 建立新的模組腳手架
///
/// # Arguments
///
/// * `module_name` - 模組名稱
/// * `version` - 模組版本
/// * `description` - 模組描述
/// * `scope` - 模組作用域
/// * `need_frontend` - 是否需要前端頁面
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - 成功返回 Ok(()), 失敗返回錯誤
async fn scaffold_module(
    module_name: &str,
    version: &str,
    description: &str,
    scope: &str,
    need_frontend: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    create_new_lib(module_name, version, description, scope)?;
    create_build_workflow(module_name)
        .map_err(|e| format!("Failed to create build workflow. {}", e))?;
    update_cargo_toml(module_name)?;
    // create_executable_script(module_name)?;
    create_gitignore(module_name)?;
    if need_frontend {
        create_frontend_pages(module_name, version).await?;
    }
    Ok(())
}

/// 使用 cargo new 建立新的函式庫
///
/// # Arguments
///
/// * `module_name` - 模組名稱
/// * `version` - 模組版本
/// * `description` - 模組描述
/// * `scope` - 模組作用域
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - 成功返回 Ok(()), 失敗返回錯誤
///
fn create_new_lib(
    module_name: &str,
    version: &str,
    description: &str,
    scope: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let plugin_name = format!("{}_Plugin", &module_name);
    let status = ProcessCommand::new("cargo")
        .arg("new")
        .arg("--lib")
        .arg("-q")
        .arg(module_name)
        .status()
        .expect("Failed to execute cargo new");

    if !status.success() {
        eprintln!("Failed to create new library with cargo");
        std::process::exit(1);
    }
    println!("Created new library '{}'", module_name);
    let lib_content = format!(
        r#"use actix_web::Responder;
use plugin_lib::{{declare_plugin, register_plugin}};

#[derive(Debug)]
#[allow(non_camel_case_types)]
struct {plugin_name};

impl {plugin_name} {{
    pub fn new() -> Self {{
        Self
    }}

    async fn test() -> impl Responder {{
        "{description}"
    }}
}}

declare_plugin!(
    {plugin_name},
    meta: {{"{plugin_name}","{version}", "{description}","/{scope}",""}},
    "{module_name}.js",
    functions:{{
        "/test" => {{
            method: actix_web::web::get(),
            handler: {plugin_name}::test
        }}
    }}
);

register_plugin!({plugin_name});"#
    );
    let lib_path = format!("{}/src/lib.rs", module_name);
    fs::write(&lib_path, lib_content)?;
    println!("Updated lib.rs content for '{}'", module_name);
    Ok(())
}

/// 更新 Cargo.toml 配置
///
/// # Arguments
///
/// * `module_name` - 模組名稱
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - 成功返回 Ok(()), 失敗返回錯誤
///
fn update_cargo_toml(module_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cargo_toml_path = format!("{}/Cargo.toml", module_name);

    let mut cargo_content = String::new();
    {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&cargo_toml_path)
            .expect("Failed to open Cargo.toml");
        file.read_to_string(&mut cargo_content)
            .expect("Failed to read Cargo.toml");
    }

    let mut doc = cargo_content.parse::<DocumentMut>()?;

    if let Some(dependencies) = doc.get_mut("dependencies") {
        let plugin_lib = Item::Table({
            let mut table = toml_edit::Table::new();
            let repo_url = std::env::var("GIT_REPO")
                .unwrap_or_else(|_| "https://github.com/End-YYDS/plugin_lib".to_string());
            table["git"] = value(repo_url);
            // table["features"] = value(vec!["plugin_macro"]);
            table
        });
        let actix_web_lib = value("4.9.0");
        let dependencies_table = dependencies.as_table_mut().unwrap();
        dependencies_table.insert("plugin_lib", plugin_lib);
        dependencies_table.insert("actix-web", actix_web_lib);
    }

    let lib_section = doc
        .entry("lib")
        .or_insert(Item::Table(toml_edit::Table::new()));
    if let Some(lib_table) = lib_section.as_table_mut() {
        let mut crate_types = Array::new();
        crate_types.push("dylib");
        lib_table["crate-type"] = value(crate_types);
    }
    fs::write(&cargo_toml_path, doc.to_string())?;
    println!("Updated Cargo.toml for module '{}'", module_name);
    Ok(())
}
/// 創建可執行腳本
/// # Arguments
/// * `module_name` - 模組名稱
///
/// # Returns
/// * `std::io::Result<()>` - 成功返回 Ok(()), 失敗返回錯誤
///
// fn create_executable_script(module_name: &str) -> std::io::Result<()> {
//     let execute_file = format!(
//         "{}/{}",
//         module_name,
//         std::env::var("EXECUTE_FILE").unwrap_or(EXECUTE_FILE.to_string())
//     );
//     let path = Path::new(&execute_file);
//     #[cfg(target_os = "linux")]
//     {
//         let content = include_str!("../chm_cli.sh");
//         let mut file = File::create(path)?;
//         file.write_all(content.as_bytes())?;
//         let mut perms = file.metadata()?.permissions();
//         perms.set_mode(0o755);
//         file.set_permissions(perms)?;
//     }
//     #[cfg(target_os = "windows")]
//     {
//         let content = include_str!("../chm_cli.bat");
//         let mut file = File::create(path)?;
//         file.write_all(content.as_bytes())?;
//     }
//     dbg!(path);
//     Ok(())
// }
/// 創建 .gitignore 文件
/// # Arguments
/// * `module_name` - 模組名稱
///
/// # Returns
/// * `std::io::Result<()>` - 成功返回 Ok(()), 失敗返回錯誤
///
fn create_gitignore(module_name: &str) -> std::io::Result<()> {
    let gitignore_file = format!("{}/.gitignore", module_name);
    let path = Path::new(&gitignore_file);
    let content = r#"target/
logs
*.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*
pnpm-debug.log*
lerna-debug.log*
node_modules
dist
dist-ssr
*.local
!.vscode/*
.idea
.DS_Store
*.suo
*.ntvs*
*.njsproj
*.sln
*.sw?"#;
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// 創建 .env 文件
/// # Arguments
/// * `module_name` - 模組名稱
/// # Returns
/// * `std::io::Result<()>` - 成功返回 Ok(()), 失敗返回錯誤
///
fn create_env(module_name: &str) -> std::io::Result<()> {
    let gitignore_file = format!("{}/src/frontend/.env", module_name);
    let path = Path::new(&gitignore_file);

    let content = format!("VITE_LIB_NAME={}", module_name);
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
/// 創建前端頁面
/// # Arguments
/// * `module_name` - 模組名稱
/// * `module_version` - 模組版本
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - 成功返回 Ok(()), 失敗返回錯誤
///
async fn create_frontend_pages(
    module_name: &str,
    module_version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let frontend_dir = format!(
        "{}/{}",
        module_name,
        std::env::var("PLUGIN_FRONTEND_DIR").unwrap_or(PLUGIN_FRONTEND_DIR.to_string())
    );
    let path = Path::new(&frontend_dir);
    let owner = std::env::var("OWNER").unwrap_or("End-YYDS".to_string());
    let repo = std::env::var("REPO").unwrap_or("React_Project_init".to_string());
    let branch = std::env::var("BRANCH").unwrap_or("front-framework".to_string());
    println!("Downloading frontend pages from GitHub...");
    download_and_extract(&owner, &repo, &branch, path).await?;
    println!("Created frontend pages for '{}'", module_name);
    println!("Create env file for '{}'", module_name);
    create_env(module_name)?;
    println!("Updated package.json for '{}'", module_name);
    let package_json = format!("{}/frontend/package.json", frontend_dir);
    let content = fs::read_to_string(&package_json)?;
    let mut json_data: serde_json::Value = serde_json::from_str(&content)?;
    if let Some(name) = json_data.get_mut("name") {
        *name = serde_json::Value::String(String::from(module_name));
    } else {
        println!("Field 'name' not found in package.json");
    }
    if let Some(version) = json_data.get_mut("version") {
        *version = serde_json::Value::String(String::from(module_version));
    } else {
        println!("Field 'version' not found in package.json");
    }
    let updated_content = serde_json::to_string_pretty(&json_data)?;
    fs::write(package_json, updated_content)?;
    Ok(())
}
/// 下載並解壓縮文件
/// # Arguments
/// * `owner` - GitHub 帳號
/// * `repo` - GitHub 倉庫名稱
/// * `branch` - 分支名稱
/// * `download_path` - 下載路徑
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - 成功返回 Ok(()), 失敗返回錯誤
///
async fn download_and_extract(
    owner: &str,
    repo: &str,
    branch: &str,
    download_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(download_path)?;
    let url = format!(
        "https://github.com/{}/{}/archive/refs/heads/{}.zip",
        owner, repo, branch
    );
    println!("Downloading from: {}", url);
    let response = reqwest::get(&url).await?;
    let bytes = response.bytes().await?;
    let zip_path = download_path.join(format!("{}_{}.zip", repo, branch));
    fs::write(&zip_path, bytes)?;
    println!("Downloaded zip to: {}", zip_path.display());
    let zip_file = fs::File::open(&zip_path)?;
    let mut archive = zip::ZipArchive::new(zip_file)?;
    let extract_path = download_path.to_path_buf();
    println!("Extracting to: {}", extract_path.display());
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let out_path = extract_path.join(file_path);

        if file.name().ends_with('/') {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(p) = out_path.parent() {
                fs::create_dir_all(p)?;
            }
            let mut outfile = fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    let extra_file =
        std::env::var("EXTRA_FILE").unwrap_or("React_Project_init-front-framework".to_string());
    let from_file = format!("{}/{}", extract_path.display(), extra_file);
    let to_file = format!("{}/frontend", extract_path.display());
    fs::remove_file(zip_path)?;
    fs::rename(from_file, to_file)?;
    println!("Successfully extracted files!");
    Ok(())
}
