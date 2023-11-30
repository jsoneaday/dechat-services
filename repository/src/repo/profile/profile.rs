use crate::repo::base::{ DbRepo, DbConnGetter };
use crate::repo::profile::model::{ProfileCreate, ProfileUpdate, ProfileQueryResult};
use crate::repo::base::EntityId;
use async_trait::async_trait;
use sqlx::{ Pool, Postgres };
use mockall::automock;
use mockall::predicate::*;
use log::{error, info};

mod private_members {
    use super::*;

    pub async fn insert_profile_inner(
        conn: &Pool<Postgres>,
        params: ProfileCreate
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx
            ::query_as::<_, EntityId>(
                r"
                insert into Profile 
                    (user_name, full_name, description, main_url, avatar) 
                    values 
                    ($1, $2, $3, $4, $5)
                returning id"
            )
            .bind(&params.user_name)
            .bind(&params.full_name)
            .bind(&params.description)
            .bind(&params.main_url)
            .bind(&params.avatar)
            .fetch_one(conn).await;

        match result {
            Ok(r) => Ok(r.id),
            Err(e) => {
                error!("create_profile error: {}", e);
                Err(e)
            }
        }
    }

    pub async fn update_profile_inner(
        conn: &Pool<Postgres>,
        user_id: i64,
        params: ProfileUpdate
    ) -> Result<(), sqlx::Error> {
        let update_result = sqlx
            ::query::<_>("update profile set full_name = $1, description = $2, main_url = $3, avatar = $4 where id = $5")
            .bind(params.full_name)
            .bind(params.description)
            .bind(params.main_url)
            .bind(params.avatar)
            .bind(user_id)
            .execute(conn).await;

        match update_result {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub async fn query_profile_by_user_name_inner(
        conn: &Pool<Postgres>,
        user_name: String
    ) -> Result<Option<ProfileQueryResult>, sqlx::Error> {
        sqlx::query_as::<_, ProfileQueryResult>("select * from profile where user_name = $1")
            .bind(user_name)
            .fetch_optional(conn).await
    }
}

#[automock]
#[async_trait]
pub trait InsertProfileFn {
    async fn insert_profile(
        &self,
        params: ProfileCreate
    ) -> Result<i64, sqlx::Error>;
}

#[async_trait]
impl InsertProfileFn for DbRepo {
    async fn insert_profile(
        &self,
        params: ProfileCreate
    ) -> Result<i64, sqlx::Error> {
        private_members::insert_profile_inner(self.get_conn(), params).await
    }
}

#[automock]
#[async_trait]
pub trait UpdateProfileFn {
    async fn update_profile(
        &self,
        user_id: i64,
        params: ProfileUpdate
    ) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl UpdateProfileFn for DbRepo {
    async fn update_profile(
        &self,
        user_id: i64,
        params: ProfileUpdate
    ) -> Result<(), sqlx::Error> {
        private_members::update_profile_inner(self.get_conn(), user_id, params).await
    }
}

#[automock]
#[async_trait]
pub trait QueryProfileByUserNameFn {
    async fn query_profile_query_profile_by_user_name(
        &self,
        user_name: String
    ) -> Result<Option<ProfileQueryResult>, sqlx::Error>;
}

#[async_trait]
impl QueryProfileByUserNameFn for DbRepo {
    async fn query_profile_query_profile_by_user_name(
        &self,
        user_name: String
    ) -> Result<Option<ProfileQueryResult>, sqlx::Error> {
        private_members::query_profile_by_user_name_inner(self.get_conn(), user_name).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repo::base::EntityId;
    use super::*;
    use lazy_static::lazy_static;
    use std::sync::{ Arc, RwLock };

    #[derive(Clone)]
    #[allow(unused)]
    struct Fixtures {
        profiles: Vec<ProfileQueryResult>,
        db_repo: DbRepo
    }

    /// Helps prevent clashes between other running tests, by adding unique prefix values for data
    const PREFIX: &str = "TestProfile";

    lazy_static! {
        static ref FIXTURES: Arc<RwLock<Option<Fixtures>>> = Arc::new(RwLock::new(None));
    }

    /// Add Profile data for this set of tests
    async fn setup_db_profile_test_data(db_repo: DbRepo) -> Result<(), Box<dyn std::error::Error>> {
        let conn = db_repo.get_conn();
        let username_dave = format!("{}dave", PREFIX);
        info!("Check profile_dave already inserted");
        let profile_dave = sqlx::query_as::<_, ProfileQueryResult>(
            r"
                select *
                from profile
                where user_name = $1
            "
        )
        .bind(username_dave.clone())
        .fetch_optional(conn)
        .await
        .unwrap();

        if let None = profile_dave {
            info!("profile_dave missing, inserting now");
            _ = sqlx::query_as::<_, EntityId>(
                r"
                    insert into Profile 
                    (user_name, full_name, description, main_url, avatar) 
                    values 
                    ($1, $2, $3, $4, $5)
                    returning id
                "
            )
            .bind(username_dave)
            .bind(format!("{}Dave Choi", PREFIX))
            .bind(format!("{}I am a chef", PREFIX))
            .bind(Some(format!("{}http://test.com", PREFIX)))
            .bind(None::<Vec<u8>>)
            .fetch_all(conn)
            .await;
        } else {
            info!("profile_dave already inserted");
        }

        Ok(())
    }

    /// Set local fixtures data
    async fn set_local_fixture_data(db_repo: DbRepo) -> Fixtures {
        setup_db_profile_test_data(db_repo.clone()).await.unwrap();

        let profiles = sqlx
            ::query_as::<_, ProfileQueryResult>(
                "select * from profile where description like 'TestProfile%'"
            )
            .fetch_all(db_repo.get_conn()).await
            .unwrap();

        Fixtures {
            profiles,
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

                *fx = Some(set_local_fixture_data(db_repo).await);
            }
        }
    }

    lazy_static! {
        static ref RT: tokio::runtime::Runtime = {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

            rt.block_on(async {
                env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
                setup_fixtures().await;
            });

            rt
        };
    }

    #[allow(unused)]
    fn fixtures() -> Fixtures {
        Arc::clone(&FIXTURES).read().unwrap().clone().unwrap()
    }

    mod test_mod_insert_profile {
        use super::*;

        async fn test_insert_profile_body() {
            let fixtures = fixtures();

            let profile_id = fixtures.db_repo
                .insert_profile(ProfileCreate {
                    user_name: format!("{}user_a", PREFIX),
                    full_name: format!("{}User A", PREFIX),
                    description: format!("{}Test description", PREFIX),
                    main_url: Some("http://whatever.com".to_string()),
                    avatar: Some(vec![]),
                }).await
                .unwrap();

            assert!(profile_id > 0);
        }

        #[test]
        fn test_insert_profile() {
            RT.block_on(test_insert_profile_body())
        }
    }

    mod test_mod_update_profile {
        use super::*;

        async fn test_update_profile_body() {
            let fixtures = fixtures();
            let username_dave = format!("{}dave", PREFIX);
            let profile = fixtures.db_repo.query_profile_query_profile_by_user_name(username_dave).await.unwrap().unwrap();

            let result = fixtures.db_repo
                .update_profile(
                    profile.id, 
                    ProfileUpdate {                    
                        full_name: format!("{}User A", PREFIX),
                        description: format!("{}Test description", PREFIX),
                        main_url: Some("http://whatever.com".to_string()),
                        avatar: Some(vec![]),
                    }
                ).await;

            assert!(result.is_ok());
        }

        #[test]
        fn test_update_profile() {
            RT.block_on(test_update_profile_body())
        }
    }

    mod test_query_profile_by_user_name {
        use super::*;

        async fn test_query_profile_by_user_name_body() {
            let fixtures = fixtures();
            let user_name = format!("{}dave", PREFIX);

            let profile = fixtures.db_repo
                .query_profile_query_profile_by_user_name(user_name.clone())
                .await
                .unwrap()
                .unwrap();

            assert!(profile.user_name == user_name);
        }

        #[test]
        fn query_profile_by_user_name() {
            RT.block_on(test_query_profile_by_user_name_body())
        }
    }
}