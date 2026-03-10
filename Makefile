# Use SILENT to stop `make` from logging each command it runs.
.SILENT:

# Some pre-requisites for this Makefile, beyond the basic Rust / cargo installation.
# TODO: Put this into its own Make command.
#
# For generating the LICENSES file:
# `cargo install cargo-license`
#
# For validating that the things we depend on have acceptable licenses.
# See installation steps: https://embarkstudios.github.io/cargo-deny/index.html
# `cargo install --locked cargo-deny`
#
# For viewing stats about this repo. Line counts, etc
# `cargo install tokei`
#
# For running `add_actions`, python3 is needed.
#
# For native binary compression, `upx` needs to be on your PATH:
#  https://github.com/upx/upx/releases/latest
# For wasm binary compression (this is needed for `wasm-opt`):
#  `sudo apt install binaryen`

# Size limits for presubmit checks (in MB)
# Feel free to change these. They only exist to prevent unexpected binary size increases.
MAX_NATIVE_SIZE_MB := 6
MAX_WASM_SIZE_MB := 8

# Name of the Rust crate containing the binary.
APP_PACKAGE := app
# Relative directory where the app crate is located
APP_DIR := src/crates/app

# This is the relative directory to the root that this will be deployed at. This only matters for wasm builds.
# For example, if I want to serve at: https://omustardo.com/share/my_app/2026-02-15/, then this needs to be: /share/my_app/2026-02-15/
PUBLIC_URL=/share/greek_computer/

all: run

build: build-native-dev build-wasm-dev
# Similar to `build`, but with a few optimizations to binary size as well as different compiler flags that prevent
# debug assertions.
# The `--release` flag is used by cargo to turn off debug mode. Some affected code is guarded by `#[cfg(debug_assertions)]`.
build-release: build-release-native build-release-wasm
# The same as `build-release`, but with additional compression applied to the output binaries.
build-deployable: build-release-native-compressed build-release-wasm-compressed

build-common: licenses-check

build-native-dev: build-common
	cd src && cargo build --package $(APP_PACKAGE) --quiet
build-release-native: build-common
	cd src && cargo build --package $(APP_PACKAGE) --release --quiet

# Note that this doesn't use build-release-native-compressed, since it's just meant for local execution and compression is slow.
run-release-native: build-release-native
	cd src && cargo run --package $(APP_PACKAGE) --release --quiet

build-wasm-dev: build-common
	cd $(APP_DIR) && trunk build --quiet --public-url=$(PUBLIC_URL)
build-release-wasm: build-common
	# This will generate a `dist` directory wherever it is run from (APP_DIR in this case).
	# To serve this, just serve the directory in a HTTP server.
	cd $(APP_DIR) && trunk build --release --quiet --public-url=$(PUBLIC_URL)
run-release-wasm: build-release-wasm
	cd $(APP_DIR) && trunk serve --release --quiet --public-URL=$(PUBLIC_URL)

# Compress the release binary with UPX. https://upx.github.io/
build-release-native-compressed: build-release-native
	@BINARY=$$(find . -name "$(APP_PACKAGE)" -path "*/target/release/*" | head -1); \
	if [ -n "$$BINARY" ]; then \
		if command -v upx >/dev/null 2>&1; then \
			echo "Compressing binary with UPX..."; \
			upx --best --lzma "$$BINARY" --quiet; \
			echo "Binary compressed ✓"; \
		else \
			echo "Warning: UPX not found. Install with your package manager or from https://upx.github.io/"; \
		fi; \
	else \
		echo "❌ Binary not found for compression"; exit 1; \
	fi;

build-release-wasm-compressed: build-release-wasm
# TODO: This code doesn't actually reduce the binary size. Perhaps cargo already runs something similar internally?
#       If I can't figure out any potential compression options, I should remove this.
#
#	@WASM_FILE=$$(find . -name "*.wasm" -path "*/dist/*" | head -1); \
#	if [ -n "$$WASM_FILE" ]; then \
#		if command -v wasm-opt >/dev/null 2>&1; then \
#			echo "Optimizing WASM with wasm-opt..."; \
#			wasm-opt -Oz "$$WASM_FILE" -o "$$WASM_FILE"; \
#			echo "WASM optimized ✓"; \
#		else \
#			echo "Warning: wasm-opt not found. Install with: sudo apt install binaryen"; \
#		fi; \
#	else \
#		echo "❌ WASM file not found for optimization"; exit 1; \
#	fi;

test:
	# If the tests pass, print a short message rather than the absurdly verbose default.
	# If the tests fail, print the usual output.
	cd src && cargo test --workspace --quiet > /tmp/test_output 2>&1 && (echo "All tests passed ✓" && rm -f /tmp/test_output) || (cat /tmp/test_output && rm -f /tmp/test_output && exit 1)

test-release:
	cd src && cargo test --workspace --release --quiet > /tmp/test_output 2>&1 && (echo "All tests passed ✓" && rm -f /tmp/test_output) || (cat /tmp/test_output && rm -f /tmp/test_output && exit 1)

# Use `test-release` to avoid compiling the same release binary more than needed.
presubmit: licenses-check test-release size-check
	# Note that an additional `--check` flag can make this blocking, but I don't think it's worth the effort.
	# I'm already running `cargo fmt` on save.
	cd src && cargo fmt --all

