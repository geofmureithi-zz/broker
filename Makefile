run:
	PORT=8080 SAVE_PATH=./tmp ORIGIN=http://localhost:3000 EXPIRY=3600 SECRET=secret cargo run
release:
	PORT=8080 SAVE_PATH=./tmp ORIGIN=http://localhost:3000 EXPIRY=3600 SECRET=secret cargo run --release
build:
	sudo snapcraft --use-lxd
publish:
	sudo snapcraft push --release stable *.snap
