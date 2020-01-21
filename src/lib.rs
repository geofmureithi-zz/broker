// use serde_derive::{Deserialize, Serialize};
// use sled;
// use std::collections::HashMap;
// use chrono::prelude::*;
// use uuid::Uuid;
// use serde_json::json;
// use bcrypt::{DEFAULT_COST, hash, verify};
// use jsonwebtoken::{encode, decode, Header, Validation};
// use portal::Filter;

// #[derive(Deserialize, Debug, Clone)]
// pub struct Config {
//   pub port: String,
//   pub origin: String,
//   pub expiry: i64,
//   pub secret: String,
//   pub save_path: String,
// }

// #[derive(Deserialize, Debug, Clone)]
// pub struct Cfg {
//   pub save_path: String,
// }

// #[derive(Debug, Clone)]
// struct MyData {
//     db: sled::Db,
//     config: Config,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Token {
//     pub jwt: String
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// struct Login {
//     username: String,
//     password: String,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// struct User {
//     id: uuid::Uuid,
//     username: String,
//     password: String,
//     collection_id: uuid::Uuid,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// struct UserForm {
//     username: String,
//     password: String,
//     collection_id: uuid::Uuid,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct UserCollection {
//     pub info: Vec<Event>,
//     pub events: Vec<Event>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Collection {
//     pub events: Vec<Event>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Record {
//     pub event: Event,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Event {
//     pub id: uuid::Uuid,
//     pub user_id: uuid::Uuid,
//     pub collection_id: uuid::Uuid,
//     pub event: String,
//     pub timestamp: i64,
//     pub published: bool,
//     pub cancelled: bool,
//     pub data: serde_json::Value,
// }


// #[derive(Debug, Serialize, Deserialize, Clone)]
// struct EventForm {
//     collection_id: uuid::Uuid,
//     event: String,
//     timestamp: i64,
//     data: serde_json::Value,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct Path {
//     id: String,
// }


// #[derive(Debug, Serialize, Deserialize)]
// struct Claims {
//     sub: String,
//     company: String,
//     exp: usize,
// }

// fn auth(data: web::Data<MyData>, req: HttpRequest) -> (bool, Claims) {
//     let headers = req.headers();
//     for (k, v) in headers {
//         if k == "Authorization" {
//             let token = v.to_str().unwrap().to_owned();
//             let parts = token.split(" ");
//             for part in parts {
//                 if part != "Bearer" {
//                     let _ = match decode::<Claims>(&part, data.config.secret.as_ref(), &Validation::default()) {
//                         Ok(c) => {
//                             return (true, c.claims)
//                         },
//                         Err(_e) => {
//                             return (false, Claims{company: "".to_owned(), exp: 0, sub: "".to_owned()})
//                         }
//                     };
//                 }
//             }
//         }
//     }
//     (false, Claims{company: "".to_owned(), exp: 0, sub: "".to_owned()})
// }

// async fn user_collection(data: web::Data<MyData>, req: HttpRequest) -> Result<HttpResponse, Error> {

//     let (check, token) = auth(data.clone(), req.clone());
//     if !check {
//         return Ok(HttpResponse::Unauthorized().json(""))
//     }

//     let versioned = format!("_u_{}", token.sub);
//     let g = data.db.get(&versioned.as_bytes()).unwrap().unwrap();
//     let v = std::str::from_utf8(&g).unwrap().to_owned();
//     let user : User = serde_json::from_str(&v).unwrap();

//     // turn iVec(s) to String(s) and make HashMap
//     let mut info: Vec<Event> = data.db.iter().into_iter().filter(|x| {
//         let p = x.as_ref().unwrap();
//         let k = std::str::from_utf8(&p.0).unwrap().to_owned();
//         if k.contains(&"_v_") {
//             let v = std::str::from_utf8(&p.1).unwrap().to_owned();
//             let j : Event = serde_json::from_str(&v).unwrap();
//             if j.collection_id.to_string() == user.collection_id.to_string() {
//                 return true
//             } else {
//                 return false
//             }
//         } else {
//             return false
//         }
//     }).map(|x| {
//         let p = x.unwrap();
//         let v = std::str::from_utf8(&p.1).unwrap().to_owned();
//         let j : Event = serde_json::from_str(&v).unwrap();
//         j
//     }).collect();

//     info.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

