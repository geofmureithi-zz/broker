use actix_web::{http::header, middleware, web, HttpServer, HttpResponse, App, Error, Responder};
use sse_actix_web::{Broadcaster, broadcast};
use serde_derive::{Deserialize, Serialize};
use std::sync::Mutex;
use sled;
use actix_cors::Cors;
use std::collections::HashMap;
use chrono::prelude::*;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
struct Config {
  port: String
}

struct MyData {
    db: sled::Db
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct JSON {
    event: String,
    timestamp: i64,
    published: bool,
    data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct Path {
    record: String
}

async fn collection(data: web::Data<MyData>, path: web::Path<Path>) -> Result<HttpResponse, Error> {
    
    // get iter to loop through all keys in db
    let iter = data.db.iter();

    // turn iVec(s) to String(s) and make HashMap
    let records: HashMap<String, serde_json::value::Value> = iter.into_iter().filter(|x| {
        let p = x.as_ref().unwrap();
        let k = std::str::from_utf8(&p.0).unwrap().to_owned();
        if k.contains(&path.record) {
            return true;
        } else {
            return false;
        }
    }).map(|x| {
        let p = x.unwrap();
        let k = std::str::from_utf8(&p.0).unwrap().to_owned();
        let v = std::str::from_utf8(&p.1).unwrap().to_owned();
        let j : serde_json::Value = serde_json::from_str(&v).unwrap_or_default();
        (k, j)
    }).collect();

    // return data to json response as 200
    Ok(HttpResponse::Ok().json(records))
}

async fn new_client(data: web::Data<MyData>, broad: web::Data<Mutex<Broadcaster>>) -> impl Responder {

    // turn iVec(s) to String(s) and make HashMap
    let vals: HashMap<String, String> = data.db.iter().into_iter().filter(|x| {
        let p = x.as_ref().unwrap();
        let k = std::str::from_utf8(&p.0).unwrap().to_owned();
        if !k.contains("_v_") {
            return true
        } else {
            return false
        }
    }).map(|x| {
        let p = x.as_ref().unwrap();
        let v = std::str::from_utf8(&p.1).unwrap().to_owned();
        let json : JSON = serde_json::from_str(&v).unwrap();
        let data : String = serde_json::to_string(&json.data).unwrap();
        (json.event, data)
    }).collect();

    // create new client for sse with hashmap of initial values
    let rx = broad.lock().unwrap().new_client(vals);

    // create sse endpoint
    HttpResponse::Ok()
        .header("content-type", "text/event-stream")
        .no_chunking()
        .streaming(rx)
}

async fn insert(data: web::Data<MyData>, json: web::Json<JSON>) -> Result<HttpResponse, Error> {

    // get new value from json
    let new_value_string = serde_json::to_string(&json.0).unwrap();
    let new_value = new_value_string.as_bytes();

    // set as future value
    let uuid = Uuid::new_v4();
    let versioned = format!("{}_v_{}", json.0.event, uuid.to_string());
    let _ = data.db.compare_and_swap(versioned.clone(), None as Option<&[u8]>, Some(new_value.clone())); 
    let _ = web::block(move || data.db.flush()).await;

    // return data to json response as 200
    Ok(HttpResponse::Ok().json(json.0.data))
}

pub async fn broker_run(origin: String) -> std::result::Result<(), std::io::Error> {
    // set actix web env vars
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    // get port env var
    let config = envy::from_env::<Config>().unwrap();
    let ip = format!("0.0.0.0:{}", config.port);
  
    // setup db and sse
    let tree = sled::open("./tmp/data").unwrap();
    let events = Broadcaster::create();
    let tree_cloned = tree.clone();
    let events_cloned = events.clone();

    // create event watcher
    let x = std::thread::spawn(move || {
        loop {
            let vals : HashMap<String, JSON> = tree_cloned.iter().into_iter().filter(|x| {
                let p = x.as_ref().unwrap();
                let v = std::str::from_utf8(&p.1).unwrap().to_owned();
                let json : JSON = serde_json::from_str(&v).unwrap();
                let now = Utc::now().timestamp();
                if json.timestamp <= now && !json.published {
                    return true
                } else {
                    return false
                }
            }).map(|x| {
                let p = x.as_ref().unwrap();
                let k = std::str::from_utf8(&p.0).unwrap().to_owned();
                let v = std::str::from_utf8(&p.1).unwrap().to_owned();
                let json : JSON = serde_json::from_str(&v).unwrap();
                let json_cloned = json.clone();
                (k, json_cloned)
            }).collect();

            for (k, v) in vals {
                let old_json = v.clone();
                let old_json_clone = old_json.clone();
                let mut new_json = v.clone();
                new_json.published = true;
                let _ = tree_cloned.compare_and_swap(old_json.event.as_bytes(), None as Option<&[u8]>, Some(b""));
                let old_json_og = tree_cloned.get(old_json.event).unwrap().unwrap();
                let old_value = std::str::from_utf8(&old_json_og).unwrap().to_owned();
                let _ = tree_cloned.compare_and_swap(old_json_clone.event.as_bytes(), Some(old_value.as_bytes()), Some(serde_json::to_string(&new_json).unwrap().as_bytes()));
                let _ = tree_cloned.compare_and_swap(k, Some(serde_json::to_string(&old_json_clone).unwrap().as_bytes()), Some(serde_json::to_string(&new_json).unwrap().as_bytes())); 
                let _ = tree_cloned.flush();
                broadcast(new_json.event, serde_json::to_string(&new_json.data).unwrap(), events_cloned.clone());
            }
       }  
    });
    x.thread();

    // create actix web server with CORS, data, and routes - handle wildcard origins
    if origin == "*" {
        HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::default())
                .wrap(
                    Cors::new()
                        .send_wildcard()
                        .allowed_methods(vec!["GET", "POST"])
                        .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
                        .max_age(3600)
                        .finish()
                )
                .app_data(events.clone())
                .app_data(web::JsonConfig::default())
                .data(MyData{ db: tree.clone() })
                .route("/insert", web::post().to(insert))
                .route("/events", web::get().to(new_client))
                .route("/collection/{record}", web::get().to(collection))
        })
        .bind(ip).unwrap()
        .run()
        .await
    } else {
        HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::default())
                .wrap(
                    Cors::new()
                        .allowed_origin(&origin)
                        .allowed_methods(vec!["GET", "POST"])
                        .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
                        .max_age(3600)
                        .finish()
                )
                .app_data(events.clone())
                .app_data(web::JsonConfig::default())
                .data(MyData{ db: tree.clone() })
                .route("/insert", web::post().to(insert))
                .route("/events", web::get().to(new_client))
                .route("/collection/{record}", web::get().to(collection))
        })
        .bind(ip).unwrap()
        .run()
        .await
    }
}
