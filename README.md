# ESP32‑S3 Rust Development (Nix + direnv + espup + Just)

This project is set up to build and flash Rust firmware to an **ESP32‑S3** using:

- **Nix + direnv** for a reproducible dev shell
- **rustup + espup** for the Xtensa Rust toolchain (`esp`)
- **espflash** for flashing and serial monitor
- **Just** for simplifying common commands

---

## 1. One‑time system / user setup (outside this repo)

You already have this, but for reference:

- NixOS / Nix configuration installs:
  - `pkgs.rustup` (so `rustup` exists globally),
  - Optional Nix Rust (`pkgs.rust-bin…`) for non‑ESP projects,
  - Tools like `rust-analyzer`, `pkg-config`, `openssl`, `cargo-llvm-cov`.

Minimal global requirements (already configured):

- Nix with flakes
- direnv integrated with your shell
- `rustup` installed globally via Nix
- A stable Rust toolchain via rustup:

  ```sh
  rustup install stable
  rustup default stable
  ```

---

## 2. One-time project setup

From the project root, run:

```sh
just setup
```

This command will:
- Create a `.envrc` file and allow direnv.
- Install the ESP Xtensa toolchain using `espup`.
- Set the `esp` toolchain as the default for this project using `rustup`.

You can verify the setup:

```sh
which rustc
rustc -V
rustup show active-toolchain
```

You should see output indicating the `esp` toolchain is active.

---

---

## 3. Project‑local Cargo configuration

A `.cargo/config.toml` file is provided in the project root to configure Cargo for the ESP32-S3 target and set `espflash` as the runner.

```toml
[build]
target = "xtensa-esp32s3-none-elf"

[target.xtensa-esp32s3-none-elf]
runner = "espflash flash --monitor"

[unstable]
build-std = ["core"]
```

**Notes:**
- Do **not** add manual `rustflags` like `-C target-cpu=esp32s3`.
- Let the ESP `esp` toolchain + target spec handle CPU settings.

---

## 3. What the dev shell does (short)

The `flake.nix` dev shell:

- Installs tools: `espup`, `espflash`, `ldproxy`, `esp-generate`, `rust-analyzer`.
- Sets:
  - `CARGO_BUILD_TARGET=xtensa-esp32s3-none-elf`
  - `RUST_BACKTRACE=1`
- In `shellHook`:
  - Inlines the ESP toolchain environment so you do not need a separate `~/export-esp.sh`. The hook will prepend your Xtensa `rustc` bin into `PATH` and set `LIBCLANG_PATH` to the esp-clang library directory (if those paths exist).
  - If `rustup` knows the `esp` toolchain, the dev shell will also prefer the `esp` toolchain's `rustc` bin when available.
  - Prints helpful informational messages (for example whether `espup` is available and what toolchain paths were used) and provides guidance to run `espup install` inside the dev shell if the toolchain is not yet present.

---

## 4. Daily workflow

After completing the one-time setup, you can use the following `just` commands for your daily workflow:

- **Build:**
  ```sh
  just build-release
  ```
  This builds the project in release mode for the `xtensa-esp32s3-none-elf` target.

- **Flash and Monitor:**
  ```sh
  just flash
  ```
  or
  ```sh
  just flash-monitor
  ```
  This command builds the project, flashes it to your ESP32-S3, and opens the serial monitor.
  **Important:** Ensure the `PORT` environment variable is set correctly, or update the `justfile` with your specific serial port (e.g., `/dev/tty.usbmodem2101`). You can list available ports with `just list-ports`.

- **List available serial ports:**
  ```sh
  just list-ports
  ```

- **Clean build artifacts:**
  ```sh
  just clean
  ```

---

## 5. Why it works

- **Nix + direnv:** Provide a consistent and reproducible development environment, ensuring all necessary tools and dependencies are available.
- **espup + rustup:** Install and manage the specific Xtensa Rust toolchain required for ESP32-S3 development.
- **`.cargo/config.toml`:** Configures `cargo` to use the correct target and runner for the ESP32-S3.
- **Justfile:** Simplifies common development tasks into easy-to-remember commands, reducing the cognitive load and potential for errors.

---

---

## 6. TL;DR

**One-time setup:**
```sh
cd /Users/utopiaeh/Developer/mcu/guage-board
just setup
```

**Day-to-day commands:**
```sh
cd /Users/utopiaeh/Developer/mcu/guage-board

# Build
just build-release

# Flash and monitor (ensure port is correct in Justfile or set as env var)
just flash
```
