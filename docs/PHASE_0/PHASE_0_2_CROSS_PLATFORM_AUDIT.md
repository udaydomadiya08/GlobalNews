# GLOBALNEWS — Phase 0.2 Cross-Platform Architecture Audit

**Document ID:** `GN-DOC-P02-001`  
**Phase:** Phase 0.2 (Cross-Platform Architecture Reconciliation)  
**Status:** Approved Technical Baseline  
**Date of Audit:** 2026-11-09  
**Mandatory Targets:** Windows (EXE), macOS (DMG), Linux (AppImage), Android (APK/Play Store), iOS (IPA/App Store).

---

## 1. Executive Summary: The Cross-Platform Mandate

The product mandate requires GLOBALNEWS to deliver identical core intelligence, offline database capability, and interactive 3D Earth exploration across five distinct operating system environments:

```
+---------------------------------------------------------------------------------------------------+
|                                  THE 5 MANDATORY PLATFORM TARGETS                                 |
+--------------------------+-----------------------+-----------------------+------------------------+
| Platform                 | Distribution Format   | Architecture / ABI    | Webview Substrate      |
+--------------------------+-----------------------+-----------------------+------------------------+
| 1. Windows Desktop       | NSIS / MSI Installer  | x86_64, aarch64       | Microsoft WebView2     |
| 2. macOS Desktop         | DMG Universal Bundle  | arm64 (Apple M), x86  | Apple WKWebView        |
| 3. Linux Desktop         | AppImage / .deb       | x86_64, aarch64       | WebKitGTK              |
| 4. Android Mobile        | APK / AAB Play Store  | arm64-v8a, x86_64     | Android System WebView |
| 5. iOS Mobile            | IPA App Store Signed  | arm64                 | Apple WKWebView        |
+--------------------------+-----------------------+-----------------------+------------------------+
```

A naive approach of building a Python-based daemon for desktop and attempting to port or replicate it for mobile creates an **unsustainable engineering bifurcation** requiring two completely separate codebases. This audit establishes the architectural substrate that allows a **single shared intelligence core** to run efficiently across all five targets.

---

## 2. In-Depth Platform Analysis & Execution Constraints

### 2.1 Windows Desktop (EXE / MSI)
* **Architectures:** `x86_64` (Intel/AMD) and emerging `aarch64` (Windows on ARM / Qualcomm Snapdragon X Elite).
* **Webview Engine:** Microsoft Edge WebView2 (Chromium-based). Pre-installed out of the box on Windows 10 (version 20H2+) and all versions of Windows 11.
* **Background Execution:**
  * In-process background Tokio tasks when the window is minimized or closed to the system notification tray.
  * Windows Task Scheduler or Windows Service architecture if running as an unattended background monitor.
* **Data Storage Location:** `%LOCALAPPDATA%\GlobalNews\` (e.g. `C:\Users\<User>\AppData\Local\GlobalNews\`).
* **Security & Packaging:** Code-signed via Authenticode (EV or standard code-signing certificate); packaged via NSIS installer or WiX MSI.

### 2.2 macOS Desktop (DMG / .app)
* **Architectures:** Universal Binary supporting `arm64` (Apple Silicon M1/M2/M3/M4) and legacy `x86_64` (Intel).
* **Webview Engine:** Native `WKWebView` (WebKit). Zero external browser installation required.
* **Background Execution:**
  * Runs as standard app or macOS Menu Bar (Status Item) application.
  * Managed via `launchd` plist if running as an independent background daemon.
* **Data Storage Location:** `~/Library/Application Support/GlobalNews/`.
* **Security & Packaging:** Apple Developer ID code signing, hardened runtime entitlements (`com.apple.security.network.client`), and automated notarization via `notarytool`.

### 2.3 Linux Desktop (AppImage)
* **Architectures:** `x86_64` and `aarch64`.
* **Webview Engine:** WebKitGTK (`libwebkit2gtk-4.1`). Available natively on Debian/Ubuntu, Fedora, Arch.
* **Packaging Formats:** Standalone AppImage (bundling runtime dependencies), alongside optional native `.deb` and `.rpm` packages.
* **Background Execution:** User-level `systemd` unit (`~/.config/systemd/user/globalnews.service`).
* **Data Storage Location:** XDG Base Directory Specification: `$XDG_DATA_HOME/globalnews/` (`~/.local/share/globalnews/`).

---

### 2.4 Android Mobile (APK / AAB)
* **Target OS:** Android 8.0 (API Level 26) through Android 15+ (API Level 35).
* **Architectures:** `arm64-v8a` (Primary target), `armeabi-v7a` (32-bit legacy fallback), `x86_64` (Emulators).
* **Webview Engine:** Android System WebView (Chromium-based, updated independently via Google Play Store).
* **Background Constraints (Critical Android OS Behavior):**
  * **Doze Mode & App Standby:** Android aggressively restricts CPU and network access when the device is unplugged and stationary with screen off.
  * **WorkManager Standard:** Periodic background tasks must be scheduled via `WorkManager`. The minimum allowable periodic execution interval is **15 minutes**.
  * **Foreground Services:** If the user opts into continuous real-time hazard monitoring (e.g. USGS earthquake alerts), Android requires a **Foreground Service with a persistent ongoing notification** (`android.permission.FOREGROUND_SERVICE_DATA_SYNC`).
  * **Network Security:** Android 9+ (API 28+) strictly blocks cleartext HTTP (`http://`) by default. Internal app IPC must never rely on cleartext localhost HTTP.
