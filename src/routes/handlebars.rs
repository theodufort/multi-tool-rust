use rocket_dyn_templates::handlebars::{Handlebars, Helper, Context, RenderContext, Output, HelperResult};

fn wow_helper(
    h: &Helper<'_>,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output
) -> HelperResult {
    if let Some(param) = h.param(0) {
        use std::fmt::Write as FmtWrite;
        let mut rendered = String::new();
        if let Err(_) = write!(&mut rendered, "{}", param.value()) {
            // handle error if needed
        }
        out.write("<b><i>")?;
        out.write(&rendered)?;
        out.write("</b></i>")?;
    }

    Ok(())
}

pub fn customize(hbs: &mut Handlebars) {
    hbs.register_helper("wow", Box::new(wow_helper));
    hbs.register_template_string("about.html", r#"
        {{#*inline "page"}}

        <section id="about">
          <h1>About - Here's another page!</h1>
        </section>

        {{/inline}}
        {{> layout}}
    "#).expect("valid HBS template");
}
