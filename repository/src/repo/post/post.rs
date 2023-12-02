use crate::repo::base::{EntityId, DbRepo, DbConnGetter};
use mockall::automock;
use sqlx::{ Pool, Postgres };
use async_trait::async_trait;

mod private_members {
    use super::*;

    pub async fn insert_standalone_post_inner(
        conn: &Pool<Postgres>,
        chain_asset_id: &str,
        chain_id: i64,
        user_id: i64,
        message: &str
    ) -> Result<EntityId, sqlx::Error> {
        let insert_msg_result = sqlx
            ::query_as::<_, EntityId>(
                "insert into post (chain_asset_id, chain_id, user_id, message) values ($1, $2, $3, $4) returning id"
            )
            .bind(chain_asset_id)
            .bind(chain_id)
            .bind(user_id)
            .bind(message)
            .fetch_one(conn)
            .await;

        match insert_msg_result {
            Ok(row) => Ok(row),
            Err(e) => Err(e)
        }
    }

    pub async fn insert_response_post_inner(
        conn: &Pool<Postgres>,
        chain_asset_id: &str,
        chain_id: i64,
        user_id: i64,
        message: &str,
        respondee_post_id: i64
    ) -> Result<EntityId, sqlx::Error> {
        let insert_post_result = sqlx
            ::query_as::<_, EntityId>(
                "insert into post (chain_asset_id, chain_id, user_id, message) values ($1, $2, $3, $4) returning id"
            )
            .bind(chain_asset_id)
            .bind(chain_id)
            .bind(user_id)
            .bind(message)
            .fetch_one(conn)
            .await;

        let insert_post_id = match insert_post_result {
            Ok(row) => Ok(row),
            Err(e) => Err(e)
        };
        if insert_post_id.is_err() {
            return insert_post_id;
        }

        let insert_response_result = sqlx::query_as::<_, EntityId>(
            "insert into post_response (respondee_post_id, responder_post_id) values ($1, $2) returning id"
        )
        .bind(respondee_post_id)
        .bind(insert_post_id.as_ref().unwrap().id)
        .fetch_one(conn)
        .await;

        match insert_response_result {
            Ok(_) => insert_post_id,
            Err(e) => Err(e)
        }        
    }
}

#[automock]
#[async_trait]
pub trait InsertPostFn {
    async fn insert_standalone_post(
        &self,
        chain_asset_id: &str,
        chain_id: i64,
        user_id: i64,
        message: &str,
    ) -> Result<EntityId, sqlx::Error>;
}

#[async_trait]
impl InsertPostFn for DbRepo {
    async fn insert_standalone_post(
        &self,
        chain_asset_id: &str,
        chain_id: i64,
        user_id: i64,
        message: &str
    ) -> Result<EntityId, sqlx::Error> {
        private_members::insert_standalone_post_inner(
            self.get_conn(),
            chain_asset_id,
            chain_id,
            user_id,
            message
        ).await
    }
}

#[automock]
#[async_trait]
pub trait InsertResponsePostFn {
    async fn insert_response_post(
        &self,
        chain_asset_id: &str,
        chain_id: i64,
        user_id: i64,
        message: &str,
        respondee_post_id: i64
    ) -> Result<EntityId, sqlx::Error>;
}

