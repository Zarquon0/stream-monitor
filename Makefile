ROOT_DIR := $(shell pwd)

.PHONY: monitor

monitor:
	cargo build --manifest-path $(ROOT_DIR)/Cargo.toml
	cp $(ROOT_DIR)/target/debug/stream-monitor $(ROOT_DIR)/streamonitor