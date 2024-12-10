mod config;
mod db;
mod dtos;
mod error;
mod models;
mod utils;
mod auth;
mod handler;

use actix_cors::Cors;
use actix_web::{
    get, http::header, middleware::Logger, web, App, HttpResponse, HttpServer, Responder,
};
use config::Config;
use db::DBClient;
use dotenv::dotenv;
use dtos::*;
use sqlx::postgres::PgPoolOptions;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;
use handler::{auth as authHandler, users};

#[derive(Debug, Clone)]
pub struct AppState {
    pub env: Config,
    pub db_client: DBClient,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        authHandler::login,authHandler::logout,authHandler::register, users::get_me, users::get_users, health_checker_handler
    ),
    components(
        schemas(UserData,FilterUserDto,LoginUserDto,RegisterUserDto,UserResponseDto,UserLoginResponseDto,Response,UserListResponseDto)
    ),
    tags(
        (name = "Auth Api based on Rust!", description = "Authentication in Rust API using Actix-web")
    )
)]

struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();

        components.add_security_scheme(
            "token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
    }
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    openssl_probe::init_ssl_cert_env_vars();

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }

    dotenv().ok();
    env_logger::init();

    let config = Config::init();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    match sqlx::migrate!("./migrations").run(&pool).await {
        Ok(_) => println!("Migrations ran successfully"),
        Err(err) => eprintln!("Error running migrations: {}", err),
    }

    let db_client = DBClient::new(pool);
    let app_state = AppState {
        env: config.clone(),
        db_client,
    };

    println!(
        "{}",
        format!("Server is running on http://localhost:{}", config.port)
    );

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:8000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
            .supports_credentials();


        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .service(handler::auth::auth_handler())
            .service(users::users_handler())
            .service(health_checker_handler)
            .service(Redoc::with_url("/redoc", openapi.clone()))
            .service(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            .service(SwaggerUi::new("/{_:.*}").url("/api-docs/openapi.json", openapi.clone()))
    })
        .bind(("0.0.0.0", config.port))?
        .run()
        .await?;
    Ok(())

}

#[utoipa::path(
    get,
    path = "/api/health_checker",
    tag = "Health Checker Endpoint",
    responses((status = 200, description = "Authenticated User", body = Response)))]
#[get("/api/health_checker")]
async fn health_checker_handler() -> impl Responder {
    const MESSAGE: &str = "Rust Api Ready to goooooooo !!!!";

    HttpResponse::Ok().json(serde_json::json!({ "status":"success", "message": MESSAGE }))
}
