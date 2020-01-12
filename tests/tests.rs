extern crate broker;
use serde_json::json;
use std::sync::mpsc;
use std::{thread};
use actix_rt::System;
use actix_web::{dev::Server};
use chrono::prelude::*;

fn run_app(tx: mpsc::Sender<Server>, config: broker::Config) -> std::io::Result<()> {
    let mut sys = System::new("test");
    let srv = broker::server_create(config);
    let _ = tx.send(srv.clone());
    sys.block_on(srv)
}

#[test]
#[cfg_attr(tarpaulin, skip)]
fn full_test() {

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let id = uuid::Uuid::new_v4();
        let p = format!("./tmp/{}", id.to_string());
        let c = broker::Config{port: "8000".to_owned(), origin: "http://localhost:3000".to_owned(), expiry: 3600, secret: "secret".to_owned(), save_path: p};
    
        let _ = run_app(tx, c);
    });

    let _ = rx.recv().unwrap();

    let user1 = json!({"username": "rust22", "password": "rust", "collection_id":"3ca76743-8d99-4d3f-b85c-633ea456f90c"});
    let user2 = json!({"username": "rust23", "password": "rust", "collection_id":"3ca76743-8d99-4d3f-b85c-633ea456f90d"});
    let user1_login = json!({"username": "rust22", "password": "rust"});
    let event1 = json!({"event": "test", "collection_id": "3ca76743-8d99-4d3f-b85c-633ea456f90c", "timestamp": 1578667309, "data": "{}"});
    let now = Utc::now().timestamp();
    let x = now + 1000;
    let event2 = json!({"event": "user", "collection_id": "3ca76743-8d99-4d3f-b85c-633ea456f90d", "timestamp": x, "data": "{}"});

    let client = reqwest::blocking::Client::new();

    // create user 1 - want success
    let res = client.post("http://localhost:8000/users")
        .json(&user1)
        .send().unwrap()
        .status();
    assert_eq!(res, 200);

    // create user 2 - want success
    let res = client.post("http://localhost:8000/users")
        .json(&user2)
        .send().unwrap()
        .status();
    assert_eq!(res, 200);

    // try to create user 2 again - want failure
    let res = client.post("http://localhost:8000/users")
        .json(&user1)
        .send().unwrap()
        .status();
    assert_eq!(res, 400);

    // login for user 1 - want success
    let res = client.post("http://localhost:8000/login")
        .json(&user1_login)
        .send().unwrap()
        .text().unwrap();
    
    let token: broker::Token = serde_json::from_str(&res).unwrap();
    let bearer = format!("Bearer {}", token.jwt);

    // try posting event without auth - want failure
    let res = client.post("http://localhost:8000/insert")
        .json(&event1)
        .send().unwrap()
        .status();
    assert_eq!(res, 401);

    // try posting event with bad auth - want failure
    let res = client.post("http://localhost:8000/insert")
        .header("Authorization", "foo")
        .json(&event1)
        .send().unwrap()
        .status();
    assert_eq!(res, 401);

    // try posting event with bad auth - want failure
    let res = client.post("http://localhost:8000/insert")
        .header("Authorization", "Bearer 1234")
        .json(&event1)
        .send().unwrap()
        .status();
    assert_eq!(res, 401);

    // post event - want success
    let res = client.post("http://localhost:8000/insert")
        .header("Authorization", &bearer)
        .json(&event1)
        .send().unwrap();
    assert_eq!(res.status(), 200);
    let event : broker::Record = serde_json::from_str(&res.text().unwrap()).unwrap();
    assert_eq!(event.event.published, false);

    // post event - want success
    let res = client.post("http://localhost:8000/insert")
        .header("Authorization", &bearer)
        .json(&event2)
        .send().unwrap();
    assert_eq!(res.status(), 200);
    let event2 : broker::Record = serde_json::from_str(&res.text().unwrap()).unwrap();
    assert_eq!(event2.event.published, false);

    // try getting collection without auth - want failure
    let res = client.get("http://localhost:8000/events/collections/123")
        .send().unwrap()
        .status();
    assert_eq!(res, 401);

    // pause for a second to process job
    let one_second = std::time::Duration::from_millis(500);
    std::thread::sleep(one_second);

    // get collection - want success
    let res = client.get("http://localhost:8000/events/collections/3ca76743-8d99-4d3f-b85c-633ea456f90c")
        .header("Authorization", &bearer)
        .send().unwrap();
    assert_eq!(res.status(), 200);
    let events : broker::Collection = serde_json::from_str(&res.text().unwrap()).unwrap();
    assert_eq!(events.events[0].published, true);

    // try getting user without auth - want failure
    let res = client.get("http://localhost:8000/events/user")
        .send().unwrap()
        .status();
    assert_eq!(res, 401);

    // get user collection - want success
    let res = client.get("http://localhost:8000/events/user")
        .header("Authorization", &bearer)
        .send().unwrap()
        .status();
    assert_eq!(res, 200);

    // try cancelling without auth - want failure
    let client = reqwest::blocking::Client::new();
    let res = client.get("http://localhost:8000/events/123/cancel")
        .send().unwrap()
        .status();
    assert_eq!(res, 401);

    // try cancelling without auth - want failure
    let client = reqwest::blocking::Client::new();
    let url = format!("http://localhost:8000/events/{}/cancel", event2.event.id);
    let res = client.get(&url)
        .header("Authorization", &bearer)
        .send().unwrap();
    assert_eq!(res.status(), 200);
    let event : broker::Record = serde_json::from_str(&res.text().unwrap()).unwrap();
    assert_eq!(event.event.cancelled, true);

    // pause for a second to process job
    let one_second = std::time::Duration::from_millis(500);
    std::thread::sleep(one_second);

    // get collection - want success
    let res = client.get("http://localhost:8000/events/collections/3ca76743-8d99-4d3f-b85c-633ea456f90d")
        .header("Authorization", &bearer)
        .send().unwrap();
    assert_eq!(res.status(), 200);
    let events : broker::Collection = serde_json::from_str(&res.text().unwrap()).unwrap();
    assert_eq!(events.events[0].published, false);
}
