# ESP32‑S3 Rust Development (Nix + direnv + rustup + espup)

This project targets **ESP32‑S3** using the official **espup** toolchain on top of a **Nix + direnv** development shell.

Short version of what happens:

- **Nix + direnv** give you a reproducible dev shell with `espup`, `espflash`, `esp-generate`, `ldproxy`, `rust-analyzer`.
- **rustup** (installed via Nix globally) is the Rust toolchain manager.
- **espup** installs an `esp` Xtensa Rust toolchain and writes `~/export-esp.sh`.
- The project’s **flake dev shell**:
  - Sources `~/export-esp.sh`.
  - Forces the `esp` toolchain’s `bin` directory to the front of `PATH`.
  - Sets `CARGO_BUILD_TARGET=xtensa-esp32s3-none-elf`.
- `.cargo/config.toml` ensures Cargo always builds for ESP32‑S3 and uses `espflash` as the runner.

Result: when you `cd` into this repo, you automatically get an ESP32‑S3‑ready Rust environment.

---

## 1. One‑time system / user setup (outside this repo)

You already have this, but for reference:

- NixOS / Nix configuration installs:
  - `pkgs.rustup` (so `rustup` exists globally),
  - Optional Nix Rust (`pkgs.rust-bin…`) for non‑ESP projects,
  - Tools like `rust-analyzer`, `pkg-config`, `openssl`, `cargo-llvm-cov`.

Example (simplified):

```nix
{ pkgs, ... }:

{
  environment.systemPackages = [
    (pkgs.rust-bin.stable.latest.default.override {
      extensions = [
        "rust-src"
        "rustfmt"
        "clippy"
        "llvm-tools"
      ];
    })

    pkgs.rust-analyzer
    pkgs.pkg-config
    pkgs.openssl
    pkgs.cargo-llvm-cov

    pkgs.rustup
  ];

  environment.variables = {
    CARGO_HOME = "$HOME/.cargo";
  };
}
```

Then, once:

```sh
rustup install stable
rustup default stable
```

This gives `rustup` a normal stable toolchain to start from.

---

## 2. One‑time project setup

### 2.1 Enable direnv for this repo

From the project root:

```sh
echo 'use flake' > .envrc
direnv allow
```

Now, every time you `cd` into this directory, direnv will enter the Nix flake dev shell and run its `shellHook`.

### 2.2 Install the ESP Xtensa toolchain with espup (once)

Inside this project (after direnv loads the dev shell):

```sh
espup install
```

What this does:

- Installs the **ESP Xtensa Rust toolchain** (toolchain name `esp`).
- Installs LLVM + GCC for Xtensa.
- Writes:

  ```text
  $HOME/export-esp.sh
  ```

You only need to do this **once** (or when you want to update the ESP toolchain).

### 2.3 Tell rustup to use the `esp` toolchain in this directory

Still in the project root:

```sh
rustup override set esp
```

This means:

- For `/Users/utopiaeh/Developer/mcu/guage-board`, rustup always uses the `esp` toolchain.
- Combined with the dev shell’s PATH fix, this ensures `cargo` and `rustc` are the Xtensa versions, not Nix’s global `rustc`.

You can verify:

```sh
which rustc
rustc -V
rustup show active-toolchain
```

Expected:

- `which rustc` → `~/.rustup/toolchains/esp/bin/rustc`
- `rustc -V` → Xtensa Rust (1.92.0.0)
- `active-toolchain` → `esp (directory override ...)`

---

## 3. Project‑local Cargo configuration

Create `.cargo/config.toml` in the project root with:

```toml
[build]
target = "xtensa-esp32s3-none-elf"

[target.xtensa-esp32s3-none-elf]
runner = "espflash flash --monitor"

[unstable]
build-std = ["core"]
```

What this does:

- Sets **ESP32‑S3** (`xtensa-esp32s3-none-elf`) as the default target.
- Uses `espflash flash --monitor` when you run `cargo run`.
- Tells `cargo` / `rustc` to build `core` for this no_std target.

