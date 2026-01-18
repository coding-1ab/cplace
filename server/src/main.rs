#[macro_use]
extern crate rocket;
use server::{draw, px_data_to_png_test};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/post", routes![draw])
        .mount("/test", routes![px_data_to_png_test])
}
