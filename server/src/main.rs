#[macro_use] extern crate rocket;
use server::draw;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/post", routes![draw])
}