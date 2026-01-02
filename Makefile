check:
	cargo check

run:
	if [ -z "$(SUPPORT_BUNDLE_PATH)" ]; then \
		echo "missing required argument: SUPPORT_BUNDLE_PATH"; \
		exit 1; \
	fi
	if [ -z "$(KEYWORD)" ]; then \
		echo "missing required argument: KEYWORD"; \
		exit 1; \
	fi
	cargo run -- -s "$(SUPPORT_BUNDLE_PATH)" -k "$(KEYWORD)"

release:
	cargo build --release

test:
	cargo test

deps:
	cargo machete --fix || 0
