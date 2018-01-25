// use actix::{Actor, Address};

pub mod dispatch;
pub mod posts;
pub mod user;
pub mod users;

// use self::dispatch::Dispatch;
// use self::posts::Posts;
// use self::users::Users;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(u64);

pub type PostId = Id;
pub type UserId = Id;

/*
fn start_server() {
    let posts: Address<_> = Posts::new().start();
    let users: Address<_> = Users::new().start();

    let dispatch: Address<_> = Dispatch::new(users.clone()).start();

    let user_id_1 = users.new_user(posts.clone(), dispatch.clone());
    let user_id_2 = users.new_user(posts.clone(), dispatch.clone());
    let user_id_3 = users.new_user(posts.clone(), dispatch.clone());
}
*/
