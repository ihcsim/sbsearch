check:
	cargo check
	cargo clippy -- -D warnings

run:
	if [ -z "$(SUPPORT_BUNDLE_PATH)" ]; then \
		echo "missing required argument: SUPPORT_BUNDLE_PATH"; \
		exit 1; \
	fi
	if [ -z "$(KEYWORD)" ]; then \
		echo "missing required argument: KEYWORD"; \
		exit 1; \
	fi
	if [ ! -z "$(LOG_LEVEL)" ]; then \
		LOG_LEVEL_ARG="-l $(LOG_LEVEL)"; \
	fi
	cargo run -- -s "$(SUPPORT_BUNDLE_PATH)" -k "$(KEYWORD)" $(LOG_LEVEL_ARG)

debug:
	$(MAKE) run LOG_LEVEL=debug

release:
	cargo build --release

test:
	cargo test -- --nocapture

fmt:
	cargo fmt -- --check

deps:
	cargo machete --fix || true
