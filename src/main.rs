#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use rocket::{Build, Data, Request, Rocket, State, catch, catchers, fairing::{self, Fairing, Info, Kind}, get, http::{Cookie, Method}, launch, post, options,response::status::Created, routes, serde::json::Json, uri};
use std::{sync::atomic::{AtomicUsize, Ordering}, collections::HashMap};
use rocket::tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use futures::executor;
use std::ops::Deref;
use core::option;
use csrf::{AesGcmCsrfProtection, CsrfProtection};
use data_encoding::BASE64;
use std::time::Duration;


#[macro_use] 
extern crate rocket;
extern crate rocket_cors;
extern crate csrf;
extern crate data_encoding;
extern crate bcrypt;

use rocket_cors::{
    AllowedHeaders, AllowedOrigins, Error, 
    Cors, CorsOptions 
};

type ID = usize;

#[derive(Serialize, Debug, Clone)]
struct Visitor{
    id: ID,
    name: String,
    username: String,
    password: String,
    phone_number: String,
    #[serde(rename(serialize = "acceptedTerms"))]
    accepted_terms: bool,
    #[serde(rename(serialize = "acceptedCommercial"))]
    accepted_comercial: bool,
    #[serde(rename(serialize = "userType"))]
    user_type: bool,
}

#[derive(Deserialize, Debug)]

struct VisitorDTO{
    name: String,
    username: String,
    password: String,
    #[serde(rename(deserialize = "phoneNumber"))]
    phone_number: String,
    #[serde(rename(deserialize = "acceptedTerms"))]
    accepted_terms: bool,
    #[serde(rename(deserialize = "acceptedCommercial"))]
    accepted_comercial: bool,
    #[serde(rename(deserialize = "userType"))]
    user_type: bool,
}

#[derive(Deserialize, Debug)]
struct VisitorLoginDTO{
    username: String,
    password: String,
}
#[derive(Serialize, Debug)]
struct TokenDTO {
    token: String,
}

struct VisitorCount(AtomicUsize);

type VisitorMap = RwLock<HashMap<String, Visitor>>;

fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[ 
        "http://localhost:4200",
        "http://127.0.0.1:4200",
    ]);

    CorsOptions { 
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Delete].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::All,
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}


#[post("/visitormanagement/v1/visitor", format = "json", data = "<visitor>")]
fn add_visitor(visitor: Json<VisitorDTO>, visitor_state: &State<VisitorMap>, visitor_count: &State<VisitorCount>,) -> Option<Json<Visitor>>{

    let vid = visitor_count.0.fetch_add(1, Ordering::Relaxed);

    let new_visitor = Visitor {
        id: vid,
        name: visitor.0.name,
        username: visitor.0.username,
        password: bcrypt::hash(visitor.0.password,8).unwrap(),
        phone_number: visitor.0.phone_number,
        accepted_terms: visitor.0.accepted_terms,
        accepted_comercial: visitor.0.accepted_comercial,
        user_type: visitor.0.user_type,
    };

    let mut visitors = executor::block_on(visitor_state.write());  
    let check = visitors.insert(new_visitor.clone().username, new_visitor.clone());

    match check{
        Some(_vis) => None,

        None => Some(Json(new_visitor))
    }

}

#[post("/login", format = "json", data = "<visitor>")]
fn login (visitor: Json<VisitorLoginDTO>, visitor_state: &State<VisitorMap>) -> Option<Json<Visitor>> {
    
    let visitors = executor::block_on(visitor_state.read());
    
    visitors.get(&visitor.0.username)
    .map(|v |v.clone())
    .filter(|v | bcrypt::verify(&visitor.password, &v.password).unwrap())
    .map(|vis | Json(vis.clone()))

}

//visitor.password.eq(&v.password)
#[get("/csrf/v1/token")]
fn get_auth_token() -> Json<TokenDTO>{
    let protect = AesGcmCsrfProtection::from_key(*b"01234567012345670123456701234567");
    let tok = protect
    .generate_token_pair(None, 300)
    .expect("No se pudo generar el token");

    //let mut c = Cookie::new("XSRF-TOKEN", tok.1.b64_string());
    let token = TokenDTO{
        token: tok.0.b64_string(),
    };

    Json(token)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
    .attach(make_cors())
    .mount("/", routes![add_visitor,login,get_auth_token])
    .manage(RwLock::new(HashMap::<String, Visitor>::new()))
    .manage(VisitorCount(AtomicUsize::new(1)))
}