//     // turn iVec(s) to String(s) and make HashMap
//     let mut owned: Vec<Event> = data.db.iter().into_iter().filter(|x| {
//         let p = x.as_ref().unwrap();
//         let k = std::str::from_utf8(&p.0).unwrap().to_owned();
//         if k.contains(&"_v_") {
//             let v = std::str::from_utf8(&p.1).unwrap().to_owned();
//             let j : Event = serde_json::from_str(&v).unwrap();
//             if j.user_id.to_string() == user.id.to_string() {
//                 return true
//             } else {
//                 return false
//             }
//         } else {
//             return false
//         }
//     }).map(|x| {
//         let p = x.unwrap();
//         let v = std::str::from_utf8(&p.1).unwrap().to_owned();
//         let j : Event = serde_json::from_str(&v).unwrap();
//         j
//     }).collect();

//     owned.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

//     // return data to json response as 200
//     let j = UserCollection{info: info, events: owned};
//     Ok(HttpResponse::Ok().json(j))
// }


// async fn collection(data: web::Data<MyData>, path: web::Path<Path>, req: HttpRequest) -> Result<HttpResponse, Error> {

//     let (check, _token) = auth(data.clone(), req.clone());
//     if !check {
//         return Ok(HttpResponse::Unauthorized().json(""))
//     }

//     // turn iVec(s) to String(s) and make HashMap
//     let mut records: Vec<Event> = data.db.iter().into_iter().filter(|x| {
//         let p = x.as_ref().unwrap();
//         let k = std::str::from_utf8(&p.0).unwrap().to_owned();
//         if k.contains(&"_v_") {
//             let v = std::str::from_utf8(&p.1).unwrap().to_owned();
//             let j : Event = serde_json::from_str(&v).unwrap();
//             if j.collection_id.to_string() == path.id {
//                 return true
//             } else {
//                 return false
//             }
//         } else {
//             return false
//         }
//     }).map(|x| {
//         let p = x.unwrap();
//         let v = std::str::from_utf8(&p.1).unwrap().to_owned();
//         let j : Event = serde_json::from_str(&v).unwrap();
//         j
//     }).collect();

//     records.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

//     // return data to json response as 200
//     let j = Collection{events: records};
//     Ok(HttpResponse::Ok().json(j))
// }

// #[cfg_attr(tarpaulin, skip)]
// async fn new_client(data: web::Data<MyData>, broad: web::Data<Mutex<Broadcaster>>) -> impl Responder {

//     // turn iVec(s) to String(s) and make HashMap
//     let vals: HashMap<String, String> = data.db.iter().into_iter().filter(|x| {
//         let p = x.as_ref().unwrap();
//         let k = std::str::from_utf8(&p.0).unwrap().to_owned();
//         if !k.contains("_v_") && !k.contains("_u_") {
//             return true
//         } else {
//             return false
//         }
//     }).map(|x| {
//         let p = x.as_ref().unwrap();
//         let v = std::str::from_utf8(&p.1).unwrap().to_owned();
//         let json : Event = serde_json::from_str(&v).unwrap();
//         let data : String = serde_json::to_string(&json.data).unwrap();
//         (json.event, data)
//     }).collect();

//     // create new client for sse with hashmap of initial values
//     let rx = broad.lock().unwrap().new_client(vals);
//     let origin = &*data.config.origin;

//     // create sse endpoint
//     HttpResponse::Ok()
//         .header("Access-Control-Allow-Origin", origin)
//         .header("Set-Cookie", "SameSite=Strict")
//         .header("Keep-Alive", "true")
//         .header("Access-Control-Allow-Credentials", "true")
//         .header("Content-Type", "text/event-stream")
//         .no_chunking()
//         .streaming(rx)
// }

// async fn insert(data: web::Data<MyData>, json: web::Json<EventForm>, req: HttpRequest) -> Result<HttpResponse, Error> {
 
//     let (check, token) = auth(data.clone(), req.clone());
//     if !check {
//         return Ok(HttpResponse::Unauthorized().json(""))
//     }

//     let user_id_str = token.sub;
//     let user_id = uuid::Uuid::parse_str(&user_id_str).unwrap();

//     let json_cloned = json.clone();

//     let id = Uuid::new_v4();
//     let j = Event{id: id, published: false, cancelled: false, data: json_cloned.data, event: json_cloned.event, timestamp: json.timestamp, user_id: user_id, collection_id: json.collection_id};
//     let new_value_string = serde_json::to_string(&j).unwrap();
//     let new_value = new_value_string.as_bytes();
//     let versioned = format!("_v_{}", id.to_string());

//     let _ = data.db.compare_and_swap(versioned, None as Option<&[u8]>, Some(new_value.clone())); 
//     let _ = web::block(move || data.db.flush()).await;

//     // return uuid to json response as 200
//     let record = Record{ event: j};
//     Ok(HttpResponse::Ok().json(record))
// }

