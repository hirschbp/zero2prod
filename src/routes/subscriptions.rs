use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    let _ = _form.email;
    let _ = _form.name;
    HttpResponse::Ok().finish()
}
