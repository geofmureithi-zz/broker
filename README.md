## Broker - Real-time Zero-Code API Server

[![crates.io](https://meritbadge.herokuapp.com/broker)](https://crates.io/crates/broker)
[![broker](https://snapcraft.io//broker/badge.svg)](https://snapcraft.io/broker)

### Purpose

The purpose of this library is to be your real-time zero-code API server. 

Broker is a SSE message broker that requires you write no backend code to have a full real-time API.

Broker is born from the need that rather than building a complex REST API with web-sockets and a SQL database to provide reactive web forms (like for React) there must be a simpler way.

Broker follows an insert-only/publish/subscribe paradigm rather than a REST CRUD paradigm. 

### Features

* Very performant with a low memory footprint that uses about 20MB and 1 CPU thread
* Under 500 lines of code
* Ships as a [Linux Snap](https://snapcraft.io/broker) or [Rust Crate](https://crates.io/crates/broker)
* Secure Real-time Event Stream via SSE - requires the use of [broker-client](https://www.npmjs.com/package/broker-client)
* Has CORS support
* Provides user authentication with JWTs and Bcrypt(ed) passwords
* Handles future events via Epoch UNIX timestamp
* Uses [Cloudflare's Time Service](https://blog.cloudflare.com/secure-time/) and doesn't rely on your local's server time
* Stateful immutable event persistence
* Insert event via JSON POST request 
* Sync latest events on SSE client connection
* Event log via GET request
* Event cancellation via GET request

### How it works

In Broker you create a user, login, then insert an event with its data, a collection_id, and a timestamp. Broker publishes the event when the timestamp is reached to the event stream via SSE. Broker keeps all events its database that can be viewed in collections (by collection_id). Broker can also cancel future events.

When the client first subscribes to the SSE connection all the latest events and data is sent to the client. Combined with sending the latest event via SSE when subscribed negates the necessity to do any GET API requests in the lifecycle of an event.

The side-effect of this system is that the latest event is the schema. Old events are saved in the database and are not changed but the latest event is the schema for the front-end. This is pure NoSQL as the backend is agnostic to the event data.

### Recommeded Services/Libraries to use with Broker
* [broker-client](https://www.npmjs.com/package/broker-client) - the official front-end client for broker
* [broker-hook](https://www.npmjs.com/package/broker-hook) - the official react hook for broker
* [Integromat](https://www.integromat.com/) - No-code Event Scheduler that supports many apps like GitHub, Meetup, and etc.
* [React Hook Form](https://react-hook-form.com/) - Best form library for React
* [React Debounce Input](https://www.npmjs.com/package/react-debounce-input) - React input for Real-time Submission (Edit in Place forms)

### API

#### Step 1 - create a user

```html
POST /users 
```
- public endpoint
- POST JSON to create a user
```json
{"username":{...}, "password":{...}, "collection_id":{...}}
```
- where {...} is for username and string, password a string, and collection_id is the uuid of the event collection for user info

will return
```json
{"id":{...}}
```
- where {...} is the uuid (string) of the user

#### Step 2 - login with the user

```html
POST /login 
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

#### Step 3 - connect to SSE

```html 
GET /events 
```
- authenticated endpoint (Authorization: Bearer {jwt})
- connect your sse-client to this endpoint using [broker-client](https://www.npmjs.com/package/broker-client)
- note: broker-client uses fetch as eventsource doesn't support headers

#### Step 4 - insert an event

```html
POST /insert 
```
- authenticated endpoint (Authorization: Bearer {jwt})
- POST JSON to insert an event
```json
{"event":{...}, "collection_id":{...}, "timestamp":{...}, "data":{...}}
```
- where {...} is for the event a string, collection_id is an assigned uuid v4 for the event collection, timestamp is the epoch unix timestamp when you want the event to become the current event, and data is any JSON you want

will return
```json
{"event":{...}}
```
- where {...} is the event

#### Optional Endpoints

```html
GET /collections/{collection_id}
```
- authenticated endpoint (Authorization: Bearer {jwt})
- do a GET request where {collection_id} is the uuid of the collection you want (sorted by ascending timestamp)

will return
```json
{"events":{...}}
```
- where {...} is the array of events

```html
GET /user_events
``` 
- authenticated endpoint (Authorization: Bearer {jwt})
- do a GET request to get the user event collections (sorted by ascending timestamp)

will return
```json
{"info": {...}, "events":{...}}
```
- where (...) is for info a list of events for user info and events a list of all events that the user inserted

```html
GET /cancel/{id}
``` 
- authenticated endpoint (Authorization: Bearer {jwt})
- do a GET request where id is the uuid of the event to cancel a future event

will return
```json
{"event":{...}}
```
- where {...} is the event

### Use

```rust
use broker::broker;

#[tokio::main]
pub async fn main() {
    broker().await
}
```
- the origin needs to be passed in as a flag - wildcard is not supported
- the port needs to be passed in as a flag
- the expiry (for jwts) needs to be passed in as a flag
- the secret (for jwts) needs to be passed in as a flag
- the save_path where the embedded database will save needs to be passed in as an environment variable
- example: SAVE_PATH=./tmp/broker_data broker -port 8080 -origin http://localhost:3000 -expiry 3600 -secret secret

### Install Crate

``` cargo install broker ```
- the origin needs to be passed in as a flag - wildcard is not supported
- the port needs to be passed in as a flag
- the expiry (for jwts) needs to be passed in as a flag
- the secret (for jwts) needs to be passed in as a flag
- the save_path where the embedded database will save needs to be passed in as an environment variable
- example: SAVE_PATH=./tmp/broker_data broker -port 8080 -origin http://localhost:3000 -expiry 3600 -secret secret

### Install Linux Snap

``` sudo snap install broker ```
- note: does not run as a daemon as requires flags
- the origin needs to be passed in as a flag - wildcard is not supported
- the snap saves the database in [$SNAP_DATA/broker_data](https://snapcraft.io/docs/environment-variables) - which is /var/snap/broker/{rev#}/broker_data - where rev# is the revision number
- the port needs to be passed in as a flag
- the expiry (for jwts) needs to be passed in as a flag
- the secret (for jwts) needs to be passed in as a flag
- example: sudo broker -port 8080 -origin http://localhost:3000 -expiry 3600 -secret secret

### Run Example

- ``` make ```

### Run Integration Tests

- ``` cargo test ```

### Under the Hood

- [warp](https://crates.io/crates/warp) - web framework
- [sled](https://crates.io/crates/sled) - embedded database

### Inspiration

* [React Hooks](https://reactjs.org/docs/hooks-intro.html)
* [Meteor](https://meteor.com)
* [MongoDB](https://www.mongodb.com/)
* [Pusher](https://pusher.com)
* [Event Sourcing](https://microservices.io/patterns/data/event-sourcing.html)
* [Best in Place](https://github.com/bernat/best_in_place)
* [Brock Whitten](https://www.youtube.com/watch?v=qljYMEfVukU)

### Migrations

- from 2.0 to 3.0: the sse endpoint is now secure and requires to use the [broker-client](https://www.npmjs.com/package/broker-client) library
- from 1.0 to 2.0: the optional API endpoints URLs have been changed but have the same functionality
