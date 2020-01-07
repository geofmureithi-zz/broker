use actix_web::{http::header, middleware, web, HttpServer, HttpResponse, HttpRequest, App, Error, Responder};
use sse_actix_web::{Broadcaster, broadcast};
use serde_derive::{Deserialize, Serialize};
use std::sync::Mutex;
use sled;
use actix_cors::Cors;
use std::collections::HashMap;
use chrono::prelude::*;
use uuid::Uuid;
use serde_json::json;
use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{encode, decode, Header, Validation};
use itertools::Itertools;
use std::iter::FromIterator;

#[derive(Deserialize, Debug)]
pub struct Config {
  port: String,
  pub origin: String,
  expiry: i64,
  secret: String,
}

struct MyData {
    db: sled::Db
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Events (Vec<HashMap<String, HashMap<uuid::Uuid, Vec<Event>>>>);

impl Events {
    fn new() -> Events {
        Events(Vec::new())
    }

    fn add(&mut self, elem: HashMap<String, HashMap<uuid::Uuid, Vec<Event>>>) {
        self.0.push(elem);
    }
}

impl std::iter::FromIterator<HashMap<String, HashMap<uuid::Uuid, Vec<Event>>>> for Events {
    fn from_iter<I: IntoIterator<Item=HashMap<String, HashMap<uuid::Uuid, Vec<Event>>>>>(iter: I) -> Self {
        let mut c = Events::new();

        for i in iter {
            c.add(i);
        }

        c
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Token {
    jwt: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Login {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: uuid::Uuid,
    username: String,
    password: String,
    info: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserForm {
    username: String,
    password: String,
    info: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Event {
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    evt_id: uuid::Uuid,
    event: String,
    timestamp: i64,
    published: bool,
    cancelled: bool,
    data: serde_json::Value,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
struct EventForm {
    id: uuid::Uuid,
    event: String,
    timestamp: i64,
    data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct Path {
    id: String,
}


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

async fn new_client(data: web::Data<MyData>, broad: web::Data<Mutex<Broadcaster>>) -> impl Responder {

    // get origin env var
    let config = envy::from_env::<Config>().unwrap();
    let origin = config.origin;

    // turn iVec(s) to String(s) and make HashMap
    let vals: HashMap<String, String> = data.db.iter().into_iter().filter(|x| {
        let p = x.as_ref().unwrap();
        let k = std::str::from_utf8(&p.0).unwrap().to_owned();
        if !k.contains("_v_") && !k.contains("_u_") {
            return true
        } else {
            return false
        }
    }).map(|x| {
        let p = x.as_ref().unwrap();
        let v = std::str::from_utf8(&p.1).unwrap().to_owned();
        let json : Event = serde_json::from_str(&v).unwrap();
        let data : String = serde_json::to_string(&json.data).unwrap();
        (json.event, data)
    }).collect();

    // create new client for sse with hashmap of initial values
    let rx = broad.lock().unwrap().new_client(vals);

    // create sse endpoint
    HttpResponse::Ok()
        .header("Access-Control-Allow-Origin", origin)
        .header("Set-Cookie", "SameSite=Strict")
        .header("Keep-Alive", "true")
        .header("Access-Control-Allow-Credentials", "true")
        .header("Content-Type", "text/event-stream")
        .no_chunking()
        .streaming(rx)
}

async fn insert(data: web::Data<MyData>, json: web::Json<EventForm>, req: HttpRequest) -> Result<HttpResponse, Error> {

    // get origin env var
    let config = envy::from_env::<Config>().unwrap();
    let secret = config.secret;

    let json_clone = json.clone();

    // verify jwt
    let headers = req.headers();
    for (k, v) in headers {
        if k == "Authorization" {
            let token = v.to_str().unwrap().to_owned();
            let parts = token.split(" ");
            for part in parts {
                if part != "Bearer" {
                     let _ = match decode::<Claims>(&part, secret.as_ref(), &Validation::default()) {
                        Ok(token) => {

                            let user_id_str = token.claims.sub;
                            let user_id = uuid::Uuid::parse_str(&user_id_str).unwrap();

                            let id = Uuid::new_v4();
                            let j = Event{id: id, published: false, cancelled: false, data: json_clone.data, event: json_clone.event, timestamp: json.timestamp, user_id: user_id, evt_id: json.id};
                            let new_value_string = serde_json::to_string(&j).unwrap();
                            let new_value = new_value_string.as_bytes();

                            let versioned = format!("_v_{}", id.to_string());
                        
                            let _ = data.db.compare_and_swap(versioned, None as Option<&[u8]>, Some(new_value.clone())); 
                            let _ = web::block(move || data.db.flush()).await;
                        
                            // return uuid to json response as 200
                            let record = json!({ "id": id.to_string() });
                            return Ok(HttpResponse::Ok().json(record))
                        },
                        Err(err) => match *err.kind() {
                            _ => return Ok(HttpResponse::Unauthorized().json(""))
                        },
                    };
                }
            }
        } 
    }
    Ok(HttpResponse::Unauthorized().json(""))
}

async fn cancel(data: web::Data<MyData>, path: web::Path<Path>, req: HttpRequest) -> Result<HttpResponse, Error> {

    // get origin env var
    let config = envy::from_env::<Config>().unwrap();
    let secret = config.secret;

    // verify jwt
    let headers = req.headers();
    let mut check : i32 = 0;
    for (k, v) in headers {
        if k == "Authorization" {
            let token = v.to_str().unwrap().to_owned();
            let parts = token.split(" ");
            for part in parts {
                if part != "Bearer" {
                    let _ = match decode::<Claims>(&part, secret.as_ref(), &Validation::default()) {
                        Ok(c) => {
                            check = check + 1;
                            c
                        },
                        Err(err) => match *err.kind() {
                            _ => return Ok(HttpResponse::Unauthorized().json(""))
                        },
                    };
                }
            }
        } 
    }
    
    // if no auth header
    if check == 0 {
        return Ok(HttpResponse::Unauthorized().json(""))
    }

    let id = &path.id;
    let versioned = format!("_v_{}", id);

    let g = data.db.get(&versioned.as_bytes()).unwrap().unwrap();
    let v = std::str::from_utf8(&g).unwrap().to_owned();
    let mut json : Event = serde_json::from_str(&v).unwrap();
    let j = json.clone();
    json.cancelled = true;
    let _ = data.db.compare_and_swap(versioned.as_bytes(), Some(serde_json::to_string(&j).unwrap().as_bytes()), Some(serde_json::to_string(&json).unwrap().as_bytes()));
    let _ = web::block(move || { data.db.flush() }).await;
    Ok(HttpResponse::Ok().json(json))
}

async fn user_create(data: web::Data<MyData>, json: web::Json<UserForm>) -> Result<HttpResponse, Error> {

    // turn iVec(s) to String(s) and make HashMap
    let records : HashMap<String, String> = data.db.iter().into_iter().filter(|x| {
        let p = x.as_ref().unwrap();
        let k = std::str::from_utf8(&p.0).unwrap().to_owned();
        let v = std::str::from_utf8(&p.1).unwrap().to_owned();
        if k.contains("_u_") {
            let user : User = serde_json::from_str(&v).unwrap();
            if user.username == json.username {
                return true
            } else {
                return false
            }
        }
        return false
    }).map(|x| {
        let p = x.unwrap();
        let k = std::str::from_utf8(&p.0).unwrap().to_owned();
        let v = std::str::from_utf8(&p.1).unwrap().to_owned();
        (k, v)
    }).collect();

    if records.len() > 0 {
        let j = json!({"error": "username already taken"});
        return Ok(HttpResponse::BadRequest().json(j))
    } else {
        // set as future value
        let uuid = Uuid::new_v4();
        let versioned = format!("user_u_{}", uuid.to_string());
        let hashed = hash(json.clone().password, DEFAULT_COST).unwrap();
        let new_user = User{id: uuid, username: json.clone().username, password: hashed, info: json.clone().info};
        
        let _ = data.db.compare_and_swap(versioned.as_bytes(), None as Option<&[u8]>, Some(serde_json::to_string(&new_user).unwrap().as_bytes())); 
        let _ = web::block(move || { data.db.flush() }).await;
        let j = json!({"id": uuid.to_string()});
        return Ok(HttpResponse::Ok().json(j))
    }
}

async fn login(data: web::Data<MyData>, json: web::Json<Login>) -> Result<HttpResponse, Error> {

    // get origin env var
    let config = envy::from_env::<Config>().unwrap();
    let exp = config.expiry;
    let secret = config.secret;


    // add timestamp
    let now = Utc::now().timestamp();
    let expi = now + exp;
    let expiry = expi as usize;

    // turn iVec(s) to String(s) and make HashMap
    let records : HashMap<String, String> = data.db.iter().into_iter().filter(|x| {
        let p = x.as_ref().unwrap();
        let k = std::str::from_utf8(&p.0).unwrap().to_owned();
        let search = format!("{}_u_", json.username);
        if k.contains(&search) {
            return true;
        } else {
            return false;
        }
    }).map(|x| {
        let p = x.unwrap();
        let k = std::str::from_utf8(&p.0).unwrap().to_owned();
        let v = std::str::from_utf8(&p.1).unwrap().to_owned();
        (k, v)
    }).collect();

    for (_k, v) in records {
        let user : User = serde_json::from_str(&v).unwrap();
        if user.username == json.username && verify(json.clone().password, &user.password).unwrap() {
            let my_claims = Claims{company: "".to_owned(), sub: user.id.to_string(), exp: expiry};
            let token = encode(&Header::default(), &my_claims, secret.as_ref()).unwrap();
            return Ok(HttpResponse::Ok().json(Token{jwt: token}))
        } else {
            return Ok(HttpResponse::Unauthorized().json(""))
        }
    }

    Ok(HttpResponse::Unauthorized().json(""))
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
    let tree_actix = tree.clone();
    let events_cloned = events.clone();

    // create event watcher
    let x = std::thread::spawn(move || {
        loop {
            let vals : HashMap<String, Event> = tree_cloned.iter().into_iter().filter(|x| {
                let p = x.as_ref().unwrap();
                let k = std::str::from_utf8(&p.0).unwrap().to_owned();
                let v = std::str::from_utf8(&p.1).unwrap().to_owned();
                if k.contains("_v_") {
                    let json : Event = serde_json::from_str(&v).unwrap();
                    let now = Utc::now().timestamp();
                    if json.timestamp <= now && !json.published && !json.cancelled {
                        return true
                    } else {
                        return false
                    }
                } else {
                    return false
                }
            }).map(|x| {
                let p = x.as_ref().unwrap();
                let k = std::str::from_utf8(&p.0).unwrap().to_owned();
                let v = std::str::from_utf8(&p.1).unwrap().to_owned();
                let json : Event = serde_json::from_str(&v).unwrap();
                let json_cloned = json.clone();
                (k, json_cloned)
            }).collect();

            let vals_cloned = vals.clone();

            for (k, v) in vals {
                let old_json = v.clone();
                let old_json_clone = old_json.clone();
                let mut new_json = v.clone();
                new_json.published = true;
                let _ = tree_cloned.compare_and_swap(k, Some(serde_json::to_string(&old_json_clone).unwrap().as_bytes()), Some(serde_json::to_string(&new_json).unwrap().as_bytes())); 
                let _ = tree_cloned.flush();
            }

            let eventz : Vec<Event> = vals_cloned.iter().map(|(_x, y)| {
                y.clone()
            }).collect();

            let groups : Events = eventz.into_iter()
                .group_by(|evt| evt.event)
                .into_iter()
                .map(|(event, group)| {
                    // let evts : Vec<Event> = group.into_iter().map(|x| { x.clone() }).collect();
                    let groups = group.into_iter().group_by(|evt| evt.evt_id);
                    let mut x = HashMap::new();
                    x.insert(event, groups);
                    x
                })
                .collect();

        }
            

            // for (k, v) in grouped {
            //     broadcast(k, serde_json::to_string(&v).unwrap(), events_cloned.clone());
            // }
    });
    x.thread();

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
            .data(MyData{ db: tree_actix.clone() })
            .route("/insert", web::post().to(insert))
            .route("/events", web::get().to(new_client))
            .route("/events/{id}/cancel", web::get().to(cancel))
            .route("/users", web::post().to(user_create))
            .route("/login", web::post().to(login))
    })
    .bind(ip).unwrap()
    .run()
    .await
}
