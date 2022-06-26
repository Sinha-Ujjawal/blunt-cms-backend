use crate::{
    openapi::addons::BearerSecurity,
    views::{users::*, posts::*},
};
use actix_web::{get, http::StatusCode, web, HttpResponse};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    handlers(
        // users
        signup,
        login,
        get_user,
        validate_token,
        change_password,
        // posts
        create_post,
        get_posts,
        get_drafts,
        update_post_subject_handler,
        update_post_body_handler,
        delete_post,
        request_admin_to_publish,
        publish_post,
    ),
    components(
        // users
        UserData,
        SignUpInput, SignUpResponse,
        Token,
        LogInInput,
        UserChangePasswordInput,
        // posts
        PostData,
        CreatePostData,
        UpdatePostSubject,
        UpdatePostBody,
    ),
    tags(
        (name = "Content Management System", description = "Content Management System Apis")
    ),
    modifiers(&BearerSecurity)
)]
pub struct ApiDoc;

const OPENAPI_JSON_ENDPOINT: &'static str = "/api-doc/openapi.json";

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(SwaggerUi::new("/swagger-ui/{_:.*}").url(OPENAPI_JSON_ENDPOINT, ApiDoc::openapi()))
        .service(swagger_ui_handler);
}

#[get("/api-doc/ui.html")]
async fn swagger_ui_handler() -> HttpResponse {
    let ui_html = format!(
        r###"<!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <meta name="description" content="SwaggerUI" />
        <title>SwaggerUI</title>
        <link
        rel="stylesheet"
        href="https://unpkg.com/swagger-ui-dist@4.5.0/swagger-ui.css"
        />
    </head>
    <body>
        <div id="swagger-ui"></div>
        <script
        src="https://unpkg.com/swagger-ui-dist@4.5.0/swagger-ui-bundle.js"
        crossorigin
        ></script>
        <script>
        window.onload = () => {{
            window.ui = SwaggerUIBundle({{
            url: `${{window.location.origin}}{}`,
            dom_id: "#swagger-ui",
            }});
        }};
        </script>
    </body>
    </html>
"###,
        OPENAPI_JSON_ENDPOINT
    );
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(ui_html)
}
