use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::prelude::*;
use portal::{Filter, http::StatusCode};
use jsonwebtoken::{encode, decode, Header, Validation};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
  pub port: String,
  pub origin: String,
  pub expiry: i64,
  pub secret: String,
  pub save_path: String,
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
    // add timestamp
    let now = Utc::now().timestamp();
    let expi = now + config.expiry;
    let expiry = expi as usize;

    // turn iVec(s) to String(s) and make HashMap
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

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let user_create_route = portal::post()
        .and(portal::path("users"))
        .and(portal::body::json())
        .map(|user: UserForm| {
            let config = config();
            let tree = sled::open(config.save_path).unwrap();
            let (check, value) = user_create(tree.clone(), user.clone());
            if check {
                let reply = portal::reply::with_status(value, StatusCode::OK);
                portal::reply::with_header(reply, "Content-Type", "application/json")
            } else {
                let reply = portal::reply::with_status(value, StatusCode::BAD_REQUEST);
                portal::reply::with_header(reply, "Content-Type", "application/json")
            }
        });
    
    let login_route = portal::post()
        .and(portal::path("login"))
        .and(portal::body::json())
        .map(|login_form: Login| {
            let config = config();
            let config_clone = config.clone();
            let tree = sled::open(config.save_path).unwrap();
            let (check, value) = login(tree.clone(), login_form.clone(), config_clone);
            if check {
                let reply = portal::reply::with_status(value, StatusCode::OK);
                portal::reply::with_header(reply, "Content-Type", "application/json")
            } else {
                let reply = portal::reply::with_status(value, StatusCode::UNAUTHORIZED);
                portal::reply::with_header(reply, "Content-Type", "application/json")
            }
        });

    let routes = portal::any().and(user_create_route).or(login_route);

    portal::serve(routes).run(([0, 0, 0, 0], 8080)).await
}