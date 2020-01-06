build:
	PORT=8080 ORIGIN=http://localhost:3000 EXPIRY=3600 SECRET=secret cargo run
rusty:
	curl -S -v --header "Content-Type: application/json" POST --data '{"event":"user", "data":{"user":"Rusty"}}'  http://localhost:8080/insert
client:
	cd example && npm i && npm start
