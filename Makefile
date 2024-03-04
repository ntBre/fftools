clippy:
	cargo clippy --workspace --tests

install: install.ffblame

install.ffblame:
	cargo install --path ffblame
