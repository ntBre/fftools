clippy:
	cargo clippy --workspace --tests

test:
	cargo test --workspace -- $(ARGS)

docflags :=

ifdef OPEN
	docflags += --open
endif

doc:
	cargo doc --no-deps $(docflags)

install: install.ffblame

install.ffblame:

install.%:
	cargo install --path $(subst .,,$(suffix $@))

run.ffsubset:

run.%:
	cargo run -p $(subst .,,$(suffix $@)) $(CARGO_ARGS) -- $(ARGS)
