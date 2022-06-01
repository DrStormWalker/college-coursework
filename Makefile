release ?=

ifdef release
$(info Release mode)
	release-flag :=--release
else
$(info Debug mode)
	release-flag :=
endif

ROOT_DIR = $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
SS_CONFIG_FILE = $(ROOT_DIR)/run/config.toml
SS_LOG_DIR = $(ROOT_DIR)/run/logs

build:
	cargo build $(release-flag)

run:
	SS_CONFIG_FILE=$(SS_CONFIG_FILE) \
	SS_LOG_DIR=$(SS_LOG_DIR) \
	cargo run $(release-flag)
