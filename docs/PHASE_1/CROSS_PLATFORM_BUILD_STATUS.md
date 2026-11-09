# GLOBALNEWS — Phase 1.1 Cross-Platform Build Status

**Document ID:** `GN-DOC-P11-004`  
**Phase:** Phase 1.1 (Portable Rust Core Foundation & Cross-Platform Scaffolding)  
**Status:** Approved Technical Status Report  
**Date:** 2026-11-09  

---

## 1. Honest Cross-Platform Compilation Status

In strict accordance with our engineering principles, **we never claim a target is supported merely because its Rust target triple exists**. 

Compiling to diverse operating systems requires native platform toolchains (Android NDK, Apple Xcode SDK, Microsoft Visual C++ Build Tools). The table below reflects the actual status of each platform target:

```
+-------------------------------------------------------------------------------------------------------------------------+
|                                      CROSS-PLATFORM TARGET STATUS MATRIX                                                |
+----------------------+-----------------------------+-----------------------+---------------------+----------------------+
| Target Platform      | Rust Target Triple          | Build Toolchain       | Compilation Status  | CI Runner Reqt       |
+----------------------+-----------------------------+-----------------------+---------------------+----------------------+
| **macOS (Apple M)**  | `aarch64-apple-darwin`      | Clang / Apple SDK     | **ACTIVE (Host)**   | `macos-14` (arm64)   |
| **macOS (Intel)**    | `x86_64-apple-darwin`       | Clang / Apple SDK     | Configured          | `macos-13` (x86_64)  |
| **Windows x86_64**   | `x86_64-pc-windows-msvc`    | MSVC Linker (C++)     | Configured (CI)     | `windows-latest`     |
| **Linux x86_64**     | `x86_64-unknown-linux-gnu`  | GCC / Glibc 2.31+     | Configured (CI)     | `ubuntu-22.04`       |
| **Android arm64**    | `aarch64-linux-android`     | Android NDK r26+      | Configured (NDK req)| `ubuntu-latest` (NDK)|
| **iOS arm64**        | `aarch64-apple-ios`         | Xcode 15+ / iOS SDK   | Configured (SDK req)| `macos-14` (Xcode)   |
+----------------------+-----------------------------+-----------------------+---------------------+----------------------+
```

---

## 2. Platform Build Prerequisites & Toolchain Requirements

### 2.1 macOS arm64 (`aarch64-apple-darwin`) — Primary Development Host
* **Local Status:** Compiles natively on the host Apple Silicon workstation.
* **Prerequisites:** Apple Command Line Tools (`clang`, `libSystem`, `ar`).
* **Command:** `cargo build --workspace`

### 2.2 Windows x86_64 (`x86_64-pc-windows-msvc`)
* **Host Limitation:** Cannot be built natively on macOS without wine or complex cross-compilers.
* **Prerequisites:** Requires Microsoft C++ Build Tools (MSVC `link.exe`, Windows 10/11 SDK).
* **CI Build Pipeline:** Executed on GitHub Actions `windows-latest`:
  ```bash
  rustup target add x86_64-pc-windows-msvc
  cargo build --workspace --target x86_64-pc-windows-msvc
  ```

### 2.3 Linux x86_64 (`x86_64-unknown-linux-gnu`)
* **Host Limitation:** Cross-compiling from macOS to Linux requires `cross` (Docker-based cross-compilation tool) or a Linux runner.
* **Prerequisites:** `glibc-devel`, `pkg-config`, `libwebkit2gtk-4.1-dev`.
* **CI Build Pipeline:** Executed on GitHub Actions `ubuntu-22.04`:
  ```bash
  sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential
  cargo build --workspace --target x86_64-unknown-linux-gnu
  ```

### 2.4 Android arm64 (`aarch64-linux-android`)
* **Prerequisites:** Android NDK (version r26b or later) and `cargo-ndk`.
* **Build Command:**
  ```bash
  rustup target add aarch64-linux-android
  cargo ndk --target aarch64-linux-android --platform 26 -- build --release -p globalnews-core
  ```
* **Output:** Generates `libglobalnews_core.so`, which is placed directly in the Android app's `jniLibs/arm64-v8a/` directory.

### 2.5 iOS arm64 (`aarch64-apple-ios`)
* **Prerequisites:** Apple macOS with Xcode 15+ installed.
* **Build Pipeline:**
  ```bash
  rustup target add aarch64-apple-ios aarch64-apple-ios-sim
  cargo build --package globalnews-core --target aarch64-apple-ios --release
  ```
* **Packaging:** Packed into an Apple universal **XCFramework** using `xcodebuild -create-xcframework`, linked directly by Tauri’s iOS Xcode project.

---

## 3. Continuous Integration (CI) Multi-Runner Matrix

A complete cross-platform pipeline requires three parallel GitHub Actions runner environments:

```
+-----------------------------------------------------------------------------------+
| GITHUB ACTIONS CI BUILD MATRIX                                                    |
+-------------------+-----------------------+---------------------------------------+
| Runner OS         | Target Triples        | Artifacts Generated                   |
+-------------------+-----------------------+---------------------------------------+
| `macos-14`        | `aarch64-apple-darwin`| macOS DMG (Universal Apple Silicon)   |
| (M1/M2 VM)        | `aarch64-apple-ios`   | iOS XCFramework & signed IPA          |
+-------------------+-----------------------+---------------------------------------+
| `windows-latest`  | `x86_64-pc-windows-msvc` Windows NSIS / WiX MSI Installer     |
+-------------------+-----------------------+---------------------------------------+
| `ubuntu-22.04`    | `x86_64-unknown-linux`| Linux AppImage & .deb bundle          |
|                   | `aarch64-linux-android` Android APK / AAB (NDK compilation)   |
+-------------------+-----------------------+---------------------------------------+
```
