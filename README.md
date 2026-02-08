# ESP32‑S3 Rust Development (Nix + direnv + espup)

This project is set up to build and flash Rust firmware to an **ESP32‑S3** using:

- **Nix + direnv** for a reproducible dev shell
- **rustup + espup** for the Xtensa Rust toolchain (`esp`)
- **espflash** for flashing and serial monitor

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

## 1. One‑time project setup

From the project root:

```sh
echo 'use flake' > .envrc
direnv allow
```

Then, inside the dev shell (direnv loads it automatically when you `cd` here):

```sh
espup install          # install ESP Xtensa toolchain (creates ~/export-esp.sh)
rustup override set esp
```

You can verify:

```sh
which rustc
rustc -V
rustup show active-toolchain
```

You should see:

- `which rustc` → `~/.rustup/toolchains/esp/bin/rustc`
- `rustc -V` → Xtensa Rust
- `active-toolchain` → `esp (directory override ...)`

---

## 2. Project‑local Cargo configuration

Create `.cargo/config.toml` in the project root:

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

Notes:

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
  - Sources `~/export-esp.sh` (espup environment).
  - Puts `esp` toolchain `bin` first in `PATH` so Xtensa `rustc` is used.

---

## 4. Daily workflow (build, flash, monitor)

From a terminal:

```sh
cd /Users/utopiaeh/Developer/mcu/guage-board   # direnv loads dev shell + esp toolchain
cargo build --release                          # build for xtensa-esp32s3-none-elf
DEFMT_LOG=info cargo espflash flash --release \
 --chip esp32s3 \
 --port /dev/tty.usbmodem2101 \
 --monitor
```

Notes:

- Use the port reported by `espflash list-ports` (on your machine: `/dev/tty.usbmodem2101`).
- If connection fails, put the board into bootloader mode:
  - Hold **BOOT**, tap **EN/RESET**, release BOOT after ~1s.
  - Then rerun the `cargo espflash` command.
- In the monitor:
  - `CTRL+R` resets the chip.
  - `CTRL+C` exits the monitor.

## 5. Why it works

- Before: Nix’s generic `rustc` was used → no Xtensa target or `core` for `xtensa-esp32s3-none-elf`.
- Now:
  - `espup` installed the `esp` Xtensa toolchain and `export-esp.sh`.
  - The dev shell sources `export-esp.sh` and puts `esp`’s `bin` first in `PATH`.
  - `rustup override set esp` selects the `esp` toolchain in this directory.
  - `.cargo/config.toml` targets `xtensa-esp32s3-none-elf` and uses `espflash` as runner.

Result: `cargo build --release` builds for ESP32‑S3, and `cargo espflash ... --monitor` flashes and shows runtime logs.

---

## 6. TL;DR

One-time in this project:

```sh
cd /Users/utopiaeh/Developer/mcu/guage-board
direnv allow
espup install
rustup override set esp
```

Day to day:

```sh
cd /Users/utopiaeh/Developer/mcu/guage-board
cargo build --release
cargo espflash flash --release --chip esp32s3 --port /dev/tty.usbmodem2101 --monitor
```
