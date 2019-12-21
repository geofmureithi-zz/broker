use actix_web::{web, HttpServer, HttpResponse, App, Error};
use sse_actix_web::{new_client, Broadcaster, broadcast};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JSON {
    event: String,
    data: Data,

}

async fn send(broad: web::Data<std::sync::Mutex<Broadcaster>>, json: web::Json<JSON>) -> Result<HttpResponse, Error> {
    let user_string = serde_json::to_string(&json.0.data).unwrap();
    broadcast(json.0.event, user_string, broad.clone()).await;
    Ok(HttpResponse::Ok().json(json.0.data))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let data = Broadcaster::create();

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .app_data(web::JsonConfig::default())
            .route("/name", web::post().to(send))
            .route("/events", web::get().to(new_client))
    })
    .bind("0.0.0.0:8080")?
    .start()
    .await
}
