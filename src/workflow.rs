//! # 工作流程模組
//! 
//! 這個模組負責生成 GitHub Actions 工作流程配置文件。

use serde::{Deserialize, Serialize};
use serde_yml::to_string;
use std::{collections::HashMap, fs::create_dir_all, path::Path};

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowConfig {
    name: String,
    #[serde(rename = "on")]
    on: OnTrigger,
    env: HashMap<String, String>,
    jobs: Jobs,
}

#[derive(Debug, Serialize, Deserialize)]
struct OnTrigger {
    push: PushTrigger,
}

#[derive(Debug, Serialize, Deserialize)]
struct PushTrigger {
    branches: Vec<String>,
    paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Jobs {
    build: BuildJob,
}

#[derive(Debug, Serialize, Deserialize)]
struct BuildJob {
    #[serde(rename = "if")]
    condition: Option<String>,
    #[serde(rename = "runs-on", skip_serializing_if = "Option::is_none")]
    runs_on: Option<String>,
    strategy: Strategy,
    steps: Vec<Step>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Strategy {
    matrix: Matrix,
}

#[derive(Debug, Serialize, Deserialize)]
struct Matrix {
    include: Vec<MatrixTarget>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MatrixTarget {
    name: String,
    runner: String,
    target: String,
    #[serde(rename = "lib-suffix")]
    lib_suffix: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Step {
    name: String,
    #[serde(rename = "uses", skip_serializing_if = "Option::is_none")]
    uses: Option<String>,
    #[serde(rename = "with", skip_serializing_if = "Option::is_none")]
    with: Option<HashMap<String, String>>,
    #[serde(rename = "run", skip_serializing_if = "Option::is_none")]
    run: Option<String>,
    #[serde(rename = "shell", skip_serializing_if = "Option::is_none")]
    shell: Option<String>,
}

/// 建立 GitHub Actions 工作流程配置
/// 
/// # Arguments
/// 
/// * `name` - 專案名稱
/// 
/// # Returns
/// 
/// * `Result<(), Box<dyn std::error::Error>>` - 成功返回 Ok(()), 失敗返回錯誤
pub fn create_build_workflow(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let project_name = name.to_string();
    let pwf = Path::new(name);
    if !pwf.exists() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("目錄 '{}' 不存在，無法生成工作流程檔案。", name),
        )));
    }
    let mut env = HashMap::new();
    env.insert("PROJECT_NAME".to_string(), project_name.clone());

    let on_trigger = OnTrigger {
        push: PushTrigger {
            branches: vec!["main".to_string()],
            paths: vec![
                "src/**".to_string(),
                "Cargo.toml".to_string(),
                ".github/workflows/build.yml".to_string(),
            ],
        },
    };

    let matrix_targets = vec![
        MatrixTarget {
            name: "linux-amd64".to_string(),
            runner: "ubuntu-latest".to_string(),
            target: "x86_64-unknown-linux-gnu".to_string(),
            lib_suffix: ".so".to_string(),
        },
        MatrixTarget {
            name: "win-amd64".to_string(),
            runner: "windows-latest".to_string(),
            target: "x86_64-pc-windows-msvc".to_string(),
            lib_suffix: ".dll".to_string(),
        },
        MatrixTarget {
            name: "macos-amd64".to_string(),
            runner: "macos-latest".to_string(),
            target: "x86_64-apple-darwin".to_string(),
            lib_suffix: ".dylib".to_string(),
        },
        MatrixTarget {
            name: "macos-arm64".to_string(),
            runner: "macos-latest".to_string(),
            target: "aarch64-apple-darwin".to_string(),
            lib_suffix: ".dylib".to_string(),
        },
    ];

    let strategy = Strategy {
        matrix: Matrix {
            include: matrix_targets,
        },
    };
    let steps = vec![
        Step {
            name: "Checkout".to_string(),
            uses: Some("actions/checkout@v4".to_string()),
            with: None,
            run: None,
            shell: None,
        },
        Step {
            name: "Install Rust".to_string(),
            uses: Some("dtolnay/rust-toolchain@stable".to_string()),
            with: {
                let mut with = HashMap::new();
                with.insert(
                    "targets".to_string(),
                    "${{ matrix.target }}".to_string(),
                );
                Some(with)
            },
            run: None,
            shell: None,
        },
        Step {
            name: "Setup Cache".to_string(),
            uses: Some("Swatinem/rust-cache@v2".to_string()),
            with: None,
            run: None,
            shell: None,
        },
        Step {
            name: "Create Dist Directory".to_string(),
            uses: None,
            with: None,
            run: Some("mkdir -p dist".to_string()),
            shell: None,
        },
        Step {
            name: "Build Library".to_string(),
            uses: None,
            with: None,
            run: Some("cargo build --verbose --locked --release --target ${{ matrix.target }}".to_string()),
            shell: None,
        },
        Step {
            name: "Prepare Release Binary".to_string(),
            uses: None,
            with: None,
            shell: Some("bash".to_string()),
            run: Some(
                r#"if [[ "${{ matrix.target }}" == "x86_64-pc-windows-msvc" ]]; then
    LIB_PREFIX=""
    LIB_OUTPUT="target/${{ matrix.target }}/release/${PROJECT_NAME}${{ matrix.lib-suffix }}"
else
    LIB_PREFIX="lib"
    LIB_OUTPUT="target/${{ matrix.target }}/release/${LIB_PREFIX}${PROJECT_NAME}${{ matrix.lib-suffix }}"
fi
LIB_RELEASE="${PROJECT_NAME}${{ matrix.lib-suffix }}"
if [ ! -f "${LIB_OUTPUT}" ]; then
    echo "Library not found at ${LIB_OUTPUT}"
    exit 1
fi
cp "${LIB_OUTPUT}" "./dist/${LIB_RELEASE}"
ls -l dist/
"#.to_string()),
        },
        Step {
            name: "Upload Artifact".to_string(),
            uses: Some("actions/upload-artifact@v4".to_string()),
            with: {
                let mut with = HashMap::new();
                with.insert("name".to_string(), format!(r#"{}-${{{{ matrix.name }}}}-library"#,name));
                with.insert("path".to_string(), "dist/".to_string());
                with.insert("retention-days".to_string(), "30".to_string());
                Some(with)
            },
            run: None,
            shell: None,
        },
    ];

    let build_job = BuildJob {
        condition: Some("!contains(github.event.head_commit.message, '[skip ci]')".to_string()),
        runs_on: Some("${{ matrix.runner }}".to_string()),
        strategy,
        steps,
    };

    let jobs = Jobs { build: build_job };

    let workflow = WorkflowConfig {
        name: "Build".to_string(),
        on: on_trigger,
        env,
        jobs,
    };
    let yaml_string = to_string(&workflow)?;

    let yaml_string = yaml_string
        .replace("'on'", "on")
        .replace("'${{", "${{")
        .replace("}}'", "}}")
        .replace(
            "'!contains(github.event.head_commit.message, '[skip ci]')'",
            "!contains(github.event.head_commit.message, '[skip ci]')",
        )
        .replace("'30'", "30");

    let pwf = pwf.join(".github/workflows");
    create_dir_all(&pwf)?;
    std::fs::write(pwf.join("build.yml"), yaml_string)?;
    println!("Build workflow generated successfully!");
    Ok(())
}
