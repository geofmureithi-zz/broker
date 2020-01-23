use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::prelude::*;
use portal::{Filter, http::StatusCode, sse::ServerSentEvent};
use jsonwebtoken::{encode, decode, Header, Validation};
use tokio::sync::mpsc::channel;
use std::convert::Infallible;
use std::time::Duration;
use futures::StreamExt;

#[derive(Debug, Clone)]
pub struct SSE {
    pub event: String,
    pub data: String,
    pub id: String,
    pub retry: Duration,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
  pub port: String,
  pub origin: String,
  pub expiry: i64,
  pub secret: String,
  pub save_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JWT {
    check: bool,
    claims: Claims,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: uuid::Uuid,
    username: String,
    password: String,
    collection_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserForm {
    username: String,
    password: String,
    collection_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Login {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Cfg {
  pub save_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub collection_id: uuid::Uuid,
    pub event: String,
    pub timestamp: i64,
    pub published: bool,
    pub cancelled: bool,
    pub data: serde_json::Value,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
struct EventForm {
    collection_id: uuid::Uuid,
    event: String,
    timestamp: i64,
    data: serde_json::Value,
}

fn user_create(tree: sled::Db, user_form: UserForm) -> (bool, String) {

    // turn iVec(s) to String(s) and make HashMap
    let records : HashMap<String, String> = tree.iter().into_iter().filter(|x| {
        let p = x.as_ref().unwrap();
        let k = std::str::from_utf8(&p.0).unwrap().to_owned();
        let v = std::str::from_utf8(&p.1).unwrap().to_owned();
        if k.contains("_u_") {
            let user : User = serde_json::from_str(&v).unwrap();
            if user.username == user_form.username {
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
        let j = json!({"error": "username already taken"}).to_string();
        return (false, j)
    } else {
        // set as future value
        let uuid = Uuid::new_v4();
        let versioned = format!("_u_{}", uuid.to_string());
        let hashed = hash(user_form.clone().password, DEFAULT_COST).unwrap();
        let new_user = User{id: uuid, username: user_form.clone().username, password: hashed, collection_id: user_form.collection_id };
        
        let _ = tree.compare_and_swap(versioned.as_bytes(), None as Option<&[u8]>, Some(serde_json::to_string(&new_user).unwrap().as_bytes())); 
        let _ = tree.flush();
        let j = json!({"id": uuid.to_string()}).to_string();
        return (true, j)
    }
}

fn login(tree: sled::Db, login: Login, config: Config) -> (bool, String) {
    let now = Utc::now().timestamp();
    let expi = now + config.expiry;
    let expiry = expi as usize;

    let records : HashMap<String, String> = tree.iter().into_iter().filter(|x| {
        let p = x.as_ref().unwrap();
        let k = std::str::from_utf8(&p.0).unwrap().to_owned();
        if k.contains(&"_u_") {
            let v = std::str::from_utf8(&p.1).unwrap().to_owned();
            let user : User = serde_json::from_str(&v).unwrap();
            if user.username == login.username {
                return true
            } else {
                return false
            }
        } else {
            return false
        }
    }).map(|x| {
        let p = x.unwrap();
        let k = std::str::from_utf8(&p.0).unwrap().to_owned();
        let v = std::str::from_utf8(&p.1).unwrap().to_owned();
        (k, v)
    }).collect();

    for (_k, v) in records {
        let user : User = serde_json::from_str(&v).unwrap();
        let verified = verify(login.password, &user.password).unwrap();
        if verified {
            let my_claims = Claims{company: "".to_owned(), sub: user.id.to_string(), exp: expiry};
            let token = encode(&Header::default(), &my_claims, config.secret.as_ref()).unwrap();
            let j = json!({"jwt": token}).to_string();
            return (true, j)
        } else {
            return (false, "".to_owned())
        }
    }
    (false, "".to_owned())
}

fn config() -> Config {
    let mut port = "8080".to_owned();
    let mut expiry : i64 = 3600;
    let mut origin = "http://localhost:3000".to_owned();
    let mut secret = "secret".to_owned();
    let _ : Vec<String> = go_flag::parse(|flags| {
        flags.add_flag("port", &mut port);
        flags.add_flag("expiry", &mut expiry);
        flags.add_flag("origin", &mut origin);
        flags.add_flag("secret", &mut secret);
    });

    let cfg = envy::from_env::<Cfg>().unwrap();

    Config{port: port, origin: origin, secret: secret, save_path: cfg.save_path, expiry: expiry}
}

fn jwt_verify(config: Config, token: String) -> JWT {
    let parts = token.split(" ");
    for part in parts {
        if part != "Bearer" {
            let _ = match decode::<Claims>(&part, config.secret.as_ref(), &Validation::default()) {
                Ok(c) => {
                    return JWT{check: true, claims: c.claims}
                },
                Err(_e) => {
                    return JWT{check: false, claims: Claims{company: "".to_owned(), exp: 0, sub: "".to_owned()}}
                }
            };
        }
    }
    JWT{check: false, claims: Claims{company: "".to_owned(), exp: 0, sub: "".to_owned()}}
}

fn insert(tree: sled::Db, user_id_str: String, evt: EventForm) -> String {
    let user_id = uuid::Uuid::parse_str(&user_id_str).unwrap();

    let id = Uuid::new_v4();
    let j = Event{id: id, published: false, cancelled: false, data: evt.data, event: evt.event, timestamp: evt.timestamp, user_id: user_id, collection_id: evt.collection_id};
    let new_value_string = serde_json::to_string(&j).unwrap();
    let new_value = new_value_string.as_bytes();
    let versioned = format!("_v_{}", id.to_string());

    let _ = tree.compare_and_swap(versioned, None as Option<&[u8]>, Some(new_value.clone())); 
    let _ = tree.flush();

    json!({"event": j}).to_string()
}

fn event_stream(sse: SSE) -> Result<impl ServerSentEvent, Infallible> {
    Ok((
        portal::sse::id(sse.id),
        portal::sse::data(sse.data),
        portal::sse::event(sse.event),
        portal::sse::retry(sse.retry),
    ))
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let configure = config();
    let tree = sled::open(configure.save_path).unwrap();
    let tree_clone = tree.clone();
    let tree_clone1 = tree.clone();
    let tree_clone2 = tree.clone();

    let user_create_route = portal::post()
        .and(portal::path("users"))
        .and(portal::body::json())
        .map(move |user: UserForm| {
            let (check, value) = user_create(tree.clone(), user.clone());
            if check {
                let reply = portal::reply::with_status(value, StatusCode::OK);
                portal::reply::with_header(reply, "Content-Type", "application/json")
            } else {
                let reply = portal::reply::with_status(value, StatusCode::BAD_REQUEST);
                portal::reply::with_header(reply, "Content-Type", "application/json")
            }
        });
    
    let auth_check = portal::header::<String>("authorization").map(|token| {
        let configure = config();
        jwt_verify(configure, token)
    });

    let login_route = portal::post()
        .and(portal::path("login"))
        .and(portal::body::json())
        .map(move |login_form: Login| {
            let configure = config();
            let (check, value) = login(tree_clone1.clone(), login_form.clone(), configure.clone());
            if check {
                let reply = portal::reply::with_status(value, StatusCode::OK);
                portal::reply::with_header(reply, "Content-Type", "application/json")
            } else {
                let reply = portal::reply::with_status(value, StatusCode::UNAUTHORIZED);
                portal::reply::with_header(reply, "Content-Type", "application/json")
            }
        });

    let insert_route = portal::post()
        .and(portal::path("insert"))
        .and(auth_check)
        .and(portal::body::json())
        .map(move |jwt: JWT, event_form: EventForm| {
            let record = insert(tree_clone2.clone(), jwt.claims.sub, event_form);
            if jwt.check {
                let reply = portal::reply::with_status(record, StatusCode::OK);
                portal::reply::with_header(reply, "Content-Type", "application/json")
            } else {
                let reply = portal::reply::with_status("".to_owned(), StatusCode::UNAUTHORIZED);
                portal::reply::with_header(reply, "Content-Type", "application/json")
            }
        });
    
    let sse = portal::path("events").and(portal::get()).map(move || {
        let (mut s, r) = channel(100);
        let tree_clone = tree_clone.clone();

        tokio::spawn(async move {
            let vals: HashMap<String, String> = tree_clone.iter().into_iter().filter(|x| {
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

            for (k, v) in vals {
                let guid = Uuid::new_v4().to_string();
                let sse = SSE{id: guid, event: k, data: v, retry: Duration::from_millis(5000)};
                let _ = s.send(sse).await;
            }

            let _ = tokio::spawn(async move {
                loop {
                    let vals : HashMap<String, Event> = tree_clone.iter().into_iter().filter(|x| {
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

                    for (k, v) in vals {
                        let old_json = v.clone();
                        let old_json_clone = old_json.clone();
                        let mut new_json = v.clone();
                        new_json.published = true;
                        let newer_json = new_json.clone();
                        
                        let guid = Uuid::new_v4().to_string();
                        let sse = SSE{id: guid, event: new_json.event, data: serde_json::to_string(&new_json.data).unwrap(), retry: Duration::from_millis(5000)};
                        let _ = s.send(sse).await;
                        let tree_cloned = tree_clone.clone();
                        let _ = tokio::spawn(async move {
                            let _ = tree_cloned.compare_and_swap(old_json.event.as_bytes(), None as Option<&[u8]>, Some(b""));
                            let old_json_og = tree_cloned.get(old_json.event).unwrap().unwrap();
                            let old_value = std::str::from_utf8(&old_json_og).unwrap().to_owned();
                            let _ = tree_cloned.compare_and_swap(old_json_clone.event.as_bytes(), Some(old_value.as_bytes()), Some(serde_json::to_string(&newer_json).unwrap().as_bytes()));
                            let _ = tree_cloned.compare_and_swap(k, Some(serde_json::to_string(&old_json_clone).unwrap().as_bytes()), Some(serde_json::to_string(&newer_json).unwrap().as_bytes())); 
                            let _ = tree_cloned.flush();
                        }).await;
                    }
                }  
            }).await;
        });
        
        let events = r.map(|sse| {
            event_stream(sse)
        });
        portal::sse::reply(portal::sse::keep_alive().interval(Duration::from_secs(5)).text("ping".to_string()).stream(events))
    });

    let routes = portal::any().and(login_route).or(user_create_route).or(insert_route).or(sse);

    portal::serve(routes).run(([0, 0, 0, 0], 8080)).await
}