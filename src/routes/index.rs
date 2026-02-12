use rocket::http::uri::Origin;
use rocket_dyn_templates::{Template, context};

#[get("/")]
pub fn index(uri: &Origin<'_>) -> Template {
    Template::render(
        "index",
        context! {
            uri: uri,
        },
    )
}
