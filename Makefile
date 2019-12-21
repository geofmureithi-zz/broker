msg:
	curl -S -v --header "Content-Type: application/json" POST --data '{"event":"name", "data":{"name":"Rusty"}}'  http://localhost:8080/name