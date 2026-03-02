.PHONY: fmt clippy test guardrails all

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

guardrails:
	bash scripts/check-install-commands.sh
	bash scripts/check-runtime-hardening.sh
	bash scripts/test-primary-problems.sh
	bash scripts/check-docs-sync.sh
	bash scripts/check-canonical-sister.sh

all: fmt clippy test guardrails
