use crate::repo::base::{EntityId, DbRepo, DbConnGetter};
use mockall::automock;
use sqlx::{ Pool, Postgres };
use async_trait::async_trait;

mod private_members {
    use super::*;

    pub async fn insert_standalone_post_inner(
        conn: &Pool<Postgres>,
        chain_id: &str,
        user_id: i64,
        message: &str
    ) -> Result<EntityId, sqlx::Error> {
        let insert_msg_result = sqlx
            ::query_as::<_, EntityId>(
                "insert into post (chain_id, user_id, message) values ($1, $2, $3) returning id"
            )
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
}

#[automock]
#[async_trait]
pub trait InsertPostFn {
    async fn insert_standalone_post(
        &self,
        chain_id: &str,
        user_id: i64,
        message: &str,
    ) -> Result<EntityId, sqlx::Error>;
}

#[async_trait]
impl InsertPostFn for DbRepo {
    async fn insert_standalone_post(
        &self,
        chain_id: &str,
        user_id: i64,
        message: &str
    ) -> Result<EntityId, sqlx::Error> {
        private_members::insert_standalone_post_inner(
            self.get_conn(),
            chain_id,
            user_id,
            message
        ).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{ Arc, RwLock };
    use fake::{ faker::name::en::{ Name, FirstName, LastName }, Fake };
    use lazy_static::lazy_static;
    use crate::repo::profile::{
        profile::{InsertProfileFn, MockInsertProfileFn},
        model::ProfileCreate,
    };
    use super::*;

    #[allow(unused)]
    #[derive(Clone)]
    struct Fixtures {
        pub respondee_post_id: i64,
        pub profile_id: i64,
        pub profile_create: ProfileCreate,
        pub db_repo: DbRepo
    }

    const PREFIX: &str = "TestPost";

    lazy_static! {
        static ref FIXTURES: Arc<RwLock<Option<Fixtures>>> = Arc::new(RwLock::new(None));
    }

    async fn setup_db_test_data(_db_repo: DbRepo) -> Result<(), Box<dyn std::error::Error>> {
        // todo: fill in data
        Ok(())
    }

    async fn setup_local_fixture_data(db_repo: DbRepo) -> Fixtures {
        _ = setup_db_test_data(db_repo.clone()).await;

        let first_name: String = FirstName().fake();
        let last_name: String = LastName().fake();
        let profile_create = ProfileCreate {
            chain_id: "chain_id123".to_string(),
            user_name: Name().fake(),
            full_name: format!("{} {}", first_name, last_name),
            description: format!("{} a description", PREFIX),
            main_url: Some("http://whatever.com".to_string()),
            avatar: None::<Vec<u8>>,
        };
        let profile_id = db_repo.insert_profile(profile_create.clone()).await.unwrap();
        let respondee_post_id = db_repo
            .insert_standalone_post(format!("{}chain_id", PREFIX).as_str(), profile_id, format!("{}Testing body 123", PREFIX).as_str())
            .await
            .unwrap();

        Fixtures {
            respondee_post_id: respondee_post_id.id,
            profile_id,
            profile_create,
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
                    chain_id: "dummy".to_string(),
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
                    profile_id,
                    format!("{}Body of message that is being responded to.", PREFIX).as_str()
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
}