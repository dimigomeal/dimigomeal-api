use actix_web::{
    get, http::StatusCode, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use chrono::prelude::*;
use db::get_meal;
use regex::Regex;
use serde_derive::Deserialize;
use serde_json::json;
use std::sync::Arc;

mod db;
use crate::db::create_conn;

fn error_response(status_code: StatusCode, message: &str) -> HttpResponse {
    HttpResponse::build(status_code)
        .content_type("application/json; charset=utf-8")
        .json(json!({ "error": message }))
}

async fn process(target_date: &str) -> HttpResponse {
    let conn = create_conn();
    let meal = get_meal(&conn, target_date);
    conn.close().unwrap();

    match meal {
        Ok(meal) => {
            let meal_data = json!({
                "date": meal.date,
                "breakfast": meal.breakfast,
                "lunch": meal.lunch,
                "dinner": meal.dinner,
            });

            HttpResponse::Ok()
                .content_type("application/json; charset=utf-8")
                .json(meal_data)
        }
        Err(_) => error_response(StatusCode::NOT_FOUND, "Meal not found"),
    }
}

#[derive(Debug, Deserialize)]
struct MealParam {
    date: String,
}

#[get("/")]
async fn index(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    match web::Query::<MealParam>::from_query(req.query_string().as_ref()) {
        Ok(query) => {
            if !data.date_regex.is_match(&query.date) {
                return error_response(StatusCode::BAD_REQUEST, "Invalid date format");
            }
            process(&query.date).await
        }
        Err(_) => {
            let now_date = Utc::now().format("%Y-%m-%d").to_string();
            process(&now_date).await
        }
    }
}

struct AppState {
    date_regex: Arc<Regex>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Actix web server...");

    let date_regex =
        Arc::new(Regex::new(r"^\d{4}\-(0[1-9]|1[012])\-(0[1-9]|[12][0-9]|3[01])$").unwrap());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                date_regex: date_regex.clone(),
            }))
            .service(index)
        // .service(date)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
