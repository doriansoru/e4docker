# Variabili
PROJECT_NAME = e4docker
BINARY_NAME = $(PROJECT_NAME)
CONFIG_DIR = config
ASSETS_DIR = assets
INSTALL_DIR = /usr/local/bin
CONFIG_INSTALL_DIR = $(HOME)/.config/$(PROJECT_NAME)
ASSETS_INSTALL_DIR = $(CONFIG_INSTALL_DIR)/assets

# Regola di default
all: build

# Compilazione del progetto
build:
	cargo build --release

# Installazione del progetto
install: build
	sudo cp target/release/$(BINARY_NAME) $(INSTALL_DIR)
	mkdir -p $(CONFIG_INSTALL_DIR)
	cp $(CONFIG_DIR)/* $(CONFIG_INSTALL_DIR)
	sudo mkdir -p $(ASSETS_INSTALL_DIR)
	sudo cp $(ASSETS_DIR)/* $(ASSETS_INSTALL_DIR)

# Pulizia del progetto
clean:
	cargo clean
	rm -rf target

# Disinstallazione del progetto
uninstall:
	sudo rm $(INSTALL_DIR)/$(BINARY_NAME)
	rm -rf $(CONFIG_INSTALL_DIR)
	sudo rm -rf $(ASSETS_INSTALL_DIR)