# Push local commits to github. Use SSH to connect to the remote repository rather than HTTP since it
# avoids the need to log in each time. Using SSH requires having created a SSH key pair on the local computer,
# and having added the public key to your Github account. For more info, see SSH in https://www.omustardo.com/help/git.html
REPO_SSH_URL := git@github.com:Omustardo/greek_computer.git
push: presubmit
	echo "Presubmit checks passed ✓"
	echo "Pushing to local changes to github..."
	git push $(REPO_SSH_URL) main
	echo "Push completed ✓"

run:
	cd src && cargo run --package $(APP_PACKAGE)

web:
	echo "Load the website at http://localhost:50051/index.html#dev"
	cd $(APP_DIR) && trunk serve --port=50051 --public-url=$(PUBLIC_URL) # trunk will build and serve at 127.0.0.1:50051 and rebuild automatically when making code changes. Use http://localhost:50051/index.html#dev as `#dev` prevents caching (implemented in main.rs).

keep-sorted:
	# https://github.com/google/keep-sorted : Sort lines between a pair of "keep-sorted start" and "keep-sorted end".
	# This requires:
	#   go install github.com/google/keep-sorted@v0.6.1
	# This installation may have an message like: "go: github.com/google/keep-sorted@v0.6.1 requires go >= 1.23.1; switching to go1.24.6"
    # You can ignore that. The installation probably worked. Try running this again.
	cd src && \
     if ! command -v ~/go/bin/keep-sorted >/dev/null 2>&1; then \
         echo "Warning: keep-sorted binary not found at ~/go/bin/keep-sorted"; \
         echo "Install with: go install github.com/google/keep-sorted@v0.6.1\n"; \
     else \
         find . -name "*.rs" -exec ~/go/bin/keep-sorted {} \; ; \
     fi

# Tidy can only be run from a clean workspace (no modified files). It has bitten me with new fixes too many times.
# These fixes affect loads of files and aren't guaranteed to even compile! Better to leave it to a workspace where
# it won't mess with anything.
tidy: keep-sorted licenses-generate
	# machete removes unused deps. https://github.com/bnjbvr/cargo-machete
	cd src && cargo machete
	cd src && cargo fmt --all
	# "the working directory of this package has uncommitted changes, and `cargo fix` can potentially perform destructive changes; if you'd like to suppress this error pass `--allow-dirty`, or commit the changes to these files"
	# clippy uses the "lints" configuration within `Cargo.toml` for detailed options.
	cd src && cargo clippy --fix --allow-dirty

licenses-generate:
	# Ignore the line starting with "N/A". This appears to only contain my own crates! I don't know why they are
	# being listed here, but better to remove them.
	cd src && cargo license --avoid-dev-deps | grep -v "^N/A" > ../LICENSES

licenses-check: licenses-generate
	cd src && cargo deny check

size:
	echo "Build sizes:"
	find . -name "$(APP_PACKAGE)" -path "*/target/release/*" -exec ls -lh {} \; | awk '{print "Native binary: " $$5 " (" $$9 ")"}'
	find . -name "*.wasm" -path "*/dist/*" -exec ls -lh {} \; | awk '{print "WASM file: " $$5 " (" $$9 ")"}'

size-check: build-deployable
	echo "Checking build sizes..."
	# Find and check native binary
	BINARY=$$(find . -name "$(APP_PACKAGE)" -path "*/target/release/*" | head -1); \
	if [ -n "$$BINARY" ]; then \
		SIZE_KB=$$(stat -c%s "$$BINARY" | awk '{print int($$1/1024)}'); \
		SIZE_MB=$$((SIZE_KB / 1024)); \
		echo "Native binary: $${SIZE_MB}MB (limit: $(MAX_NATIVE_SIZE_MB)MB)"; \
		if [ $$SIZE_MB -gt $(MAX_NATIVE_SIZE_MB) ]; then \
			echo "❌ Native binary too large"; exit 1; \
		fi; \
	else \
		echo "❌ Native binary not found"; exit 1; \
	fi
	# Find and check WASM file
	WASM=$$(find . -name "*.wasm" -path "*/dist/*" | head -1); \
	if [ -n "$$WASM" ]; then \
		SIZE_KB=$$(stat -c%s "$$WASM" | awk '{print int($$1/1024)}'); \
		SIZE_MB=$$((SIZE_KB / 1024)); \
		echo "WASM file: $${SIZE_MB}MB (limit: $(MAX_WASM_SIZE_MB)MB)"; \
		if [ $$SIZE_MB -gt $(MAX_WASM_SIZE_MB) ]; then \
			echo "❌ WASM file too large"; exit 1; \
		fi; \
	else \
		echo "❌ WASM file not found"; exit 1; \
	fi
	echo "All sizes OK ✓"

# See statistics about this project.
stats:
	cd src && tokei --sort=lines --compact

remove_savefile:
	# TEMPLATE_TODO: Change my_app to your app_name.
	rm -f ~/.local/share/my_app/app.ron

# Remove build artifacts. Does not remove savefiles.
clean:
	test -n "$(APP_DIR)" # Test that the variable is set, otherwise it could delete root dirs.
	cd src && cargo clean
	rm -rf $(APP_DIR)/pkg/
	rm -rf $(APP_DIR)/dist/
	# TEMPLATE_TODO: Change my_app to your app_name.
	rm -rf /tmp/my_app/*
