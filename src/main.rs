//! # CHM Plugin Scaffold
//!
//! 這是一個用於快速建立 CHM 插件模組的命令列工具。
//!
//! ## 功能
//!
//! - 自動建立新的 Rust 函式庫專案
//! - 配置 Cargo.toml 以支援動態函式庫編譯
//! - 自動整合 plugin_core 依賴
//! - 生成 GitHub Actions 工作流程
//!
//! ## 使用方式
//!
//! ```bash
//! chmmod-create --name my_module
//! ```

mod workflow;
use clap::{Arg, Command};
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::process::Command as ProcessCommand;
use toml_edit::{value, Array, DocumentMut, Item};
use workflow::create_build_workflow;

/// 主程式入口點
fn main() {
    dotenv::dotenv().ok();
    let matches = Command::new("CHM Plugin Scaffold")
        .version("0.1.0")
        .author("CHM")
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

    if let Err(e) = scaffold_module(&module_name, &version, &description, &scope) {
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
/// * `scope` - 模組作用域
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - 成功返回 Ok(()), 失敗返回錯誤
fn scaffold_module(
    module_name: &str,
    version: &str,
    description: &str,
    scope: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    create_new_lib(module_name, version, description, scope)?;
    create_build_workflow(module_name)
        .map_err(|e| format!("Failed to create build workflow. {}", e))?;
    update_cargo_toml(module_name)?;
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
    "",
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
            let repo_url = std::env::var("GIT_REPO").expect("GIT_REPO is not set");
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
