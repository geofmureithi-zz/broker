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

pub struct MyData {
    db: sled::Db
}

#[derive(Debug, Serialize, Deserialize)]
struct JSON {
    event: String,
    data: serde_json::Value,
}

async fn new_client(data: web::Data<MyData>, broad: web::Data<Mutex<Broadcaster>>) -> impl Responder {

    let iter = data.db.iter();

    let vals: HashMap<String, String> = iter.into_iter().map(|x| {
        let p = x.unwrap();
        let k = std::str::from_utf8(&p.0).unwrap().to_owned();
        let v = std::str::from_utf8(&p.1).unwrap().to_owned();
        (k, v)
    }).collect();

    let rx = broad.lock().unwrap().new_client(vals);

    HttpResponse::Ok()
        .header("content-type", "text/event-stream")
        .no_chunking()
        .streaming(rx)
}

async fn insert(data: web::Data<MyData>, broad: web::Data<Mutex<Broadcaster>>, json: web::Json<JSON>) -> Result<HttpResponse, Error> {

    let data_cloned = data.clone();

    let _ = data_cloned.db.compare_and_swap(json.0.event.clone(), None as Option<&[u8]>, Some(b""));
    
    let _ = web::block(move || data_cloned.db.flush()).await;

    let user_string = serde_json::to_string(&json.0.data).unwrap();

    let user_buffer = data.db.get(json.0.event.clone()).unwrap().unwrap();
    
    let user = std::str::from_utf8(&user_buffer).unwrap();
    
    let new_user = user_string.as_bytes();
    
    let old_user = user.clone().as_bytes();

    let _ = data.db.compare_and_swap(json.0.event.clone(), Some(old_user.clone()), Some(new_user.clone()));

    let _ = web::block(move || data.db.flush()).await;

    broadcast(json.0.event.clone(), serde_json::to_string(&json.0.data).unwrap(), broad.clone()).await;

    Ok(HttpResponse::Ok().json(json.0.data))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();
    let config = envy::from_env::<Config>().unwrap();
    let ip = format!("0.0.0.0:{}", config.port);
  
    let tree = sled::open("./tmp/data").unwrap();
    let tree_clone = tree.clone();

    let data = Broadcaster::create();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::new()
                    .send_wildcard()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
                    .max_age(3600)
                    .finish(),
            )
            .app_data(data.clone())
            .data(MyData{ db: tree_clone.clone()})
            .app_data(web::JsonConfig::default())
            .route("/insert", web::post().to(insert))
            .route("/events", web::get().to(new_client))
    })
    .bind(ip)?
    .run()
    .await
}
