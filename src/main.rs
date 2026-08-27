#[macro_use]
extern crate rocket;
extern crate rocket_dyn_templates;

mod routes;
mod tools;

use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![routes::index::index])
        .mount(
            "/",
            routes![
                routes::tools::api_with_action,
                routes::tools::api,
                routes::tools::tool_full,
                routes::tools::tool_output,
                routes::tools::tool,
            ],
        )
        .register("/", catchers![routes::not_found::not_found])
        .attach(Template::custom(|engines| {
            routes::handlebars::customize(&mut engines.handlebars);
        }))
}
