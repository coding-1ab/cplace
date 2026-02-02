#[macro_use] extern crate rocket;
use server::{draw, get_drawing, get_draw_test};

#[launch]
fn rocket() -> _ {
    rocket::build()
        // #[post("/draw", format = "json", data = "<draw_request>")]
        .mount("/draw", routes![draw])

        // #[get("/get_pxel_png/<chunk_x>/<chunk_y>/<zoom_lv>")]
        .mount("/get", routes![get_draw_test])

        // #[get("/get_pxel_png/<chunkx>/<chunky>/<zoom_lv>")]
        .mount("/mock", routes![get_drawing])
}
