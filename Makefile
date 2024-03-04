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

run.ffsubset.default:
	$(call run,ffsubset,-r testfiles/dde.csv -d testfiles/industry.json -f \
						openff-2.1.0.offxml -s testfiles/subset.in)

run.%:
	$(call run,$(subst .,,$(suffix $@)))

# usage:
# $(call run,BIN_NAME,[args])
run = cargo run -p $1 $(CARGO_ARGS) -- $(ARGS) $2
