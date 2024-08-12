.PHONY: build
build:
	cargo auditable build --release

.PHONY: clean
clean:
	cargo clean
	rm -f tracing.folded
	rm -f Cargo.lock

.PHONY: flamegraph
flamegraph: run
    # WARNING: Doesn't work, killing run with Ctrl-C kills Makefile rule before
    # this gets to run. Need to implement keystroke/combo in Kaiseki that
    # gracefully exits to allow this rule to run.
	cat tracing.folded | inferno-flamegraph > flamegraph.svg

.PHONY: format
format:
	cargo fmt --all

.PHONY: lint
lint: KAISEKI_TARGET_DIR=$(ROOT)/target/debug/kaiseki
lint:
	cargo fmt --all --check
	cargo auditable clippy
	cargo audit
	cargo audit bin $(TARGET_DIR)

.PHONY: run
run: KAISEKI_MACHINE=chip8
run:
	cargo auditable run -- --machine $(KAISEKI_MACHINE)

.PHONY: test
test:
	cargo auditable test --all --all-features