// async fn cancel(data: web::Data<MyData>, path: web::Path<Path>, req: HttpRequest) -> Result<HttpResponse, Error> {

//     let (check, _token) = auth(data.clone(), req.clone());
//     if !check {
//         return Ok(HttpResponse::Unauthorized().json(""))
//     }

//     let id = &path.id;
//     let versioned = format!("_v_{}", id);

//     let g = data.db.get(&versioned.as_bytes()).unwrap().unwrap();
//     let v = std::str::from_utf8(&g).unwrap().to_owned();
//     let mut json : Event = serde_json::from_str(&v).unwrap();
//     let j = json.clone();
//     json.cancelled = true;
//     let _ = data.db.compare_and_swap(versioned.as_bytes(), Some(serde_json::to_string(&j).unwrap().as_bytes()), Some(serde_json::to_string(&json).unwrap().as_bytes()));
//     let _ = web::block(move || { data.db.flush() }).await;
//     let record = Record{ event: json};
//     Ok(HttpResponse::Ok().json(record))
// }

// async fn user_create(data: web::Data<MyData>, json: web::Json<UserForm>) -> Result<HttpResponse, Error> {

//     // turn iVec(s) to String(s) and make HashMap
//     let records : HashMap<String, String> = data.db.iter().into_iter().filter(|x| {
//         let p = x.as_ref().unwrap();
//         let k = std::str::from_utf8(&p.0).unwrap().to_owned();
//         let v = std::str::from_utf8(&p.1).unwrap().to_owned();
//         if k.contains("_u_") {
//             let user : User = serde_json::from_str(&v).unwrap();
//             if user.username == json.username {
//                 return true
//             } else {
//                 return false
//             }
//         }
//         return false
//     }).map(|x| {
//         let p = x.unwrap();
//         let k = std::str::from_utf8(&p.0).unwrap().to_owned();
//         let v = std::str::from_utf8(&p.1).unwrap().to_owned();
//         (k, v)
//     }).collect();

//     if records.len() > 0 {
//         let j = json!({"error": "username already taken"});
//         return Ok(HttpResponse::BadRequest().json(j))
//     } else {
//         // set as future value
//         let uuid = Uuid::new_v4();
//         let versioned = format!("_u_{}", uuid.to_string());
//         let hashed = hash(json.clone().password, DEFAULT_COST).unwrap();
//         let new_user = User{id: uuid, username: json.clone().username, password: hashed, collection_id: json.collection_id };
        
//         let _ = data.db.compare_and_swap(versioned.as_bytes(), None as Option<&[u8]>, Some(serde_json::to_string(&new_user).unwrap().as_bytes())); 
//         let _ = web::block(move || { data.db.flush() }).await;
//         let j = json!({"id": uuid.to_string()});
//         return Ok(HttpResponse::Ok().json(j))
//     }
// }

// async fn login(data: web::Data<MyData>, json: web::Json<Login>) -> Result<HttpResponse, Error> {
//     // add timestamp
//     let now = Utc::now().timestamp();
//     let expi = now + data.config.expiry;
//     let expiry = expi as usize;

//     // turn iVec(s) to String(s) and make HashMap
//     let records : HashMap<String, String> = data.db.iter().into_iter().filter(|x| {
//         let p = x.as_ref().unwrap();
//         let k = std::str::from_utf8(&p.0).unwrap().to_owned();
//         if k.contains(&"_u_") {
//             let v = std::str::from_utf8(&p.1).unwrap().to_owned();
//             let user : User = serde_json::from_str(&v).unwrap();
//             if user.username == json.username {
//                 return true
//             } else {
//                 return false
//             }
//         } else {
//             return false
//         }
//     }).map(|x| {
//         let p = x.unwrap();
//         let k = std::str::from_utf8(&p.0).unwrap().to_owned();
//         let v = std::str::from_utf8(&p.1).unwrap().to_owned();
//         (k, v)
//     }).collect();

//     for (_k, v) in records {
//         let user : User = serde_json::from_str(&v).unwrap();
//         let json_cloned = json.clone();
//         let verified = verify(json_cloned.password, &user.password).unwrap();
//         if verified {
//             let my_claims = Claims{company: "".to_owned(), sub: user.id.to_string(), exp: expiry};
//             let token = encode(&Header::default(), &my_claims, data.config.secret.as_ref()).unwrap();
//             return Ok(HttpResponse::Ok().json(Token{jwt: token}))
//         } else {
//             return Ok(HttpResponse::Unauthorized().json(""))
//         }
//     }

//     Ok(HttpResponse::Unauthorized().json(""))
// }

