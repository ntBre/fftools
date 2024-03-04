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
	cargo install --path ffblame

run.ffsubset:

run.%:
	cargo run -p $(subst .,,$(suffix $@)) $(ARGS)
