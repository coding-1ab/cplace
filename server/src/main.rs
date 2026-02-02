#[macro_use] extern crate rocket;
use server::{draw, get_drawing, get_draw_test};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/draw", routes![draw])
        .mount("/get", routes![get_draw_test])
        .mount("/test", routes![get_drawing, get_draw_test])
}
