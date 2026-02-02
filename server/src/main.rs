#[macro_use] extern crate rocket;
use server::{draw, get_drawing, get_draw_test};

#[launch]
fn rocket() -> _ {
    rocket::build()
        // #[post("/draw", format = "json", data = "<draw_request>")]
        .mount("/draw", routes![draw])

        // #[get("/get_pixel_png/<chunk_x>/<chunk_y>/<zoom_lv>")]
        .mount("/get", routes![get_draw_test])

        // #[get("/get_pixel_png/<chunk_x>/<chunk_y>/<zoom_lv>")]
        .mount("/mock", routes![get_drawing])
}
