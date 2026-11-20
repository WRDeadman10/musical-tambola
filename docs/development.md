# Development Guide — Media Tambola

## Prerequisites
- **Node.js**: v22.15.0+ / npm 10.9.2+
- **Rust Toolchain**: `rustup` with `stable-x86_64-pc-windows-msvc`
- **C++ Build Tools**: Visual Studio 2022 Community with C++ Build Tools (`Microsoft.VisualStudio.Component.VC.Tools.x86.x64`)
- **WebView2**: Windows Evergreen WebView2 Runtime

## Setup & Toolchain Installation
1. If Rust is not installed, install Rustup:
   ```powershell
   winget install --id Rustlang.Rustup -e
   # Restart your terminal to refresh PATH with %USERPROFILE%\.cargo\bin
   rustup default stable-x86_64-pc-windows-msvc
   ```
   > [!IMPORTANT]
   > The developer shell requires `%USERPROFILE%\.cargo\bin` on PATH for running `cargo` and `npm test`. If opening a new session, ensure PATH includes `%USERPROFILE%\.cargo\bin` (restart the terminal after rustup installation). Do not hardcode user paths in npm scripts.
2. Verify dependencies:
   ```powershell
   rustc --version
   cargo --version
   node --version
   npm --version
   ```
3. Install frontend dependencies:
   ```powershell
   npm install
   ```

## Development Commands
- **Run in Development**:
  ```powershell
  npm run dev
  # Starts Tauri dev environment (Vite frontend + Rust compilation)
  ```
- **Run Tests**:
  ```powershell
  npm run test
  # Runs Vitest for frontend + cargo test for Rust backend
  ```
- **Lint & Format**:
  ```powershell
  npm run lint
  ```
- **Build Windows Installer (NSIS x64)**:
  ```powershell
  npm run package:win
  ```

## Coding Conventions & Quality Gates
- **TypeScript**:
  - Explicit typing on all function signatures, variable declarations, and state variables (no implicit `any`).
  - Strict null checks enabled.
  - K&R braces on all blocks.
- **Rust**:
  - `cargo clippy -- -D warnings` must pass cleanly without warnings.
  - Domain tests must achieve full coverage of generator, shuffle, and claim matrix rules.
- **Git Commit Workflow**:
  - Atomic, logical commits with imperative titles (e.g. `feat: add migration system`).
  - Commit messages specify context, files changed, and test results.
