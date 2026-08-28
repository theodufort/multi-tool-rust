use rocket::Request;
use rocket_dyn_templates::{Template, context};

#[catch(404)]
pub fn not_found(req: &Request<'_>) -> Template {
    Template::render(
        "error/404",
        context! {
            uri: req.uri(),
            sidebar: super::tools::sidebar_links()
        },
    )
}