#[async_trait]
impl InsertResponsePostFn for DbRepo {
    async fn insert_response_post(
        &self,
        chain_asset_id: &str,
        chain_id: i64,
        user_id: i64,
        message: &str,
        respondee_post_id: i64
    ) -> Result<EntityId, sqlx::Error> {
        private_members::insert_response_post_inner(
            self.get_conn(),
            chain_asset_id,
            chain_id,
            user_id,
            message,
            respondee_post_id
        ).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{ Arc, RwLock };
    use fake::{ faker::name::en::{ FirstName, LastName }, Fake };
    use lazy_static::lazy_static;
    use crate::repo::{profile::{
        profile::{InsertProfileFn, MockInsertProfileFn},
        model::{ProfileCreate, ProfileQueryResult},
    }, post::model::PostWithProfileQueryResult};
    use crate::test_helpers::fixtures::SUI_CHAIN_ID;
    use super::*;

    #[allow(unused)]
    #[derive(Clone)]
    struct Fixtures {
        pub responder_post_id: i64,
        pub respondee_post_id: i64,
        pub profile_id: i64,
        pub profile_create: ProfileCreate,
        pub db_repo: DbRepo
    }

    const PREFIX: &str = "TestPost";    

    lazy_static! {
        static ref FIXTURES: Arc<RwLock<Option<Fixtures>>> = Arc::new(RwLock::new(None));
    }

    fn get_test_profile_create() -> ProfileCreate {
        let first_name: String = format!("{}{}", PREFIX, FirstName().fake::<String>());
        let last_name: String = LastName().fake();
        let user_name = "johnson";
        ProfileCreate {
            chain_asset_id: "chain_id123".to_string(),
            chain_id: SUI_CHAIN_ID,
            user_name: format!("{}{}", PREFIX, user_name),
            full_name: format!("{} {}", first_name, last_name),
            description: format!("{} a description", PREFIX),
            main_url: Some("http://whatever.com".to_string()),
            avatar: None::<Vec<u8>>,
        }
    }

    async fn setup_db_test_data(db_repo: DbRepo) -> Result<(), Box<dyn std::error::Error>> {
        let profile_create = get_test_profile_create();
        let existing_user = sqlx::query_as::<_, ProfileQueryResult>(
            "select * from profile where user_name = $1"
        )
        .bind(profile_create.clone().user_name)
        .fetch_optional(db_repo.get_conn())
        .await
        .unwrap();

        #[allow(unused)]
        let mut profile_id = 0;
        if let None = existing_user {
            profile_id = db_repo.insert_profile(profile_create).await.unwrap();
        } else {
            profile_id = existing_user.unwrap().id;
        }

        let respondee_message = format!("{}Respondee message 123", PREFIX);
        let existing_respondee_post = sqlx::query_as::<_, PostWithProfileQueryResult>(
            r"
                select
                    pt.id,
                    pt.updated_at,
                    pt.chain_asset_id,
                    pt.chain_id,
                    pt.message,
                    pt.image,
                    pt.user_id,
                    pe.user_name,
                    pe.full_name,
                    pe.avatar,
                    pr.respondee_post_id
                from post pt
                    join
                profile pe
                    on pt.user_id = pe.id
                    left join
                post_response pr
                    on pt.id = pr.responder_post_id
                where message = $1 and pt.user_id = $2
            "
        )
        .bind(respondee_message.clone())
        .bind(profile_id)
        .fetch_optional(db_repo.get_conn())
        .await;
        
        match existing_respondee_post {
            Ok(option_post) => {
                match option_post {
                    None => {
                        let respondee_post_id = db_repo
                        .insert_standalone_post(format!("{}chain_id", PREFIX).as_str(), SUI_CHAIN_ID, profile_id, respondee_message.as_str())
                        .await
                        .unwrap();

                        _ = db_repo
                        .insert_response_post(format!("{}chain_id", PREFIX).as_str(), SUI_CHAIN_ID, profile_id, format!("{}Responder message 123", PREFIX).as_str(), respondee_post_id.id)
                        .await
                        .unwrap();
                    },
                    _ => println!("responder and respondee posts already exist")
                }
            },
            Err(e) => {
                panic!("{:?}", e);
            }
        };

        Ok(())
    }

    async fn setup_local_fixture_data(db_repo: DbRepo) -> Fixtures {
        _ = setup_db_test_data(db_repo.clone()).await;

        let profile = sqlx::query_as::<_, ProfileQueryResult>("select * from profile where user_name like $1")
            .bind("%johnson%")
            .fetch_one(db_repo.get_conn())
            .await
            .unwrap();

        let respondee_message = format!("{}Respondee message 123", PREFIX);        
        let respondee_post = sqlx::query_as::<_, PostWithProfileQueryResult>(
            r"
                select
                    pt.id,
                    pt.updated_at,
                    pt.chain_asset_id,
                    pt.chain_id,
                    pt.message,
                    pt.image,
                    pt.user_id,
                    pe.user_name,
                    pe.full_name,
                    pe.avatar,
                    pr.respondee_post_id
                from post pt
                    join
                profile pe
                    on pt.user_id = pe.id
                left join post_response pr
                    on pt.id = pr.responder_post_id
                where message = $1 and pt.user_id = $2
            ")
            .bind(respondee_message)
            .bind(profile.id)
            .fetch_one(db_repo.get_conn())
            .await;

        if respondee_post.is_err() {
            panic!("{:?}", respondee_post.err());
        }

        let responder_message = format!("{}Responder message 123", PREFIX);        
        let responder_post = sqlx::query_as::<_, PostWithProfileQueryResult>(
            r"
                select
                    pt.id,
                    pt.updated_at,
                    pt.chain_asset_id,
                    pt.chain_id,
                    pt.message,
                    pt.image,
                    pt.user_id,
                    pe.user_name,
                    pe.full_name,
                    pe.avatar,
                    pr.respondee_post_id
                from post pt
                    join
                profile pe
                    on pt.user_id = pe.id
                left join post_response pr
                    on pt.id = pr.responder_post_id
                where message = $1 and pt.user_id = $2
            ")
            .bind(responder_message)
            .bind(profile.id)
            .fetch_one(db_repo.get_conn())
            .await;

        if responder_post.is_err() {
            panic!("{:?}", responder_post.err());
        }

        Fixtures {
            responder_post_id: responder_post.unwrap().id,
            respondee_post_id: respondee_post.unwrap().id,
            profile_id: profile.id,
            profile_create: get_test_profile_create(),
            db_repo,
        }
    }

    async fn setup_fixtures() {
        let fixtures = Arc::clone(&FIXTURES);
        let mut fx = fixtures.write().unwrap();
        match fx.clone() {
            Some(_) => (),
            None => {
                let db_repo = DbRepo::init().await;

                *fx = Some(setup_local_fixture_data(db_repo).await);
            }
        }
    }

    lazy_static! {
        static ref RT: tokio::runtime::Runtime = {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

            rt.block_on(async {
                setup_fixtures().await;
            });

            rt
        };
    }

    #[allow(unused)]
    fn fixtures() -> Fixtures {
        Arc::clone(&FIXTURES).read().unwrap().clone().unwrap()
    }

    fn get_insert_profile_mock() -> MockInsertProfileFn {
        let mut mock_insert_profile = MockInsertProfileFn::new();

        mock_insert_profile
            .expect_insert_profile()
            .returning(move |_| { Ok(fixtures().profile_id) });

        mock_insert_profile
    }

    mod test_mod_insert_post {
        use super::*;

        async fn test_insert_post_body() {
            let fixtures = fixtures();

            let mock_insert_profile = get_insert_profile_mock();
            let profile_id = mock_insert_profile
                .insert_profile(ProfileCreate {
                    chain_asset_id: "dummy".to_string(),
                    chain_id: SUI_CHAIN_ID,
                    user_name: "dummy".to_string(),
                    full_name: "dummy".to_string(),
                    description: "dummy".to_string(),
                    main_url: Some("dummy".to_string()),
                    avatar: Some(vec![]),
                }).await
                .unwrap();

            let respondee_post_id = fixtures.db_repo
                .insert_standalone_post(
                    format!("{}chain_id", PREFIX).as_str(),
                    SUI_CHAIN_ID,
                    profile_id,
                    format!("{}test_insert_post", PREFIX).as_str()
                )
                .await
                .unwrap();

            assert!(respondee_post_id.id > 0);
        }

        #[test]
        fn test_insert_post() {
            RT.block_on(test_insert_post_body())
        }
    }

    mod test_mod_insert_response_post {
        use super::*;

        async fn test_insert_response_post_body() {
            let fixtures = fixtures();

            let mock_insert_profile = get_insert_profile_mock();
            let profile_id = mock_insert_profile
                .insert_profile(ProfileCreate {
                    chain_asset_id: "dummy".to_string(),
                    chain_id: SUI_CHAIN_ID,
                    user_name: "dummy".to_string(),
                    full_name: "dummy".to_string(),
                    description: "dummy".to_string(),
                    main_url: Some("dummy".to_string()),
                    avatar: Some(vec![]),
                }).await
                .unwrap();

            let responder_post_id = fixtures.db_repo
                .insert_response_post(
                    format!("{}chain_id", PREFIX).as_str(),
                    SUI_CHAIN_ID,
                    profile_id,
                    format!("{}test_insert_response_post", PREFIX).as_str(),
                    fixtures.respondee_post_id
                )
                .await
                .unwrap();

            assert!(responder_post_id.id > 0);
        }

        #[test]
        fn test_insert_response_post() {
            RT.block_on(test_insert_response_post_body())
        }
    }
}