Important:

- Do **not** add manual `rustflags` like `-C target-cpu=esp32s3`. Those caused the earlier warnings:
  > `'esp32s3' is not a recognized processor for this target (ignoring processor)`
- Let the ESP `esp` toolchain + target spec handle CPU settings.

---

## 4. What the flake dev shell does

The project’s `flake.nix` dev shell:

- Installs tools you need for ESP:
  - `espup`
  - `espflash`
  - `ldproxy`
  - `esp-generate`
  - `rust-analyzer`
- Sets helpful env vars:
  - `CARGO_BUILD_TARGET=xtensa-esp32s3-none-elf`
  - `RUST_BACKTRACE=1`
- In its `shellHook` it:
  1. Sources `~/export-esp.sh` if it exists:
     - This sets up the ESP environment that espup created.
  2. Uses `rustup which rustc --toolchain esp` to find the `esp` toolchain bin dir.
  3. Prepends that bin dir to `PATH` so that:
     - `/Users/utopiaeh/.rustup/toolchains/esp/bin/rustc` wins over `/run/current-system/sw/bin/rustc`.
  4. Prints a short message like:

     ```text
     [ESP32-S3 dev shell]
     Loaded ESP Rust toolchain from /Users/utopiaeh/export-esp.sh
     Using esp toolchain rustc from: /Users/utopiaeh/.rustup/toolchains/esp/bin
     ```

This was the crucial fix: earlier builds used Nix’s generic `rustc` (1.93.0), so `xtensa-esp32s3-none-elf` and `core` for Xtensa were missing. After forcing the `esp` toolchain bin to the front of `PATH`, `cargo build` now uses the correct Xtensa `rustc`.

---

## 5. Daily workflow

After all the above is in place:

1. Enter the project:

   ```sh
   cd /Users/utopiaeh/Developer/mcu/guage-board
   ```

   You should see:

   ```text
   [ESP32-S3 dev shell]
   Loaded ESP Rust toolchain from /Users/utopiaeh/export-esp.sh
   Using esp toolchain rustc from: /Users/utopiaeh/.rustup/toolchains/esp/bin
   ```

2. Build:

   ```sh
   cargo build --release
   ```

   - Uses the `esp` Xtensa toolchain.
   - Targets `xtensa-esp32s3-none-elf`.
   - Builds `core` for this target (via `build-std = ["core"]`).

3. Flash:

   ```sh
   cargo espflash flash --release
   ```

   or:

   ```sh
   cargo run --release
   ```

   - `cargo run` uses the configured runner (`espflash flash --monitor`), so it builds, flashes, and opens a serial monitor.

---

## 6. Quick explanation of “what changed” and “why it works now”

Previously:

- `rustc` was coming from Nix (`/run/current-system/sw/bin/rustc`, 1.93.0).
- That toolchain:
  - Did not know about `xtensa-esp32s3-none-elf`,
  - Did not provide `core` for Xtensa,
  - Ignored the `esp32s3` processor flags, hence:
    - `can't find crate for core`
    - `'esp32s3' is not a recognized processor for this target` warnings.

Now:

- `espup` installed the **esp** Xtensa toolchain and `export-esp.sh`.
- The project:
  - Uses `rustup override set esp` to select that toolchain.
  - Ensures PATH prefers `~/.rustup/toolchains/esp/bin` inside the dev shell.
  - Configures Cargo (`.cargo/config.toml`) to target `xtensa-esp32s3-none-elf` and build `core`.

So `cargo build --release` uses the **correct Xtensa Rust toolchain** and the build succeeds.

---

## 7. TL;DR commands you actually run

After the one-time setup:

```sh
cd /Users/utopiaeh/Developer/mcu/guage-board
# (direnv loads the flake dev shell + esp toolchain)

cargo build --release
cargo espflash flash --release   # or: cargo run --release
```

That’s all you need day to day.