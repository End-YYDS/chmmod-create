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

    if let Err(e) = scaffold_module(&module_name) {
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
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - 成功返回 Ok(()), 失敗返回錯誤
fn scaffold_module(module_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    create_new_lib(module_name);
    // create_mod_toml(module_name).map_err(|e| format!("Failed to create mod.toml. {}", e))?;
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
fn create_new_lib(module_name: &str) {
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
}

#[allow(dead_code)]
fn create_mod_toml(module_name: &str) {
    let mod_content = format!(
        r#"[module]
name = "{}"
version = "0.1.0"
description = "This is the {} module"
"#,
        module_name, module_name
    );

    let mod_toml_path = format!("{}/Mod.toml", module_name);
    fs::write(&mod_toml_path, mod_content).expect("Failed to create Mod.toml");
    println!("Created Mod.toml for module '{}'", module_name);
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
        dependencies
            .as_table_mut()
            .unwrap()
            .insert("plugin_lib", plugin_lib);
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
