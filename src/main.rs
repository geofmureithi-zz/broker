use actix_web::{web, HttpServer, HttpResponse, App, Error, Responder};
use sse_actix_web::{Broadcaster, broadcast};
use serde_derive::{Deserialize, Serialize};
use std::sync::Mutex;
use sled;

pub struct MyData {
    db: sled::Db
}

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    user: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JSON {
    event: String,
    data: Data,
}

async fn new_client(data: web::Data<MyData>, broadcaster: web::Data<Mutex<Broadcaster>>) -> impl Responder {

    let user_buffer = data.db.get(b"user").unwrap().unwrap();
    
    let user = std::str::from_utf8(&user_buffer).unwrap();

    let rx = broadcaster.lock().unwrap().new_client(&"user", user);

    HttpResponse::Ok()
        .header("content-type", "text/event-stream")
        .header("Access-Control-Allow-Origin",  "*")
        .header("Access-Control-Allow-Credentials", "true")
        .no_chunking()
        .streaming(rx)
}

async fn insert(data: web::Data<MyData>, broad: web::Data<Mutex<Broadcaster>>, json: web::Json<JSON>) -> Result<HttpResponse, Error> {

    let user_string = serde_json::to_string(&json.0.data).unwrap();

    let user_buffer = data.db.get(b"user").unwrap().unwrap();
    
    let user = std::str::from_utf8(&user_buffer).unwrap();
    
    let new_user = user_string.as_bytes();
    
    let old_user = user.clone().as_bytes();

    let _ = data.db.compare_and_swap(b"user", Some(old_user.clone()), Some(new_user.clone()));

    let _ = web::block(move || data.db.flush()).await;

    broadcast(json.0.event, serde_json::to_string(&json.0.data).unwrap(), broad.clone()).await;

    Ok(HttpResponse::Ok().json(json.0.data))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let tree = sled::open("./tmp/data").unwrap();
    let tree_clone = tree.clone();
    let _ = tree.compare_and_swap(b"user", None as Option<&[u8]>, Some(b""));
    let _ = web::block(move || tree.flush()).await;

    let data = Broadcaster::create();

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .data(MyData{ db: tree_clone.clone()})
            .app_data(web::JsonConfig::default())
            .route("/insert", web::post().to(insert))
            .route("/events", web::get().to(new_client))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
