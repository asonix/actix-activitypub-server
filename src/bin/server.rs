extern crate actix;
extern crate actix_ap_demo;
extern crate actix_web;
extern crate env_logger;
extern crate futures;

use std::collections::BTreeSet;

use actix::{Actor, SyncAddress};
use actix_web::{httpcodes, middleware, Application, AsyncResponder, Error, HttpRequest,
                HttpResponse, HttpServer};
use futures::Future;

use actix_ap_demo::actors::{Id, UserId};
use actix_ap_demo::actors::blocklist::Blocklists;
use actix_ap_demo::actors::peered::Peered;
use actix_ap_demo::actors::peered::messages::Message;
use actix_ap_demo::actors::posts::Posts;
use actix_ap_demo::actors::user::messages::{GetUserPostIds, NewPostOut};
use actix_ap_demo::actors::users::Users;
use actix_ap_demo::actors::users::messages::{Lookup, NewUser};

#[derive(Clone)]
struct BasicState {
    users: SyncAddress<Peered<Users>>,
    blocklists: SyncAddress<Peered<Blocklists>>,
}

fn new_user(req: HttpRequest<BasicState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let users = req.state().users.clone();
    let blocklists = req.state().blocklists.clone();

    req.state()
        .users
        .call_fut(Message::new(NewUser(users, blocklists)))
        .and_then(|res| Ok(res.unwrap()))
        .and_then(|user_id| Ok(httpcodes::HTTPOk.with_body(format!("{:?}", user_id))))
        .from_err()
        .responder()
}

fn new_post(req: HttpRequest<BasicState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let users_id = req.match_info().get("usid").unwrap().parse().unwrap();
    let user_id = req.match_info().get("uid").unwrap().parse().unwrap();

    let uid = UserId(Id(users_id), Id(user_id));

    req.state()
        .users
        .call_fut(Message::new(Lookup(uid)))
        .and_then(|res| Ok(res.unwrap()))
        .map(|u_addr| {
            u_addr.outbox().send(NewPostOut(BTreeSet::new()));
        })
        .and_then(|_| Ok(httpcodes::HTTPOk.with_body("created".to_owned())))
        .from_err()
        .responder()
}

fn get_posts(req: HttpRequest<BasicState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let users_id = req.match_info().get("usid").unwrap().parse().unwrap();
    let user_id = req.match_info().get("uid").unwrap().parse().unwrap();

    let uid = UserId(Id(users_id), Id(user_id));

    req.state()
        .users
        .call_fut(Message::new(Lookup(uid)))
        .and_then(|res| Ok(res.unwrap()))
        .and_then(|u_addr| u_addr.user().call_fut(GetUserPostIds(10)))
        .and_then(|res| Ok(res.unwrap()))
        .and_then(|post_id| Ok(httpcodes::HTTPOk.with_body(format!("{:?}", post_id))))
        .from_err()
        .responder()
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web,actix_ap_demo=info");
    env_logger::init();

    let sys = actix::System::new("system");

    let blocklists = Peered::new(Blocklists::new()).start();
    let posts = Peered::new(Posts::new(Id(0))).start();
    let users = Peered::new(Users::new(Id(0), posts)).start();

    let state = BasicState { users, blocklists };

    HttpServer::new(move || {
        Application::with_state(state.clone())
            .middleware(middleware::Logger::default())
            .resource("/new_user", |r| r.f(new_user))
            .resource("/new_post/{usid}/{uid}", |r| r.f(new_post))
            .resource("/get_posts/{usid}/{uid}", |r| r.f(get_posts))
    }).bind("127.0.0.1:8080")
        .unwrap()
        .start();

    sys.run();
}
