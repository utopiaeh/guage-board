# Justfile

# One-time project setup
setup:
    @echo "Setting up project (direnv allow, espup install, rustup override set esp)..."
    echo 'use flake' > .envrc
    direnv allow
    espup install
    rustup override set esp
    @echo "Project setup complete. Verify with 'which rustc', 'rustc -V', and 'rustup show active-toolchain'."

# Build the project in release mode
build-release:
    @echo "Building project in release mode..."
    cargo build --release

# Flash and monitor the ESP32-S3 board

# The port /dev/tty.usbmodem2101 is specific to your machine.
flash-monitor:
    @echo "Flashing and monitoring ESP32-S3 (port: /dev/tty.usbmodem2101)..."
    DEFMT_LOG=info cargo espflash flash --release \
     --chip esp32s3 \
     --port /dev/tty.usbmodem2101 \
     --monitor

# List available ESP devices
list-ports:
    @echo "Listing available ESP ports..."
    cargo espflash list-ports

# Clean the target directory
clean:
    @echo "Cleaning target directory..."
    cargo clean

# Run `rust-analyzer`
rust-analyzer:
    @echo "Running rust-analyzer..."
    rust-analyzer
