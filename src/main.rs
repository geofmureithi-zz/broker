use actix_web::web::Path;
use actix_web::{web, HttpServer, HttpResponse, App, Responder, Error};
use sse_actix_web::{new_client, Broadcaster, broadcast};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct MyUser {
    name: String,
}

async fn broad (
    msg: Path<String>,
    broadcaster: web::Data<std::sync::Mutex<Broadcaster>>,
) -> impl Responder {
    broadcaster.lock().unwrap().send("message", &msg.into_inner());

    HttpResponse::Ok().body("msg sent")
}


async fn msg(item: web::Json<MyUser>, broad: web::Data<std::sync::Mutex<Broadcaster>>) -> Result<HttpResponse, Error> {
    let user_string = serde_json::to_string(&item.0).unwrap();
    broadcast("message".to_owned(), user_string, broad).await;
    Ok(HttpResponse::Ok().json(item.0))
}

async fn index() -> impl Responder {

    let content = r#"<html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <meta http-equiv="X-UA-Compatible" content="ie=edge">
        <title>Server-sent events</title>
        <style>
            p {
                margin-top: 0.5em;
                margin-bottom: 0.5em;
            }
        </style>
    </head>
    <body>
        <div id="root"></div>
        <script>
            let root = document.getElementById("root");
            let events = new EventSource("/events");
            let data = document.createElement("p");
            root.appendChild(data);
            events.onmessage = (event) => {
                if (event.data != "ping" && event.data != "connected") {
                    data.innerText = event.data;
                }
            }
        </script>
    </body>
    </html>"#;

    HttpResponse::Ok()
        .header("content-type", "text/html")
        .body(content)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let data = Broadcaster::create();

    HttpServer::new(move || {
        App::new()
            .register_data(data.clone())
            .data(web::JsonConfig::default())
            .route("/", web::get().to(index))
            .route("/msg", web::post().to(msg))
            .route("/broadcast/{msg}", web::get().to(broad))
            .route("/events", web::get().to(new_client))
    })
    .bind("0.0.0.0:3000")?
    .start()
    .await
}
