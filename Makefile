build:
	PORT=8080 cargo run
rusty:
	curl -S -v --header "Content-Type: application/json" POST --data '{"event":"user", "data":{"user":"Rusty"}}'  http://localhost:8080/insert