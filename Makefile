# Check if you are root
ifeq ($(shell id -u),0)
$(error Please, do not exec make as root)
endif

# Variables
PROJECT_NAME = e4docker
BINARY_NAME = $(PROJECT_NAME)
CONFIG_DIR = config
ASSETS_DIR = assets
INSTALL_DIR = /usr/bin
CONFIG_INSTALL_DIR = $(HOME)/.config/$(PROJECT_NAME)
ASSETS_INSTALL_DIR = $(CONFIG_INSTALL_DIR)/assets

# Default rule
all: linux debian windows

# Build the project for linux
linux:
	cargo build --release

# Build the debian package
debian:
	cargo deb

# Build the windows exe
windows:
	cargo build --release --target x86_64-pc-windows-gnu

# Install the project
install: linux
	sudo cp target/release/$(BINARY_NAME) $(INSTALL_DIR)
	mkdir -p $(CONFIG_INSTALL_DIR)
	cp $(CONFIG_DIR)/* $(CONFIG_INSTALL_DIR)
	mkdir -p $(ASSETS_INSTALL_DIR)
	cp $(ASSETS_DIR)/* $(ASSETS_INSTALL_DIR)

# Clean the project
clean:
	cargo clean
	rm -rf target

# Uninstall the project
uninstall:
	sudo rm $(INSTALL_DIR)/$(BINARY_NAME)
	rm -rf $(CONFIG_INSTALL_DIR)
	sudo rm -rf $(ASSETS_INSTALL_DIR)
