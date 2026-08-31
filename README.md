# Tatpar तत्पर
 
> Always ready to run.  
> हमेशा चलाने के लिए तैयार।
 
A floating code editor that runs anywhere. Write code in **Kotlin, Python, Java, JavaScript, TypeScript, C++** and more — without opening an IDE.
 
---
 
## English Version
 
### 🎯 What is Tatpar?
 
Tatpar is a persistent, floating code editor that lives on your desktop. Launch it instantly with a global keyboard shortcut (`Ctrl+Shift+Space`) and run code in multiple languages locally, seeing output instantly.
 
**Core Idea:** Write. Run. Done. No IDE, no complexity.
 
### ⚡ Features
 
- **🪟 Floating Window** — Always accessible with `Ctrl+Shift+Space`
- **⚡ Instant Launch** — Opens in <500ms
- **🔧 Multiple Languages** — Kotlin, Python, Java, JavaScript, TypeScript, C++
- **🎨 Dark/Light Themes** — Easy on the eyes
- **🏠 Local Execution** — Your code never leaves your machine
- **🔒 Privacy-First** — No tracking, no accounts, no telemetry
- **⚙️ Customizable** — Hotkey, theme, compiler paths
- **🎯 Always Ready** — Window persists exactly where you left it
### 🚀 Quick Start
 
#### Installation
 
1. Download [Tatpar-0.1.0.msi](https://github.com/mohitsharma16/tatpar/releases)
2. Run the installer
3. Press `Ctrl+Shift+Space` to open Tatpar
#### First Run
 
The editor opens with a default Kotlin example:
 
```kotlin
fun main() {
    println("Hello, Tatpar!")
}
```
 
Press ▶ or `Ctrl+Enter` to execute.
 
**Output:**
```
Hello, Tatpar!
```
 
### 💻 Supported Languages (v0.1)
 
| Language | Status | Example |
|----------|--------|---------|
| **Kotlin** | ✅ Full Support | `println("Hello")` |
| **Python** | ✅ Full Support | `print("Hello")` |
| **Java** | ✅ Full Support | `System.out.println("Hello");` |
| **JavaScript** | ✅ Full Support | `console.log("Hello");` |
| **TypeScript** | ✅ Full Support | `console.log("Hello");` |
| **C++** | ✅ Full Support | `std::cout << "Hello";` |
| **HTML/CSS** | 🔜 Coming in v0.2 | Browser rendering |
 
### 🎮 Usage Examples
 
#### Quick Testing
 
```python
# In any application, press Ctrl+Shift+Space
# Tatpar appears, already open to Python
 
numbers = [1, 2, 3, 4, 5]
print(sum(numbers))
 
# Press ▶ Run
# Output: 15
```
 
#### Algorithm Practice
 
```javascript
// Learning data structures?
function mergeSort(arr) {
    if (arr.length <= 1) return arr;
    const mid = Math.floor(arr.length / 2);
    const left = mergeSort(arr.slice(0, mid));
    const right = mergeSort(arr.slice(mid));
    
    return merge(left, right);
}
 
function merge(left, right) {
    const result = [];
    while (left.length && right.length) {
        result.push(left[0] <= right[0] ? left.shift() : right.shift());
    }
    return [...result, ...left, ...right];
}
 
console.log(mergeSort([5, 2, 8, 1, 9]));
// Output: [1, 2, 5, 8, 9]
```
 
#### Stack Overflow Code Testing
 
Found a solution on Stack Overflow? Test it instantly without leaving the browser:
1. Copy code
2. Press `Ctrl+Shift+Space`
3. Paste and run
4. Back to browser in 10 seconds
### ⚙️ Settings
 
Access settings via the **⚙️ icon** in the header:
 
- **Language Selection** — Choose your language
- **Hotkey** — Customize the global shortcut
- **Theme** — Dark (default) or Light
- **Compiler Paths** — Auto-detected, but customizable
- **Execution Timeout** — Default 10s, adjustable per language
- **Always on Top** — Toggle persistent floating
- **Launch on Startup** — Optional
### 🔒 Security & Privacy
 
**What Tatpar Does:**
- ✅ Executes code locally on your machine
- ✅ Shows you all output
- ✅ Respects your privacy
**What Tatpar Doesn't Do:**
- ❌ Send your code anywhere
- ❌ Track your activity
- ❌ Require user accounts
- ❌ Collect analytics
- ❌ Access files outside temp directory
**Code Execution Safety:**
- Code runs with your user permissions (no admin needed)
- 10-second execution timeout (prevents infinite loops)
- Isolated temporary workspace
- Automatic cleanup after execution
### 🛠️ Development
 
#### Prerequisites
 
- Node.js 18+
- Rust 1.70+
- pnpm
- Git
#### Setup
 
```bash
# Clone repository
git clone https://github.com/mohitsharma16/tatpar.git
cd tatpar
 
# Install dependencies
pnpm install
 
# Start development
pnpm tauri dev
```
 
#### Build for Release
 
```bash
pnpm tauri build
```
 
Output: `src-tauri/target/release/bundle/msi/Tatpar-*.msi`
 
### 📋 Roadmap
 
**v0.1 (9 weeks)** — Current
- ✅ Floating window
- ✅ 6 languages (Kotlin, Python, Java, JS, TS, C++)
- ✅ Dark/Light themes
- ✅ Global hotkey
- ✅ Settings persistence
**v0.2**
- Editor themes (Catppuccin, Dracula, etc.)
- Language management UI
- Code templates
- macOS support
**v0.3**
- Community language plugins
- Advanced execution (stdin, network)
- Execution history export
- Linux support
**v1.0**
- Cloud sync (Tatpar Pro)
- Team collaboration
- AI code assistant
- Remote execution
### 🤝 Contributing
 
We love contributions! See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.
 
#### Add a New Language
 
The language architecture is designed for community contributions:
 
```rust
// 1. Create src-tauri/src/execution/rust.rs
pub struct RustExecutor { /* ... */ }
 
impl Language for RustExecutor {
    fn name(&self) -> &str { "Rust" }
    fn id(&self) -> &str { "rust" }
    fn execute(&self, code: &str) -> Result<ExecutionResult, String> {
        // Implement execution
    }
}
 
// 2. Register in src-tauri/src/execution/mod.rs
// 3. Test locally
// 4. Submit PR
```
 
See [CONTRIBUTING.md](./CONTRIBUTING.md#adding-a-new-language) for detailed steps.
 
### 📝 License
 
MIT License — See [LICENSE](./LICENSE) for details
 
### 📞 Support
 
- 📖 [Documentation](./docs/)
- 🐛 [Report a Bug](https://github.com/mohitsharma16/tatpar/issues)
- ✨ [Request a Feature](https://github.com/mohitsharma16/tatpar/issues)
- 💬 [Discussions](https://github.com/mohitsharma16/tatpar/discussions)
### 🙏 Acknowledgments
 
Built with:
- [Tauri](https://tauri.app/) — Desktop framework
- [React](https://react.dev/) — UI framework
- [Monaco Editor](https://microsoft.github.io/monaco-editor/) — Code editor
- [Rust](https://www.rust-lang.org/) — Backend