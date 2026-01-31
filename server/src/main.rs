#[macro_use]
extern crate rocket;
use server::{draw, get_drawing};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/post", routes![draw])
        .mount("/test", routes![get_drawing])
}
