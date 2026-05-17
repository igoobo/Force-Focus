# Force-Focus Desktop Agent

A Tauri v2 desktop application built with **React + TypeScript** (frontend) and **Rust** (backend). This agent monitors user activity, manages focus sessions, and communicates with the Force-Focus backend server.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Project Structure](#project-structure)
- [Environment Setup](#environment-setup)
- [Development](#development)
- [Testing](#testing)
- [Production Build](#production-build)
- [Deep Link (URL Scheme) Setup](#deep-link-url-scheme-setup)
- [Demo Videos](#demo-videos)

---

## Prerequisites

Make sure the following tools are installed on your system before getting started:

| Tool        | Version     | Install Guide                                                    |
|-------------|-------------|------------------------------------------------------------------|
| **Node.js** | v18+        | [nodejs.org](https://nodejs.org/)                                |
| **npm**     | v9+         | Bundled with Node.js                                             |
| **Rust**    | latest      | [rustup.rs](https://rustup.rs/)                                  |
| **Tauri CLI** | v2        | Installed via npm (`@tauri-apps/cli` in devDependencies)         |

> **Windows users**: You also need the [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 10/11).

---

## Project Structure

```
desktop-agent/
├── src/                  # React + TypeScript frontend source
├── src-tauri/            # Rust backend (Tauri core)
│   ├── src/              # Rust source modules
│   ├── tests/            # Rust integration tests
│   ├── resources/models/ # Bundled ML model files
│   ├── Cargo.toml        # Rust dependencies
│   ├── tauri.conf.json   # Tauri app configuration
│   └── .env              # (Optional) Rust-side environment variables (fallback)
├── docs/                 # Project documentation
│   └── demo/             # Demo video files (place recordings here)
├── .env                  # (Optional) Frontend (Vite) environment variables (fallback)
├── package.json          # Node.js dependencies and scripts
└── vite.config.ts        # Vite build configuration
```

---

## Environment Setup

The backend API URL (`API_BASE_URL`) is resolved using a **priority-based system**. The app checks each source in order and uses the first value it finds:

| Priority | Source | When It Applies | Scope |
|----------|--------|-----------------|-------|
| **1st** | Shell environment variable | Set in terminal before `build` / `dev` | Baked into binary at compile time |
| **2nd** | `.env` file | Present in project directory | Loaded at runtime (fallback) |
| **3rd** | Hardcoded default | Always | `http://127.0.0.1:8000/api/v1` |

### Option A — Set via Shell (Recommended)

Set the environment variable directly in your terminal **before** running any build or dev command. This is the primary and recommended approach.

**PowerShell:**

```powershell
# Rust backend (compile-time, baked into binary via option_env!())
$env:API_BASE_URL="http://127.0.0.1:8000/api/v1"

# Frontend (Vite requires VITE_ prefix)
$env:VITE_API_BASE_URL="http://127.0.0.1:8000/api/v1"

# Then run dev or build
npm run tauri dev
```

**CMD:**

```cmd
set API_BASE_URL=http://127.0.0.1:8000/api/v1
set VITE_API_BASE_URL=http://127.0.0.1:8000/api/v1
npm run tauri dev
```

> [!IMPORTANT]
> Shell environment variables are **session-scoped** — they reset when you close the terminal. Set them again each time you open a new terminal window.

### Option B — Use `.env` Files (Fallback)

If no shell variable is set, the app falls back to `.env` files. This is convenient for local development so you don't have to set variables every time.

**Frontend** (`desktop-agent/.env`):

```env
# VITE_ prefix is required for Vite to expose it to the browser
VITE_API_BASE_URL=http://127.0.0.1:8000/api/v1
```

**Rust backend** (`desktop-agent/src-tauri/.env`):

```env
API_BASE_URL=http://127.0.0.1:8000/api/v1
```

> [!NOTE]
> Both `.env` files are listed in `.gitignore` and will **not** be committed to version control. Every developer must create them locally if using this method.

### Option C — Use the Default

If neither a shell variable nor a `.env` file is present, the app defaults to `http://127.0.0.1:8000/api/v1`. This works out of the box when the backend is running locally on the default port.

---

## Development

### 1. Install Node.js Dependencies

```bash
cd desktop-agent
npm install
```

### 2. Run in Development Mode (Tauri + Vite)

This starts both the Vite dev server (frontend) and the Tauri Rust backend with hot-reload:

```bash
npm run tauri dev
```

- The Vite dev server starts on `http://localhost:1420`
- The Tauri window launches automatically
- Frontend changes trigger HMR (Hot Module Replacement)
- Rust changes trigger a recompile

### 3. Run the Rust Backend Only (Cargo)

If you want to work on the Rust side independently:

```bash
cd src-tauri
cargo run
```

> [!NOTE]
> Running `cargo run` alone will start the Tauri application, but the frontend must already be built (via `npm run build`) or the Vite dev server must be running separately (`npm run dev`).

---

## Testing

### Run Rust Tests

```bash
cd src-tauri
cargo test
```

This runs all unit tests and integration tests (e.g., `tests/integration_test.rs`).

To run a specific test:

```bash
cargo test <test_name>
```

To see detailed output:

```bash
cargo test -- --nocapture
```

---

## Production Build

For production, set the API URL via shell environment variables **before** building. This bakes the URL into the binary at compile time via Rust's `option_env!()` macro, so no `.env` file is needed at runtime.

```powershell
# PowerShell — set the production API URL
$env:API_BASE_URL="https://api.example.com/api/v1"
$env:VITE_API_BASE_URL="https://api.example.com/api/v1"

# Build the installer
npm run tauri build
```

This will:

1. Compile the React frontend (`npm run build` → `dist/`)
2. Compile the Rust backend in **release mode** with the API URL baked in
3. Generate platform-specific installers in `src-tauri/target/release/bundle/`

| Platform | Output Location                                         |
|----------|---------------------------------------------------------|
| Windows  | `src-tauri/target/release/bundle/msi/` and `nsis/`      |

> [!WARNING]
> If you forget to set `API_BASE_URL` before building, the production binary will fall back to `http://127.0.0.1:8000/api/v1`, which is almost certainly not what you want for a release build.

---

## Deep Link (URL Scheme) Setup

Force-Focus uses the `force-focus://` custom URL scheme for OAuth callback redirection. When a user logs in via the web dashboard, the browser redirects to:

```
force-focus://auth/callback?access_token=...&refresh_token=...&email=...&user_id=...
```

This opens (or focuses) the desktop agent and completes the login automatically.

### Development Mode — Register the URL Scheme

During development (`cargo run` / `npm run tauri dev`), you need to manually register the custom URL scheme in the Windows Registry so that `force-focus://` links open your **debug build**.

1. Open `src-tauri/register_scheme.reg`
2. **Edit the `.exe` path** to match your local build path:
   ```reg
   @="\"C:\\Users\\<YOUR_USERNAME>\\Downloads\\Force-Focus\\desktop-agent\\src-tauri\\target\\debug\\desktop-agent.exe\" \"%1\""
   ```
3. Double-click the `.reg` file to import it into the registry.

> [!WARNING]
> You must update the path in `register_scheme.reg` to your own system path before importing. Use double backslashes (`\\`) in the registry file.

### Testing the Deep Link

After registering the scheme, you can test it from:

- **Browser address bar**: Type `force-focus://auth/callback?access_token=test` and press Enter.
- **PowerShell**:
  ```powershell
  Start-Process "force-focus://auth/callback?access_token=test&refresh_token=test&email=test@test.com&user_id=123"
  ```
- **Web Dashboard**: Use the login flow — the backend will redirect to `force-focus://auth/callback?...` with real tokens.

### Redirect from Web Dashboard (`set [URL]`)

When the backend needs to redirect the user's browser to the desktop agent, the web dashboard issues a redirect to the `force-focus://` URL. The typical flow:

1. User clicks **Login** on the web dashboard
2. Backend authenticates and constructs a callback URL:
   ```
   force-focus://auth/callback?access_token=<TOKEN>&refresh_token=<TOKEN>&email=<EMAIL>&user_id=<ID>
   ```
3. The browser navigates to this URL, which triggers the OS to open the registered application (`desktop-agent.exe`)
4. The desktop agent receives the URL, parses the tokens, saves them, and shows the main window

### Removing the URL Scheme

To unregister the custom scheme, run in PowerShell (Admin):

```powershell
Remove-Item -Path "HKCU:\Software\Classes\force-focus" -Recurse -Force
```

---

## Demo Videos

The following demo recordings are available in [`docs/demo/`](docs/demo/):

| Video | Description |
|-------|-------------|
| [login.mp4](docs/demo/login.mp4) | OAuth login flow via web dashboard → deep link callback |
| [session(base).mp4](docs/demo/sessoion(base).mp4) | Basic focus session — start, monitor, and end |
| [session(snapshot).mp4](docs/demo/session(snapshot).mp4) | Session with activity snapshot capture |
| [session(Cache hit).mp4](docs/demo/session(Cache%20hit).mp4) | Session demonstrating local cache hit behavior |

> [!TIP]
> To add new recordings, place `.mp4` files in `docs/demo/` and update this table. For large video files (>50 MB), consider using [Git LFS](https://git-lfs.com/) to avoid bloating the repository.

---

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
