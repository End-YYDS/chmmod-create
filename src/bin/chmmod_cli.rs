use clap::{Parser, Subcommand};
use std::{
    env::{self},
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};
use walkdir::WalkDir;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};
struct TempCurrentDir {
    original_dir: std::path::PathBuf,
}
impl TempCurrentDir {
    fn new<P: AsRef<Path>>(new_path: P) -> io::Result<Self> {
        let original_dir = std::env::current_dir()?;
        env::set_current_dir(&new_path)?;
        Ok(Self { original_dir })
    }
}
impl Drop for TempCurrentDir {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original_dir);
    }
}
#[derive(Parser)]
#[command(name="chmmod-cli",author, version, about = "編譯與運行項目及前端操作", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    /// 編譯項目：編譯前端、cargo build --release、複製lib文件並打包
    Build,
    /// 執行項目
    Run {
        /// 額外參數
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// 前端操作
    Yarn {
        /// 額外參數
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// 執行cargo命令
    Cargo {
        /// 額外參數
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// 安裝前端依賴
    WebInstall,
    /// 前端開發模式，可以傳入參數
    WebDev {
        /// 額外參數
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// 前端編譯
    WebBuild,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let script_dir = env::current_dir().unwrap();
    let program_name = script_dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let frontend_dir = script_dir.join("src").join("frontend");
    env::set_var("LIB_NAME", &program_name);
    match cli.command {
        Commands::Build => build_release(&script_dir, &frontend_dir, &program_name)?,
        Commands::Run { args } => {
            run_project(&script_dir, &args)?;
        }
        Commands::WebBuild => {
            run_command("yarn", &["build"], &frontend_dir)?;
        }
        Commands::WebDev { args } => {
            run_command(
                "yarn",
                &["dev"]
                    .into_iter()
                    .chain(args.iter().map(String::as_str))
                    .collect::<Vec<_>>(),
                &frontend_dir,
            )?;
        }
        Commands::WebInstall => {
            run_command("yarn", &["install"], &frontend_dir)?;
        }
        Commands::Yarn { args } => {
            run_command(
                "yarn",
                &args.iter().map(String::as_str).collect::<Vec<_>>(),
                &frontend_dir,
            )?;
        }
        Commands::Cargo { args } => {
            run_command(
                "cargo",
                &args.iter().map(String::as_str).collect::<Vec<_>>(),
                &script_dir,
            )?;
        }
    }
    Ok(())
}
/// 編譯項目：編譯前端、cargo build --release、複製lib文件並打包
/// 參數:
/// script_src: 當前目錄
/// frontend_dir: 前端目錄
/// program_name: 程序名稱
/// 返回值: io::Result<()>
fn build_release(script_src: &Path, frontend_dir: &Path, program_name: &str) -> io::Result<()> {
    // 前端管理工具
    let front_packer = env::var("FRONT_PACKER").unwrap_or_else(|_| "yarn".to_string());
    // 編譯前端
    run_command(&front_packer, &["build"], frontend_dir)?;
    run_command("cargo", &["build", "--release"], script_src)?;
    let dist_dir = script_src.join("dist");
    if dist_dir.exists() {
        fs::remove_dir_all(&dist_dir)?;
    }
    fs::create_dir_all(&dist_dir)?;
    let lib_ext = if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "linux") {
        "so"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        eprintln!("Unsupported OS");
        std::process::exit(1);
    };
    let release_dir = script_src.join("target").join("release");
    #[cfg(target_os = "windows")]
    let lib_file = format!("{}.{}", program_name, lib_ext);
    #[cfg(not(target_os = "windows"))]
    let lib_file = format!("lib{}.{}", program_name, lib_ext);
    for entry in fs::read_dir(&release_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.ends_with(&lib_file) {
            fs::copy(&path, dist_dir.join(&lib_file))?;
        }
    }
    let frontend_dist = frontend_dir.join("dist");
    let packer_dist = dist_dir.join("frontend");
    if frontend_dist.exists() {
        copy_recursive(&frontend_dist, &packer_dist, script_src)?;
    } else {
        eprintln!("Frontend dist directory not found");
        std::process::exit(1);
    }
    let output_dir = script_src.join("output");
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir)?;
    }
    fs::create_dir_all(&output_dir)?;
    let output_file = output_dir.join(format!("{}.zip", program_name));
    create_zip_archive(&dist_dir, &output_file)?;
    Ok(())
}
/// 執行項目: cargo run [extra_args]
/// 參數:
/// script_dir: 當前目錄
/// extra_args: 額外參數
/// 返回值: io::Result<ExitStatus>
fn run_project(script_dir: &Path, extra_args: &[String]) -> io::Result<ExitStatus> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").current_dir(script_dir);
    if !extra_args.is_empty() {
        cmd.args(extra_args);
    }
    let status = cmd.status()?;
    Ok(status)
}
/// 執行特定命令
/// 參數:
/// cmd_name: 命令名稱
/// args: 參數
/// dir: 當前目錄
/// 返回值: io::Result<ExitStatus>
fn run_command(cmd_name: &str, args: &[&str], dir: &Path) -> io::Result<ExitStatus> {
    let status = Command::new(cmd_name)
        .args(args)
        .current_dir(dir)
        .status()?;
    if !status.success() {
        eprintln!("Command {} failed with status: {:?}", cmd_name, status);
        std::process::exit(1);
    }
    Ok(status)
}

/// 複製目錄
/// 參數:
/// src: 來源目錄
/// dest: 目標目錄
/// dir: 當前目錄
/// 返回值: io::Result<()>
fn copy_recursive(src: &Path, dest: &Path, dir: &Path) -> io::Result<()> {
    let _temp_dir = TempCurrentDir::new(dir)?;
    if src.is_dir() {
        if !dest.exists() {
            fs::create_dir_all(dest)?;
        }
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let dest_path = dest.join(path.file_name().unwrap());
            copy_recursive(&path, &dest_path, dir)?;
        }
    } else {
        fs::copy(src, dest)?;
    }
    Ok(())
}

/// 使用 zip crate 壓縮指定的目錄，並將結果寫入 zip_path 指定的檔案中。
/// 參數:
/// src_dir: 來源目錄
/// zip_path: 目標 zip 檔案
/// 返回值: io::Result<()>
fn create_zip_archive(src_dir: &Path, zip_path: &Path) -> io::Result<()> {
    let file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<()> = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);
    let src_dir_components = src_dir.components().count();
    for entry in WalkDir::new(src_dir) {
        let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let path = entry.path();
        let name = path
            .components()
            .skip(src_dir_components)
            .collect::<PathBuf>();
        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let mut f = File::open(path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        } else if path.is_dir() && !name.as_os_str().is_empty() {
            let dir_name = format!("{}/", name.to_string_lossy());
            zip.add_directory(dir_name, options)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }
    }
    zip.finish()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(())
}
