run:
	SAVE_PATH=./tmp/broker_data cargo run
release:
	SAVE_PATH=./tmp/broker_data cargo run --release
build:
	sudo snapcraft
edge:
	sudo snapcraft push --release edge *.snap
publish:
	sudo snapcraft push --release stable *.snap
cover:
	cargo tarpaulin
purge:
	sudo multipass delete snapcraft-broker && sudo multipass purge
