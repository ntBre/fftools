clippy:
	cargo clippy --workspace --tests

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
	cargo run -p $(subst .,,$(suffix $@)) $(ARGS)