// // #[cfg_attr(tarpaulin, skip)]
// // pub async fn broker_run() -> Result<(), std::io::Error> {
// //     let mut port = "8080".to_owned();
// //     let mut expiry : i64 = 3600;
// //     let mut origin = "http://localhost:3000".to_owned();
// //     let mut secret = "secret".to_owned();
// //     let _ : Vec<String> = go_flag::parse(|flags| {
// //         flags.add_flag("port", &mut port);
// //         flags.add_flag("expiry", &mut expiry);
// //         flags.add_flag("origin", &mut origin);
// //         flags.add_flag("secret", &mut secret);
// //     });

// //     std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");


// //     let cfg = envy::from_env::<Cfg>().unwrap();

// //     let config = Config{port: port, origin: origin, secret: secret, save_path: cfg.save_path, expiry: expiry};
// //     server_create(config).await
// // }

// #[cfg_attr(tarpaulin, skip)]
// pub fn server_create(config: Config) -> actix_server::Server {
//     let ip = format!("0.0.0.0:{}", config.port);

//     // setup db and sse
//     let tree = sled::open(&config.save_path).unwrap();

//     let tree_cloned = tree.clone();
//     let tree_actix = tree.clone();
//     // let events_cloned = events.clone();
//     let config_cloned = config.clone();

//     // // create event watcher
//     // let x = std::thread::spawn(move || {
//     //     loop {
//     //         let vals : HashMap<String, Event> = tree_cloned.iter().into_iter().filter(|x| {
//     //             let p = x.as_ref().unwrap();
//     //             let k = std::str::from_utf8(&p.0).unwrap().to_owned();
//     //             let v = std::str::from_utf8(&p.1).unwrap().to_owned();
//     //             if k.contains("_v_") {
//     //                 let json : Event = serde_json::from_str(&v).unwrap();
//     //                 let now = Utc::now().timestamp();
//     //                 if json.timestamp <= now && !json.published && !json.cancelled {
//     //                     return true
//     //                 } else {
//     //                     return false
//     //                 }
//     //             } else {
//     //                 return false
//     //             }
//     //         }).map(|x| {
//     //             let p = x.as_ref().unwrap();
//     //             let k = std::str::from_utf8(&p.0).unwrap().to_owned();
//     //             let v = std::str::from_utf8(&p.1).unwrap().to_owned();
//     //             let json : Event = serde_json::from_str(&v).unwrap();
//     //             let json_cloned = json.clone();
//     //             (k, json_cloned)
//     //         }).collect();

//     //         for (k, v) in vals {
//     //             let old_json = v.clone();
//     //             let old_json_clone = old_json.clone();
//     //             let mut new_json = v.clone();
//     //             new_json.published = true;
//     //             let _ = tree_cloned.compare_and_swap(old_json.event.as_bytes(), None as Option<&[u8]>, Some(b""));
//     //             let old_json_og = tree_cloned.get(old_json.event).unwrap().unwrap();
//     //             let old_value = std::str::from_utf8(&old_json_og).unwrap().to_owned();
//     //             let _ = tree_cloned.compare_and_swap(old_json_clone.event.as_bytes(), Some(old_value.as_bytes()), Some(serde_json::to_string(&new_json).unwrap().as_bytes()));
//     //             let _ = tree_cloned.compare_and_swap(k, Some(serde_json::to_string(&old_json_clone).unwrap().as_bytes()), Some(serde_json::to_string(&new_json).unwrap().as_bytes())); 
//     //             let _ = tree_cloned.flush();
//     //             // broadcast(new_json.event, serde_json::to_string(&new_json.data).unwrap(), events_cloned.clone());
//     //         }
//     //    }  
//     // });
//     // x.thread();

//     pretty_env_logger::init();



//     HttpServer::new(move || {
//         App::new()
//             .wrap(middleware::Logger::default())
//             .wrap(
//                 Cors::new()
//                     .allowed_origin(&config.origin)
//                     .allowed_methods(vec!["GET", "POST"])
//                     .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
//                     .max_age(3600)
//                     .finish()
//             )
//             .app_data(events.clone())
//             .app_data(web::JsonConfig::default())
//             .data(MyData{ db: tree_actix.clone(), config: config_cloned.clone() })
//             .route("/insert", web::post().to(insert))
//             .route("/events", web::get().to(new_client))
//             .route("/events/collections/{id}", web::get().to(collection))
//             .route("/events/{id}/cancel", web::get().to(cancel))
//             .route("/events/user", web::get().to(user_collection))
//             .route("/users", web::post().to(user_create))
//             .route("/login", web::post().to(login))
//     })
//     .bind(ip).unwrap()
//     .run()
// }
