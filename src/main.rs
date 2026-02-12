#[macro_use]
extern crate rocket;

mod routes;

use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![routes::index::index])
        .register("/", catchers![routes::not_found::not_found])
        .attach(Template::custom(|engines| {
            routes::handlebars::customize(&mut engines.handlebars);
        }))
}
