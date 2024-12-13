# 建立CHM模組腳手架

這是一個用於快速建立 CHM 插件模組的命令列工具。它可以自動生成必要的專案結構和配置文件。

## 功能特點

- 自動建立新的 Rust 函式庫專案
- 配置 Cargo.toml 以支援動態函式庫編譯
- 自動整合 plugin_core 依賴
- 生成 GitHub Actions 工作流程，支援多平台編譯：
  - Windows (x64)
  - Linux (x64)
  - macOS (x64/ARM64)

## 安裝
### 1. 下載原代碼並安裝
```bash
cargo install --path .
```
### 2. 使用cargo crate 安裝
```bash
cargo install chmmod-create
```
## 使用方式
有兩種使用方式
### 1. 使用命令行參數
```bash
chmmod-create --name example
```
### 2. 互動式輸入
```bash
chmmod-create 執行之後會提示輸入模組名稱
```

## 生成的專案結構
```
example/
|---- .github/
    |---- workflows/
        |---- build.yml # Github Actions 工作流程配置
|---- src/
    |---- lib.rs #模組主程式碼
|---- Cargo.toml #專案配置文件
```
## 自動化建置
專案包含 GitHub Actions 工作流程，當推送到 main 分支時會自動觸發建置：
- 支援多平台交叉編譯
- 自動產生各平台的動態函式庫檔案
- 建置產物會上傳為 GitHub Actions 成品
## 注意事項
- 確保模組名稱符合 Rust 命名規範（小寫字母、數字和底線）
- 需要有 Rust 工具鏈和 Cargo 套件管理器
- 如果要跳過 CI 建置，在 commit 訊息中加入 `[skip ci]`

## 發布資訊

[![Crates.io](https://img.shields.io/crates/v/chmmod-create.svg)](https://crates.io/crates/chmmod-create)
[![Documentation](https://docs.rs/chmmod-create/badge.svg)](https://docs.rs/chmmod-create)

本專案已發布於 crates.io，可以通過以下指令安裝：

```bash
cargo install chmmod-create
```