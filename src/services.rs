use actix_web::{post, Responder};

#[post("/user/signup")]
pub async fn signup() -> actix_web::Result<impl Responder> {
    Ok("User Signed up!")
}
