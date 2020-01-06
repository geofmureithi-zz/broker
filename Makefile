build:
	PORT=8080 ORIGIN=http://localhost:3000 EXPIRY=3600 SECRET=secret cargo run
release:
	PORT=8080 ORIGIN=http://localhost:3000 EXPIRY=3600 SECRET=secret cargo run --release
