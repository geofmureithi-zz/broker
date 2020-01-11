extern crate broker;
use serde_json::json;
use std::sync::mpsc;
use std::{thread};
use actix_rt::System;
use actix_web::{dev::Server};

fn run_app(tx: mpsc::Sender<Server>, config: broker::Config) -> std::io::Result<()> {
    let mut sys = System::new("test");
    let srv = broker::server_create(config);
    let _ = tx.send(srv.clone());
    sys.block_on(srv)
}

#[test]
#[cfg_attr(tarpaulin, skip)]
fn create_two_unique_users() {

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let id = uuid::Uuid::new_v4();
        let p = format!("./tmp/{}", id.to_string());
        let c = broker::Config{port: "8000".to_owned(), origin: "http://localhost:3000".to_owned(), expiry: 3600, secret: "secret".to_owned(), save_path: p};
    
        let _ = run_app(tx, c);
    });

    let _ = rx.recv().unwrap();

    let user1 = json!({"username": "rust1", "password": "rust", "collection_id":"3ca76743-8d99-4d3f-b85c-633ea456f90c"});
    let user2 = json!({"username": "rust2", "password": "rust", "collection_id":"3ca76743-8d99-4d3f-b85c-633ea456f90c"});

    let client = reqwest::blocking::Client::new();
    let res = client.post("http://localhost:8000/users")
        .json(&user1)
        .send().unwrap()
        .status();
    assert_eq!(res, 200);

    let client = reqwest::blocking::Client::new();
    let res = client.post("http://localhost:8000/users")
        .json(&user2)
        .send().unwrap()
        .status();
    assert_eq!(res, 200);
}

#[test]
#[cfg_attr(tarpaulin, skip)]
fn fail_on_non_unique_users() {

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let id = uuid::Uuid::new_v4();
        let p = format!("./tmp/{}", id.to_string());
        let c = broker::Config{port: "8001".to_owned(), origin: "http://localhost:3000".to_owned(), expiry: 3600, secret: "secret".to_owned(), save_path: p};
    
        let _ = run_app(tx, c);
    });

    let _ = rx.recv().unwrap();

    let user1 = json!({"username": "rust11", "password": "rust", "collection_id":"3ca76743-8d99-4d3f-b85c-633ea456f90c"});
    let user2 = json!({"username": "rust11", "password": "rust", "collection_id":"3ca76743-8d99-4d3f-b85c-633ea456f90c"});

    let client = reqwest::blocking::Client::new();
    let res = client.post("http://localhost:8001/users")
        .json(&user1)
        .send().unwrap()
        .status();
    assert_eq!(res, 200);

    let client = reqwest::blocking::Client::new();
    let res = client.post("http://localhost:8001/users")
        .json(&user2)
        .send().unwrap()
        .status();
    assert_eq!(res, 400);
}

#[test]
#[cfg_attr(tarpaulin, skip)]
fn insert_success() {

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let id = uuid::Uuid::new_v4();
        let p = format!("./tmp/{}", id.to_string());
        let c = broker::Config{port: "8002".to_owned(), origin: "http://localhost:3000".to_owned(), expiry: 3600, secret: "secret".to_owned(), save_path: p};
    
        let _ = run_app(tx, c);
    });

    let _ = rx.recv().unwrap();

    let user1 = json!({"username": "rust22", "password": "rust", "collection_id":"3ca76743-8d99-4d3f-b85c-633ea456f90c"});
    let user1_login = json!({"username": "rust22", "password": "rust"});
    let event = json!({"event": "test", "collection_id": "3ca76743-8d99-4d3f-b85c-633ea456f90c", "timestamp": 1578667309, "data": "{}"});

    let client = reqwest::blocking::Client::new();
    let res = client.post("http://localhost:8002/users")
        .json(&user1)
        .send().unwrap()
        .status();
    assert_eq!(res, 200);

    let client = reqwest::blocking::Client::new();
    let res = client.post("http://localhost:8002/login")
        .json(&user1_login)
        .send().unwrap()
        .text().unwrap();
    
    let token: broker::Token = serde_json::from_str(&res).unwrap();
    let bearer = format!("Bearer {}", token.jwt);

    let client = reqwest::blocking::Client::new();
    let res = client.post("http://localhost:8002/insert")
        .header("Authorization", bearer)
        .json(&event)
        .send().unwrap()
        .status();
    assert_eq!(res, 200);
}

#[test]
#[cfg_attr(tarpaulin, skip)]
fn insert_failure_no_auth_header() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let id = uuid::Uuid::new_v4();
        let p = format!("./tmp/{}", id.to_string());
        let c = broker::Config{port: "8003".to_owned(), origin: "http://localhost:3000".to_owned(), expiry: 3600, secret: "secret".to_owned(), save_path: p};
    
        let _ = run_app(tx, c);
    });

    let _ = rx.recv().unwrap();

    let event = json!({"event": "test", "collection_id": "3ca76743-8d99-4d3f-b85c-633ea456f90c", "timestamp": 1578667309, "data": "{}"});

    let client = reqwest::blocking::Client::new();
    let res = client.post("http://localhost:8003/insert")
        .json(&event)
        .send().unwrap()
        .status();
    assert_eq!(res, 401);
}

#[test]
#[cfg_attr(tarpaulin, skip)]
fn collection_failure_no_auth_header() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let id = uuid::Uuid::new_v4();
        let p = format!("./tmp/{}", id.to_string());
        let c = broker::Config{port: "8004".to_owned(), origin: "http://localhost:3000".to_owned(), expiry: 3600, secret: "secret".to_owned(), save_path: p};
    
        let _ = run_app(tx, c);
    });

    let _ = rx.recv().unwrap();

    let client = reqwest::blocking::Client::new();
    let res = client.get("http://localhost:8004/events/collections/123")
        .send().unwrap()
        .status();
    assert_eq!(res, 401);
}

#[test]
#[cfg_attr(tarpaulin, skip)]
fn user_failure_no_auth_header() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let id = uuid::Uuid::new_v4();
        let p = format!("./tmp/{}", id.to_string());
        let c = broker::Config{port: "8005".to_owned(), origin: "http://localhost:3000".to_owned(), expiry: 3600, secret: "secret".to_owned(), save_path: p};
    
        let _ = run_app(tx, c);
    });

    let _ = rx.recv().unwrap();

    let client = reqwest::blocking::Client::new();
    let res = client.get("http://localhost:8005/events/user")
        .send().unwrap()
        .status();
    assert_eq!(res, 401);
}

#[test]
#[cfg_attr(tarpaulin, skip)]
fn cancel_failure_no_auth_header() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let id = uuid::Uuid::new_v4();
        let p = format!("./tmp/{}", id.to_string());
        let c = broker::Config{port: "8006".to_owned(), origin: "http://localhost:3000".to_owned(), expiry: 3600, secret: "secret".to_owned(), save_path: p};
    
        let _ = run_app(tx, c);
    });

    let _ = rx.recv().unwrap();

    let client = reqwest::blocking::Client::new();
    let res = client.get("http://localhost:8006/events/123/cancel")
        .send().unwrap()
        .status();
    assert_eq!(res, 401);
}
