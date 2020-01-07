## Broker - Real-time Zero-Code API Server

[![crates.io](https://meritbadge.herokuapp.com/broker)](https://crates.io/crates/broker)

### Purpose

The purpose of this library is to be your real-time zero-code API server. 

Broker is a SSE message broker that requires you write no backend code to have a full real-time API.

Broker is born from the need that rather than building a complex REST API with web-sockets and a SQL database to provide reactive web forms (like for React) there must be a simpler way.

Broker follows an insert-only/publish/subscribe paradigm rather than a REST CRUD paradigm. 


### How it works

In Broker you create a user, login, then insert an event and its data with a timestamp. Broker publishes the event when the timestamp is reached to the event stream via SSE. Broker keeps all event versions in its database that can be viewed. Broker can also cancel future events.

When the client first subscribes to the SSE connection all the latest events and data is sent to the client. Combined with sending the latest event via SSE when subscribed negates the necessity to do any GET API requests in the lifecycle of an event.

The side-effect of this system is that the latest event is the schema. Old events are saved in the database and are not changed but the latest event is the schema for the front-end. This is pure NoSQL as the backend is agnostic to the event data.


#### API

```html
/users 
```
- public endpoint
- POST JSON to create a user
```json
{"username":{...}, "password":{...}, "info":{...}}
```
- where {...} is for username and string, password a string, and info any JSON you want

will return
```json
{"id":{...}}
```
- where {...} is the uuid (string) of the user

```html
/login 
```
- public endpoint
- POST JSON to login
```json
{"username":{...}, "password":{...}}
```
- where {...} is for username a string and password a string

will return 
```json
{"jwt":{...}}
```
- where {...} is a JWT (string)

```html 
/events 
```
- public endpoint
- connect your sse-client to this endpoint

```html
/insert 
```
- authenticated endpoint
- POST JSON to insert an event
```json
{"event":{...}, "timestamp":{...}, "data":{...}}
```
- where {...} is for the event a string, timestamp is the epoch unix timestamp when you want the event to become the current event, and data is any JSON you want

will return
```json
{"id":{...}}
```
- where {...} is the uuid (string) of the event

```html
/events/{event}
```
- authenticated endpoint
- do a GET request where {event} is the name of the event you want the events queue (sorted by ascending timestamp)

```html
/events/{id}/cancel
``` 
- authenticated endpoint
- do a GET request where id is the uuid of the event to cancel a future event

### Features

* Very performant with a low memory footprint
* Real-time Event Stream via SSE
* CORS support
* Handles SSE client timeouts
* Provides user authentication with JWTs and Bcrypt(ed) passwords
* Handles future events via Epoch UNIX timestamp
* Stateful immutable event persistence
* Insert event via JSON POST request 
* Sync latest events on SSE client connection
* Event log via GET request
* Event cancellation via GET request

### Use

```rust
use broker::{broker_run};

#[actix_rt::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    broker_run("http://localhost:3000".to_owned()).await
}
```

- the only param is the origin you want to allow - wildcard is not supported
- the PORT needs to be passed in as an environment variable
- the ORIGIN needs to be passed in as an environment variable
- the EXPIRY (for jwts) needs to be passed in as an environment variable
- the SECRET (for jwts) needs to be passed in as an environment variable
- the file database saves to ``` ./tmp ``` of the project root

### Run Example

- ``` make ```

### Under the Hood

- [actix-web](https://crates.io/crates/actix-web) - web framework
- [sled](https://crates.io/crates/sled) - embedded database
- [sse-actix-web](https://crates.io/crates/sse-actix-web) - sse server

### Inspiration

* [React Hooks](https://reactjs.org/docs/hooks-intro.html)
* [Meteor](https://meteor.com)
* [MongoDB](https://www.mongodb.com/)
* [Pusher](https://pusher.com)
* [Event Sourcing](https://microservices.io/patterns/data/event-sourcing.html)
* [Best in Place](https://github.com/bernat/best_in_place)
* [Brock Whitten](https://www.youtube.com/watch?v=qljYMEfVukU)
