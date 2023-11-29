use std::ops::Range;
use crate::app_state::AppState;
use fake::{faker::{internet::en::Username, name::en::{LastName, FirstName}, lorem::en::Sentence, address::en::CountryName}, Fake};
use repository::repo::base::DbRepo;
use repository::test_helpers::fixtures::get_fake_main_url;
use actix_web::{ App, web::{ self, BytesMut, Bytes }, Error, test, dev::{ Service, ServiceResponse } };
use actix_http::Request;

#[allow(unused)]
pub async fn get_app_state<T>(db_repo: T) -> AppState<T> {
    AppState {
        client: reqwest::Client::new(),
        db_repo,
    }
}

pub async fn get_app_data<T>(db_repo: T) -> web::Data<AppState<T>> {
    web::Data::new(get_app_state(db_repo).await)
}

#[allow(unused)]
pub async fn get_app() -> impl Service<Request, Response = ServiceResponse, Error = Error> {
    let app_data = get_app_data(DbRepo::init().await).await;
    test::init_service(
        App::new()
            .app_data(app_data.clone())            
            .service(
                web::scope("/v1")
                    // .service(web::resource("/msg/{id}").route(web::get().to(get_message::<DbRepo>)))
                    // .service(web::resource("/msg").route(web::post().to(create_message::<DbRepo>)))
                    // .service(web::resource("/msgs").route(web::post().to(get_messages::<DbRepo>)))
                    // .service(web::resource("/profile/{id}").route(web::get().to(get_profile::<DbRepo>)))
                    // .service(web::resource("/profile/username/{user_name}").route(web::get().to(get_profile_by_user::<DbRepo>)))
                    // .service(web::resource("/profile").route(web::post().to(create_profile::<DbRepo>)))
            )
    ).await
}

/// warning: line breaks are very important when ending any line!!!
pub fn get_profile_create_multipart(
    avatar: &Vec<u8>,
    boundary: &str,
    with_avatar: bool
) -> BytesMut {
    let mut payload = actix_web::web::BytesMut::new();
    payload.extend(format!("--{}\r\n", boundary).as_bytes());
    payload.extend(
        format!("Content-Disposition: form-data; name=\"user_name\"\r\n\r\n").as_bytes()
    );
    payload.extend(format!("{}\r\n", Username().fake::<String>()).as_bytes());
    payload.extend(format!("--{}\r\n", boundary).as_bytes());
    payload.extend(
        format!("Content-Disposition: form-data; name=\"full_name\"\r\n\r\n").as_bytes()
    );
    payload.extend(
        format!("{} {}\r\n", FirstName().fake::<String>(), LastName().fake::<String>()).as_bytes()
    );
    payload.extend(format!("--{}\r\n", boundary).as_bytes());
    payload.extend(
        format!("Content-Disposition: form-data; name=\"description\"\r\n\r\n").as_bytes()
    );
    payload.extend(
        format!("{}\r\n", Sentence(Range { start: 8, end: 10 }).fake::<String>()).as_bytes()
    );
    payload.extend(format!("--{}\r\n", boundary).as_bytes());
    payload.extend(format!("Content-Disposition: form-data; name=\"region\"\r\n\r\n").as_bytes());
    payload.extend(format!("{}\r\n", CountryName().fake::<String>()).as_bytes());
    payload.extend(format!("--{}\r\n", boundary).as_bytes());
    
    payload.extend(format!("Content-Disposition: form-data; name=\"main_url\"\r\n\r\n").as_bytes());    
    payload.extend(get_fake_main_url().as_bytes());
    payload.extend(b"\r\n"); // warning: line breaks are very important!!! 
    payload.extend(format!("--{}\r\n", boundary).as_bytes());

    if with_avatar == true {
        payload.extend(
            b"Content-Disposition: form-data; name=\"avatar\"; filename=\"profile.jpeg\"\r\n"
        );
        payload.extend(b"Content-Type: image/jpeg\r\n\r\n");
        payload.extend(Bytes::from(avatar.clone()));
        payload.extend(b"\r\n"); // warning: line breaks are very important!!!        
    }
    payload.extend(format!("--{}--\r\n", boundary).as_bytes()); // note the extra -- at the end of the boundary

    payload
}