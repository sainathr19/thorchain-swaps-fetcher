mod db;
mod fetcher;
mod models;
mod routes;
mod tests;
mod utils;
use actix_cors::Cors;
use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};
use db::MySQL;
use fetcher::fetch_historical_data;
use utils::cron::start_cronjob;

#[get("/")]
async fn home() -> impl Responder {
    HttpResponse::Ok().body("Rust Backend Server")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Spawn the fetch_historical_data task
    tokio::spawn(async { fetch_historical_data().await });

    // Initialize MySQL connection and start CronJob
    let mysql = MySQL::init().await;
    println!("Now starting CronJob");
    start_cronjob(mysql.clone()).await;

    // Create mysql_data for the Actix app
    let mysql_data = Data::new(mysql);

    // Start Actix-web server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(mysql_data.clone())
            .wrap(Cors::permissive())
            .service(home)
            .configure(routes::swap_history::init)
    })
    .bind(("0.0.0.0", 3000))
    .expect("Failed to bind Actix server")
    .run();

    server.await?;

    Ok(())
}
