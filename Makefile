.PHONY: all clean amd64 rpi64 rpi32 win64 win32 macos-intel macos-arm deb linux windows macos prepare

# Linux Targets
TARGET_AMD64 = x86_64-unknown-linux-gnu
TARGET_RPI64 = aarch64-unknown-linux-gnu
TARGET_RPI32 = armv7-unknown-linux-gnueabihf

# Windows Targets
TARGET_WIN64 = x86_64-pc-windows-gnu
TARGET_WIN32 = i686-pc-windows-gnu

# macOS Targets (require native build or osxcross)
TARGET_MACOS_INTEL = x86_64-apple-darwin
TARGET_MACOS_ARM = aarch64-apple-darwin

# Output directory
DIST = dist

# Binary name
BINARY = gg-retro

all: linux windows

linux: amd64 rpi64 rpi32

windows: win64 win32

macos: macos-intel macos-arm

prepare:
	@mkdir -p $(DIST)
	@command -v cross >/dev/null 2>&1 || { echo "Installing cross..."; cargo install cross; }
	@command -v cargo-deb >/dev/null 2>&1 || { echo "Installing cargo-deb..."; cargo install cargo-deb; }

# AMD64
amd64: prepare
	@echo "Building for AMD64..."
	cross build --release --target $(TARGET_AMD64) -p gg-server
	cargo deb --target $(TARGET_AMD64) -p gg-server --no-build -o $(DIST)
	@echo "Package: $(DIST)/gg-retro_*_amd64.deb"

# Raspberry Pi 64-bit (Pi 3/4/5)
rpi64: prepare
	@echo "Building for Raspberry Pi 64-bit..."
	cross build --release --target $(TARGET_RPI64) -p gg-server
	cargo deb --target $(TARGET_RPI64) -p gg-server --no-build -o $(DIST)
	@echo "Package: $(DIST)/gg-retro_*_arm64.deb"

# Raspberry Pi 32-bit (Pi 2/3/4)
rpi32: prepare
	@echo "Building for Raspberry Pi 32-bit..."
	cross build --release --target $(TARGET_RPI32) -p gg-server
	cargo deb --target $(TARGET_RPI32) -p gg-server --no-build -o $(DIST)
	@echo "Package: $(DIST)/gg-retro_*_armhf.deb"

# Windows 64-bit (Windows 7+)
win64: prepare
	@echo "Building for Windows 64-bit..."
	cross build --release --target $(TARGET_WIN64) -p gg-server
	cp target/$(TARGET_WIN64)/release/$(BINARY).exe $(DIST)/$(BINARY)_win64.exe
	@echo "Package: $(DIST)/$(BINARY)_win64.exe"

# Windows 32-bit (Windows 7+, runs on 32-bit and 64-bit)
win32: prepare
	@echo "Building for Windows 32-bit..."
	cross build --release --target $(TARGET_WIN32) -p gg-server
	cp target/$(TARGET_WIN32)/release/$(BINARY).exe $(DIST)/$(BINARY)_win32.exe
	@echo "Package: $(DIST)/$(BINARY)_win32.exe"

# macOS Intel (macOS 10.12+) - requires native build on Mac
macos-intel: prepare
	@echo "Building for macOS Intel..."
	cargo build --release --target $(TARGET_MACOS_INTEL) -p gg-server
	cp target/$(TARGET_MACOS_INTEL)/release/$(BINARY) $(DIST)/$(BINARY)_macos_intel
	@echo "Package: $(DIST)/$(BINARY)_macos_intel"

# macOS Apple Silicon (macOS 11+) - requires native build on Mac
macos-arm: prepare
	@echo "Building for macOS Apple Silicon..."
	cargo build --release --target $(TARGET_MACOS_ARM) -p gg-server
	cp target/$(TARGET_MACOS_ARM)/release/$(BINARY) $(DIST)/$(BINARY)_macos_arm
	@echo "Package: $(DIST)/$(BINARY)_macos_arm"

# Build all .deb packages
deb: linux
	@echo "All packages built in $(DIST)/"
	@ls -la $(DIST)/*.deb 2>/dev/null || true

clean:
	cargo clean
	rm -rf $(DIST)