* **Storage Location:** Scoped internal storage `context.filesDir` (e.g. `/data/user/0/com.globalnews.app/files/`).

---

### 2.5 iOS Mobile (IPA / App Store)
* **Target OS:** iOS 15.0 through iOS 18+.
* **Architecture:** `arm64` only.
* **Webview Engine:** Apple `WKWebView`.
* **App Store Policies & Technical Constraints (The iOS Reality):**
  1. **Strict Prohibition of Subprocesses:** iOS applications are strictly sandboxed. The POSIX functions `fork()`, `execve()`, and `system()` are forbidden and blocked by kernel sandbox policies. A standalone background daemon process **cannot exist on iOS**.
  2. **Guideline 2.5.2 (No Executable Code / JIT):** Apps cannot download, bundle, or execute external interpreted code or unvetted runtimes that alter application behavior.
  3. **Strict Background Execution Ceilings (`BGTaskScheduler`):**
     * iOS does **NOT permit continuous background ingestion**. Promising "continuous 24/7 background feed monitoring on iOS" is a fatal architectural falsehood that guarantees App Store rejection or runtime execution kills.
     * Background execution is strictly opportunistic, managed via `BGAppRefreshTask`. When iOS grants background execution (based on battery state and user usage patterns), the app has **at most 30 seconds** of execution time.
     * If the background task exceeds 30 seconds or consumes excessive CPU, the iOS watchdog terminates the process with `EXC_RESOURCE -> WAKEUPS`.
  4. **Strict Memory Limits (Jetsam Watchdog):** Background tasks that exceed memory budgets ($\approx 30\text{ MB}$ to $50\text{ MB}$) are instantly killed by Apple's jetsam memory manager.
* **Storage Location:** `~/Library/Application Support/` (persisted and excluded from iCloud backup).

---

## 3. The Fatal Flaws of Python on Mobile

In Phase 0 and 0.1, Python 3.13 was evaluated as the runtime for the intelligence daemon. While Python is functional on desktop, attempting to deploy Python to Android and iOS reveals **insurmountable architectural barriers**:

```
+---------------------------------------------------------------------------------------------------+
|                                 PYTHON MOBILE FEASIBILITY AUDIT                                   |
+--------------------------+-----------------------+-----------------------+------------------------+
| Evaluation Criteria      | Android (Chaquopy)    | iOS (PythonKit/Pyto)  | Verdict for GLOBALNEWS |
+--------------------------+-----------------------+-----------------------+------------------------+
| 1. Subprocess Daemon     | Flaky (Battery killer)| FORBIDDEN by OS       | FATAL FAILURE on iOS   |
| 2. App Store Compliance  | Allowed               | Highly restrictive    | High rejection risk    |
| 3. Cold Start Time       | 1.8 – 3.2 seconds     | 2.5 – 4.0 seconds     | Unacceptable on Mobile |
| 4. APK / IPA Binary Size | +50 MB to +85 MB      | +60 MB to +110 MB     | Massive app bloat      |
| 5. RAM Footprint         | >180 MB RSS           | >200 MB RSS           | INSTANT JETSAM KILL    |
| 6. Background Task (30s) | Inefficient           | Exceeds 30s ceiling   | KILLED BY WATCHDOG     |
+--------------------------+-----------------------+-----------------------+------------------------+
```

### The Inevitable Conclusion
If the core intelligence engine is built in Python, it can only run on Windows, macOS, and Linux. Android and iOS would require a completely separate reimplementation in Kotlin and Swift. 

**Maintaining two completely divergent intelligence engines violates our core project principles.** The core must be unified.

