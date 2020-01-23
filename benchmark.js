const http = require('http')

let drop_goal = 10000;
let dropped = 0;

let query = {
    host: 'localhost',
    port: 8080,
    path: '/events'
}

setInterval(() => {
    http.get('http://localhost:8080/events', () => print_status(true))
        .setTimeout(100, () => print_status(false))
        .on('error', () => {})
}, 20)

function print_status(accepting_connections) {
    process.stdout.write("\r\x1b[K");
    process.stdout.write(`Connections dropped: ${dropped}, accepting connections: ${accepting_connections}`);
}