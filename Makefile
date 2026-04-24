.PHONY: all fmt build release app install install-app install-config install-shell-integration check test docs servedocs

all: build

test:
	cargo nextest run
	cargo nextest run -p wezterm-escape-parser # no_std by default

check:
	cargo check
	cargo check -p wezterm-escape-parser
	cargo check -p wezterm-cell
	cargo check -p wezterm-surface
	cargo check -p wezterm-ssh

build:
	cargo build $(BUILD_OPTS) -p wezterm
	cargo build $(BUILD_OPTS) -p wezterm-gui
	cargo build $(BUILD_OPTS) -p wezterm-mux-server
	cargo build $(BUILD_OPTS) -p strip-ansi-escapes

release:
	cargo build --release -p wezterm -p wezterm-gui -p wezterm-mux-server -p strip-ansi-escapes

app: release
	bash ci/deploy.sh target

install-config:
	bash custom/wezterm/install-config.sh

install-shell-integration:
	bash custom/wezterm/install-shell-integration.sh

install-app:
	bash custom/wezterm/install-app.sh

install: install-app install-config install-shell-integration

fmt:
	cargo +nightly fmt

docs:
	ci/build-docs.sh

servedocs:
	ci/build-docs.sh serve
