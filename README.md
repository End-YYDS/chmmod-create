# 建立CHM模組腳手架

這是一個用於快速建立 CHM 插件模組的命令列工具。它可以自動生成必要的專案結構和配置文件。

## 功能特點
從Github上下載插件專案檔，並初始化一些功能

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
詳見: `https://github.com/End-YYDS/React_Project_init/tree/Plugin-FrameWork`
## 專案執行方式
### 使用專門提供的工具`chmmod-cli`來執行或其他操作
```bash
chmmod-cli -h
```
### 進入各個目錄下去執行各種指令：`yarn`、`cargo`

## 發布資訊

[![Crates.io](https://img.shields.io/crates/v/chmmod-create.svg)](https://crates.io/crates/chmmod-create)
[![Documentation](https://docs.rs/chmmod-create/badge.svg)](https://docs.rs/chmmod-create)

本專案已發布於 crates.io，可以通過以下指令安裝：

```bash
cargo install chmmod-create
```