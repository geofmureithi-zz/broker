use actix_web::{http::header, middleware, web, HttpServer, HttpResponse, App, Error, Responder};
use sse_actix_web::{Broadcaster, broadcast};
use serde_derive::{Deserialize, Serialize};
use std::sync::Mutex;
use sled;
use actix_cors::Cors;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct Config {
  port: String
}

struct MyData {
    db: sled::Db
}

#[derive(Debug, Serialize, Deserialize)]
struct JSON {
    event: String,
    data: serde_json::Value,
}

async fn new_client(data: web::Data<MyData>, broad: web::Data<Mutex<Broadcaster>>) -> impl Responder {

    // get iter to loop through all keys in db
    let iter = data.db.iter();

    // turn iVec(s) to String(s) and make HashMap to pass to sse
    let vals: HashMap<String, String> = iter.into_iter().map(|x| {
        let p = x.unwrap();
        let k = std::str::from_utf8(&p.0).unwrap().to_owned();
        let v = std::str::from_utf8(&p.1).unwrap().to_owned();
        (k, v)
    }).collect();
    let rx = broad.lock().unwrap().new_client(vals);

    // create sse endpoint
    HttpResponse::Ok()
        .header("content-type", "text/event-stream")
        .no_chunking()
        .streaming(rx)
}

async fn insert(data: web::Data<MyData>, broad: web::Data<Mutex<Broadcaster>>, json: web::Json<JSON>) -> Result<HttpResponse, Error> {

    // write blank value if no value for key exists so that get later doesn't crash
    let data_cloned = data.clone();
    let _ = data_cloned.db.compare_and_swap(json.0.event.clone(), None as Option<&[u8]>, Some(b""));
    let _ = web::block(move || data_cloned.db.flush()).await;

    // get old value from db and new value from json
    let new_value_string = serde_json::to_string(&json.0.data).unwrap();
    let new_value = new_value_string.as_bytes();
    let old_value_buffer = data.db.get(json.0.event.clone()).unwrap().unwrap();
    let old_value_string = std::str::from_utf8(&old_value_buffer).unwrap();
    let old_value = old_value_string.clone().as_bytes();

    // write old and new values to db
    let _ = data.db.compare_and_swap(json.0.event.clone(), Some(old_value.clone()), Some(new_value.clone()));
    let _ = web::block(move || data.db.flush()).await;

    // write new event and data to sse
    broadcast(json.0.event.clone(), serde_json::to_string(&json.0.data).unwrap(), broad.clone()).await;

    // return data to json response as 200
    Ok(HttpResponse::Ok().json(json.0.data))
}

pub async fn sse_start(origin: String) -> std::result::Result<(), std::io::Error> {
    // set actix web env vars
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    // get port env var
    let config = envy::from_env::<Config>().unwrap();
    let ip = format!("0.0.0.0:{}", config.port);
  
    // setup db and sse
    let tree = sled::open("./tmp/data").unwrap();
    let data = Broadcaster::create();

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
                .app_data(data.clone())
                .app_data(web::JsonConfig::default())
                .data(MyData{ db: tree.clone() })
                .route("/insert", web::post().to(insert))
                .route("/events", web::get().to(new_client))
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
                .app_data(data.clone())
                .app_data(web::JsonConfig::default())
                .data(MyData{ db: tree.clone() })
                .route("/insert", web::post().to(insert))
                .route("/events", web::get().to(new_client))
        })
        .bind(ip).unwrap()
        .run()
        .await
    }
}
