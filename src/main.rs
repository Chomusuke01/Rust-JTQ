#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use rocket::{Build, Data, Request, Rocket, State, catch, catchers, fairing::{self, Fairing, Info, Kind}, get, http::{Cookie, CookieJar, Method}, launch, post, response::status::Created, routes, serde::json::Json, uri};
use std::{sync::atomic::{AtomicUsize, Ordering}, collections::HashMap};
use rocket::tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use futures::executor;
use std::ops::Deref;
#[macro_use] extern crate rocket;

type ID = usize;

#[derive(Serialize, Debug, Clone)]
struct Visitor{
    id: ID,
    name: String,
    username: String,
    password: String,
    phone_number: String,
    #[serde(rename(serialize = "accepted_terms"))]
    accepted_terms: bool,
    #[serde(rename(serialize = "accepted_comercials"))]
    accepted_comercials: bool,
    #[serde(rename(serialize = "user_type"))]
    user_type: bool,
}

#[derive(Deserialize, Debug)]
struct VisitorDTO{
    name: String,
    username: String,
    password: String,
    phone_number: String,
    #[serde(rename(deserialize = "accepted_terms"))]
    accepted_terms: bool,
    #[serde(rename(deserialize = "accepted_comercials"))]
    accepted_comercials: bool,
    #[serde(rename(deserialize = "user_type"))]
    user_type: bool,
}

#[derive(Deserialize, Debug)]
struct VisitorLoginDTO{
    username: String,
    password: String,
}

struct VisitorCount(AtomicUsize);

type VisitorMap = RwLock<HashMap<ID, Visitor>>;

#[post("/register", format = "json", data = "<visitor>")]
fn add_visitor(visitor: Json<VisitorDTO>, visitor_state: &State<VisitorMap>, visitor_count: &State<VisitorCount>,) -> Created<Json<Visitor>>{

    let vid = visitor_count.0.fetch_add(1, Ordering::Relaxed);

    let new_visitor = Visitor {
        id: vid,
        name: visitor.0.name,
        username: visitor.0.username,
        password: visitor.0.password,
        phone_number: visitor.0.phone_number,
        accepted_terms: visitor.0.accepted_terms,
        accepted_comercials: visitor.0.accepted_comercials,
        user_type: visitor.0.user_type,
    };

    let mut visitors = executor::block_on(visitor_state.write());
    visitors.insert(vid, new_visitor.clone());

    let location = uri!("/api", get_visitor(vid));
    Created::new(location.to_string()).body(Json(new_visitor))
}

#[get("/visitor/<id>")]
fn get_visitor(id: ID, visitor_state: &State<VisitorMap>) -> Option<Json<Visitor>> {
    
    let visitors = async {
        let map = visitor_state.read().await;
        map.get(&id).map(|h| Json(h.clone()))
    };
    executor::block_on(visitors)
}

#[post("/login", format = "json", data = "<visitor>")]
fn login (visitor: Json<VisitorLoginDTO>, visitor_state: &State<VisitorMap>) -> Option<Json<Visitor>> {
    
    let visitors = executor::block_on(visitor_state.read());
    
    let users = visitors
    .clone()
    .into_iter()
    .map(|(_id,visitor)| visitor)
    .collect::<Vec<Visitor>>();

    let user = users
    .into_iter()
    .map(|vis | vis)
    .find(|visi | visitor.username.eq(&visi.username) && visitor.password.eq(&visi.password))
    .map (|v | Json(v));

    user
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![add_visitor,get_visitor,login])
    .manage(RwLock::new(HashMap::<ID, Visitor>::new()))
    .manage(VisitorCount(AtomicUsize::new(1)))
}