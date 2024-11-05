mod db;
mod fetcher;
mod models;
mod tests;
mod utils;
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use db::MySQL;
use fetcher::fetch_historical_data;
use utils::cron::start_cronjob;

#[get("/")]
async fn home() -> impl Responder {
    HttpResponse::Ok().body("Rust Backend Server")
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize MySQL connection
    let mysql = MySQL::init().await;

    // Historical Data Fetcher (runs only one time)
    let res = fetch_historical_data(&mysql).await;
    match res {
        Ok(_) => {}
        Err(err) => {
            println!("{:?}", err);
        }
    }

    println!("Historial Data Fetched . Now starting CronJob");
    let mysql_clone = mysql.clone();
    start_cronjob(mysql_clone).await;

    // Start the HTTP server
    HttpServer::new(move || App::new().service(home))
        .bind(("0.0.0.0", 3000))?
        .run()
        .await
}
