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

.PHONY: install
install: install.ffblame install.ffdiff install.ffsubset

install.%:
	cargo install --path $(subst .,,$(suffix $@))

run.ffsubset:

run.ffsubset.default:
	$(call run,ffsubset,-r testfiles/dde.csv -d testfiles/industry.json -f \
						openff-2.1.0.offxml -s testfiles/subset.in)

run.ffchar.default:
	$(call run,ffchar,testfiles/dde.csv testfiles/industry.json openff-2.1.0.offxml)

run.%:
	$(call run,$(subst .,,$(suffix $@)))

# usage:
# $(call run,BIN_NAME,[args])
run = cargo run -p $1 $(CARGO_ARGS) -- $(ARGS) $2
