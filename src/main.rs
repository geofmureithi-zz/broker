use std::io::{self};
use actix_web::{web, HttpServer, HttpResponse, App, Responder};
use sse_actix_web::{new_client, Broadcaster, broadcast};
use sled::Db;
use actix_files::NamedFile;

struct MyData {
    db: sled::Db,
}

async fn download(data: web::Data<MyData>, broad: web::Data<std::sync::Mutex<Broadcaster>>) -> io::Result<NamedFile> {
    let counter_buffer = data.db.get(b"counter").unwrap().unwrap();
    
    let counter = std::str::from_utf8(&counter_buffer).unwrap();
    let counter_int = counter.clone().parse::<i32>().unwrap();
    let new_counter_int = counter_int + 1;
    let new_counter_string = new_counter_int.clone().to_string();
    let new_counter = new_counter_string.as_bytes();
    let old_counter = counter.clone().as_bytes();

    let _ = data.db.compare_and_swap(b"counter", Some(old_counter.clone()), Some(new_counter.clone()));

    let _ = web::block(move || data.db.flush()).await;

    broadcast(counter, broad).await;

    let f = web::block(|| std::fs::File::create("test.pdf")).await.unwrap();
    
    NamedFile::from_file(f, "test.pdf")
}

async fn index() -> impl Responder {
    let content = include_str!("index.html");

    HttpResponse::Ok()
        .header("content-type", "text/html")
        .body(content)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let data = Broadcaster::create();
    let tree = Db::open("./tmp/data").unwrap();
    tree.insert(b"counter", b"0").unwrap();

    HttpServer::new(move || {
        App::new()
            .register_data(data.clone())
            .data(MyData{ db: tree.clone() })
            .route("/", web::get().to(index))
            .route("/events", web::get().to(new_client))
            .route("/download", web::get().to(download))
    })
    .bind("0.0.0.0:3000")?
    .start()
    .await
}
