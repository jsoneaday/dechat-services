pub mod routes {
    pub mod profile;
    pub mod post;
}
pub mod app_state;
pub mod test_helpers {
    pub mod fixtures;
}

use std::env;
use dotenv::dotenv;
use actix_web::{ web, App, HttpServer, middleware::Logger };

pub async fn run() -> std::io::Result<()> {
    dotenv().ok();

    let port: u16 = env::var("PORT").unwrap().parse().unwrap();
    let host = env::var("HOST").unwrap();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            //.app_data(app_data.clone())
            .service(
                web::scope("/v1")
                    // .service(web::resource("/msg/{id}").route(web::get().to(get_message::<DbRepo>)))
                    // .service(web::resource("/msg").route(web::post().to(create_message::<DbRepo>)))
                    // .service(web::resource("/msgs").route(web::post().to(get_messages::<DbRepo>)))
                    // .service(web::resource("/profile/{id}").route(web::get().to(get_profile::<DbRepo>)))
                    // .service(web::resource("/profile/username/{user_name}").route(web::get().to(get_profile_by_user::<DbRepo>)))
                    // .service(web::resource("/profile").route(web::post().to(create_profile::<DbRepo>)))
            )
    })
    .bind((host, port))?
    .run().await
}