# Password Manager

A fast and secure password manager built with Rust and `eframe` (egui).

## Prerequisites

* **Rust** (1.80+ recommended) and Cargo: [Install Rust](https://www.rust-lang.org/tools/install)
* Git (to clone the repository)

---

## Installation

### 1. Clone the repository

```bash
git clone https://github.com/Lucop1911/password_manager.git
cd password_manager
```

### 2. Build the app from source

```bash
cargo build --release
```

The compiled binary will be located at:

```text
target/release/password_manager   # Linux/macOS
target\release\password_manager.exe  # Windows
```

### 3. Installing the Binary

#### Linux/macOS

You can manually move the binary to a directory in your PATH, for example:

```bash
chmod +x target/release/password_manager
mv target/release/password_manager ~/.local/bin/
```

After this, you can run the app from the terminal:

```bash
password_manager
```

If you want a desktop shortcut, you can manually create a `.desktop` file in `~/.local/share/applications/`.

#### Windows

Move the binary `password_manager.exe` to a preferred location and optionally add it to your PATH to run it from the command line.

---

### 4. Running the App

Run the binary directly from the terminal or file explorer:

```bash
password_manager   # Linux/macOS
password_manager.exe  # Windows
```

---

### 6. Updating

* Pull the latest changes:

```bash
git pull
cargo build --release
```

* Replace the binary in your chosen installation location.

---

### 7. License

This project is licensed under the **MIT License**. See [LICENSE](LICENSE) for details.
