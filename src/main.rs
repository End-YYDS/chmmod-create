//! # CHM Plugin Scaffold
//!
//! 這是一個用於快速建立 CHM 插件模組的命令列工具。
//!
//! ## 功能
//!
//! - 自動生成預設插件代碼
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
use clap::{crate_authors, crate_version, Arg, Command};
use serde_json::Value;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;
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

    if let Err(e) = scaffold_module(&module_name, &version, &description).await {
        eprintln!(
            "Error: Failed to scaffold the module '{}'. Reason: {}",
            module_name, e
        );
        std::process::exit(1);
    }
    println!("Module '{}' has been successfully scaffolded!", module_name);
}

/// 建立新的模組腳手架
///
/// # Arguments
///
/// * `module_name` - 模組名稱
/// * `version` - 模組版本
/// * `description` - 模組描述
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - 成功返回 Ok(()), 失敗返回錯誤
async fn scaffold_module(
    module_name: &str,
    version: &str,
    description: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let module_path = Path::new(module_name);
    if module_path.exists() {
        println!(
            "Module '{}' already exists. Please choose a different name.",
            module_name
        );
        return Ok(());
    }

    create_frontend_pages(module_path, version, description).await?;
    create_gitignore(module_path)?;
    Ok(())
}

/// 創建 .gitignore 文件
/// # Arguments
/// * `module` - 模組路徑
///
/// # Returns
/// * `std::io::Result<()>` - 成功返回 Ok(()), 失敗返回錯誤
///
fn create_gitignore(module: &Path) -> std::io::Result<()> {
    let path = module.join(".gitignore");
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
*.sw?
out
.__mf__temp"#;
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
    module: &Path,
    module_version: &str,
    module_description: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let owner = std::env::var("OWNER").unwrap_or("End-YYDS".to_string());
    let repo = std::env::var("REPO").unwrap_or("React_Project_init".to_string());
    let branch = std::env::var("BRANCH").unwrap_or("Plugin-FrameWork".to_string());
    let module_name = module.file_stem().unwrap().to_str().unwrap();
    println!("Downloading CHM Plugin Framework from GitHub...");
    download_and_extract(&owner, &repo, &branch, module).await?;
    println!("Updated package.json for '{}'", module_name);
    let package_json = module.join("package.json");
    let f = File::open(&package_json)?;
    let mut data: Value = serde_json::from_reader(f)?;
    if let Value::Object(ref mut map) = data {
        map["name"] = Value::String(module_name.to_string());
        map["description"] = Value::String(module_description.to_string());
        map["version"] = Value::String(module_version.to_string());
        if let Some(Value::Object(ref mut scripts)) = map.get_mut("scripts") {
            scripts.insert(
                "postbuild".to_string(),
                Value::String("node scripts/postbuild.js".to_string()),
            );
        }
    }
    let f = File::create(&package_json)?;
    serde_json::to_writer_pretty(f, &data)?;
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
    let temp_dir_t = tempdir()?;
    let temp_dir = temp_dir_t.path();
    let url = format!(
        "https://github.com/{}/{}/archive/refs/heads/{}.zip",
        owner, repo, branch
    );
    println!("Downloading from: {}", url);
    let response = reqwest::get(&url).await?;
    let bytes = response.bytes().await?;
    let extra_file = format!("{}-{}", repo, branch);
    let zip_path = temp_dir.join(format!("{}.zip", extra_file.as_str()));
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

        let out_path = temp_dir.join(file_path);
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
    let from_file = temp_dir.join(extra_file);
    fs::remove_file(zip_path)?;
    fs::rename(from_file, extract_path)?;
    temp_dir_t.close()?;
    println!("Successfully extracted files!");
    Ok(())
}
