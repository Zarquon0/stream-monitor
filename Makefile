ROOT_DIR := $(shell pwd)

.PHONY: debug release

debug:
	cargo build --manifest-path $(ROOT_DIR)/monitor/Cargo.toml
	cp $(ROOT_DIR)/target/debug/monitor $(ROOT_DIR)/streamonitor

# Must be run with sudo
release:
	cargo build --release --manifest-path $(ROOT_DIR)/monitor/Cargo.toml
	cp $(ROOT_DIR)/target/release/monitor $(ROOT_DIR)/streamonitor