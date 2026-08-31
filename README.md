# Tatpar तत्पर

> **Always ready to run.**  
> **हमेशा चलाने के लिए तैयार।**

A persistent, floating desktop code scratchpad built with **Tauri v2**, **React 19**, and **Rust**. Write and execute code instantly in **Kotlin, Python, Java, JavaScript, TypeScript, and C++** — without opening a heavy IDE.

---

## ⚡ Features

- **🪟 Persistent Floating Window** — Stays on top and available whenever inspiration strikes.
- **⚡ Instant Local Execution** — Built in Rust with minimal overhead; run code snippets with `Ctrl+Enter`.
- **🛑 Live Process Cancellation** — Dedicated **Stop** button to immediately terminate long-running processes or infinite loops.
- **🟢 Runtime Availability Indicator** — Automatic real-time detection checking system `PATH` for required compilers and interpreters with visual status indicators.
- **🔧 6 Built-in Language Executors** — Kotlin, Python, Java, JavaScript, TypeScript, and C++.
- **🎨 Dark & Light Themes** — High-contrast, polished UI tailored for day and night coding sessions.
- **🔒 Local & Privacy-First** — 100% offline local execution in temporary sandboxed workspaces; zero telemetry or external network requests.
- **⚙️ Configurable Execution** — Customizable execution timeouts (default 10s), theme toggling, and compiler paths.

---

## 🚀 Quick Start

### Installation

1. Download the latest release (`Tatpar-*.msi` / installer) from [GitHub Releases](https://github.com/mohitsharma16/tatpar/releases).
2. Run the installer to set up **Tatpar**.
3. Launch Tatpar (or use global hotkey `Ctrl+Shift+Space`).

### First Run

1. Select your target programming language from the top toolbar.
2. Observe the status dot next to the picker:
   - 🟢 **Green**: Runtime compiler/interpreter found on `PATH`.
   - 🔴 **Red**: Compiler/interpreter missing on `PATH` (shows warning bar).
3. Write your code in the Monaco editor.
4. Press **▶ Run** or hit `Ctrl+Enter` to execute.

---

## 💻 Supported Languages & System Runtimes

Tatpar executes code locally using installed compilers and interpreters on your system `PATH`:

| Language | File Ext | Required CLI Runtime | Default Example Snippet | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Kotlin** | `.kt` | `kotlinc` (+ `java`) | `println("Hello, Tatpar!")` | ✅ Full Support |
| **Python** | `.py` | `python` / `python3` | `print("Hello, Tatpar!")` | ✅ Full Support |
| **Java** | `.java` | `javac` (+ `java`) | `System.out.println("Hello, Tatpar!");` | ✅ Full Support |
| **JavaScript** | `.js` | `node` | `console.log("Hello, Tatpar!");` | ✅ Full Support |
| **TypeScript** | `.ts` | `tsc` / `npx` | `console.log("Hello, Tatpar!");` | ✅ Full Support |
| **C++** | `.cpp` | `g++` / `clang++` | `std::cout << "Hello, Tatpar!";` | ✅ Full Support |

---

## ⌨️ Key Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Ctrl+Enter` | Run active code in editor |
| `Ctrl+Shift+Space` | Global hotkey / Toggle window visibility |

---

## 🔒 Security & Privacy

- **Local Execution Only**: All code runs strictly on your machine under user-level permissions.
- **Timeout Protection**: Default 10-second process timeout prevents runaway scripts or CPU locks.
- **Isolated Workspace**: Code executes in temporary files isolated to system temp folders and automatically cleaned up.
- **Zero Telemetry**: No tracking, analytics, user accounts, or external network requests.

---

## 🛠️ Development Setup

### Prerequisites

- **Node.js** (v18+)
- **pnpm** (v8+)
- **Rust & Cargo** (v1.70+)
- **Git**

### Clone & Setup

```bash
# Clone repository
git clone https://github.com/mohitsharma16/tatpar.git
cd tatpar

# Install dependencies
pnpm install

# Start development server with Tauri
pnpm tauri dev
```

### Build Production Package

```bash
pnpm tauri build
```

Built release artifacts will be placed in `src-tauri/target/release/bundle/`.

---

## 🤝 Adding a New Language Executor

Tatpar's backend architecture uses modular language executors in Rust (`src-tauri/src/execution/`):

```rust
// 1. Define your executor in src-tauri/src/execution/your_language.rs
pub struct YourLanguageExecutor;

impl YourLanguageExecutor {
    pub async fn execute(
        &self,
        code: &str,
        timeout_secs: u64,
        cancel_flag: Arc<Mutex<bool>>,
    ) -> Result<ExecutionResult, String> {
        // Implement temporary file creation, CLI execution, timeout & cancellation handling
    }
}

// 2. Register executor in src-tauri/src/execution/mod.rs
// 3. Add runtime PATH check in check_languages()
// 4. Update language definitions in src/types/index.ts
```

---

## 🛠️ Tech Stack & Architecture

- **Desktop Framework**: [Tauri v2](https://tauri.app/)
- **Frontend Framework**: [React 19](https://react.dev/) with [TypeScript 5.8](https://www.typescriptlang.org/)
- **Build Tool**: [Vite 7](https://vitejs.dev/)
- **Code Editor**: [Monaco Editor](https://microsoft.github.io/monaco-editor/) (`@monaco-editor/react`)
- **State Management**: [Zustand 5](https://github.com/pmndrs/zustand)
- **Styling**: [TailwindCSS v4](https://tailwindcss.com/) + [Lucide Icons](https://lucide.dev/)
- **Backend & Execution**: [Rust](https://www.rust-lang.org/) with Tokio async runtime & `which` crate

---

## 📋 Roadmap

- **v0.1** *(Current)*
  - ✅ Persistent floating window with light/dark themes
  - ✅ 6 core language executors (Kotlin, Python, Java, JS, TS, C++)
  - ✅ Live execution cancellation (Stop button)
  - ✅ Automatic runtime PATH detection & availability indicators
  - ✅ Monaco editor integration with `Ctrl+Enter` shortcut
- **v0.2**
  - 🔜 Custom editor themes (Dracula, Catppuccin, One Dark)
  - 🔜 Stdin input support for interactive console apps
  - 🔜 Code snippet bookmarking & template library
- **v0.3**
  - 🔜 Community language plugin support (Rust, Go, Ruby, PHP)
  - 🔜 Multi-tab scratchpad sessions

---

## 📝 License

MIT License — see [LICENSE](./LICENSE) for details.

---

## 📞 Support & Community

- 🐛 [Report a Bug](https://github.com/mohitsharma16/tatpar/issues)
- ✨ [Request a Feature](https://github.com/mohitsharma16/tatpar/issues)
- 💬 [GitHub Discussions](https://github.com/mohitsharma16/tatpar/discussions)