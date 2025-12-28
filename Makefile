check:
	cargo check

run:
	if [ -z "$(SUPPORT_BUNDLE_PATH)" ]; then \
		echo "missing required argument: SUPPORT_BUNDLE_PATH"; \
		exit 1; \
	fi
	if [ -z "$(RESOURCE_NAME)" ]; then \
		echo "missing required argument: RESOURCE_NAME"; \
		exit 1; \
	fi
	cargo run -- -s "$(SUPPORT_BUNDLE_PATH)" -r "$(RESOURCE_NAME)"

release:
	cargo build --release

test:
	cargo test
