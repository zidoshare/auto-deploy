build:
	cargo build --release
install: build
	sudo cp target/release/auto-deploy /usr/local/bin
uninstall:
	sudo rm -f /usr/local/bin/auto-deploy
clear:
	cargo clean