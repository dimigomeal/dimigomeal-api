use actix_web::{
    get, http::StatusCode, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use chrono::{prelude::*, Duration};
use db::{get_meal, get_multi_meals};
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

async fn process(target_date: &str, is_week: bool) -> HttpResponse {
    let conn = create_conn();
    if is_week {
        let start_date =
            NaiveDate::parse_from_str(target_date, "%Y-%m-%d").unwrap() - Duration::days(6);
        let end_date =
            NaiveDate::parse_from_str(target_date, "%Y-%m-%d").unwrap() + Duration::days(6);

        let start_date_str = start_date.format("%Y-%m-%d").to_string();
        let end_date_str = end_date.format("%Y-%m-%d").to_string();

        let meals = get_multi_meals(&conn, &start_date_str, &end_date_str);
        conn.close().unwrap();

        match meals {
            Ok(meals) => {
                let mut meal_list = Vec::new();
                for meal in meals {
                    let meal_data = json!({
                        "date": meal.date,
                        "breakfast": meal.breakfast,
                        "lunch": meal.lunch,
                        "dinner": meal.dinner,
                    });
                    meal_list.push(meal_data);
                }

                HttpResponse::Ok()
                    .content_type("application/json; charset=utf-8")
                    .json(meal_list)
            }
            Err(_) => error_response(StatusCode::NOT_FOUND, "Meals not found"),
        }
    } else {
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
            process(&query.date, false).await
        }
        Err(_) => {
            let now_date = Utc::now().format("%Y-%m-%d").to_string();
            process(&now_date, false).await
        }
    }
}

#[derive(Debug, Deserialize)]
struct WeekParam {
    date: String,
}

#[get("/week")]
async fn week(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    match web::Query::<WeekParam>::from_query(req.query_string().as_ref()) {
        Ok(query) => {
            if !data.date_regex.is_match(&query.date) {
                return error_response(StatusCode::BAD_REQUEST, "Invalid date format");
            }
            process(&query.date, true).await
        }
        Err(_) => {
            let now_date = Utc::now().format("%Y-%m-%d").to_string();
            process(&now_date, true).await
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
            .service(week)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
