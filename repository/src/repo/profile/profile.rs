use crate::repo::base::{ DbRepo, DbConnGetter };
use crate::repo::profile::model::{ProfileCreate, ProfileUpdate, ProfileQueryResult};
use crate::repo::base::EntityId;
use async_trait::async_trait;
use sqlx::{ Pool, Postgres };
use mockall::automock;
use mockall::predicate::*;
#[allow(unused)]
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
                    (chain_id, user_name, full_name, description, main_url, avatar) 
                    values 
                    ($1, $2, $3, $4, $5, $6)
                returning id"
            )
            .bind(&params.chain_id)
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
        user_name: &str
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
        user_name: &str
    ) -> Result<Option<ProfileQueryResult>, sqlx::Error>;
}

#[async_trait]
impl QueryProfileByUserNameFn for DbRepo {
    async fn query_profile_query_profile_by_user_name(
        &self,
        user_name: &str
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
    async fn setup_db_test_data(db_repo: DbRepo) -> Result<(), Box<dyn std::error::Error>> {
        let conn = db_repo.get_conn();

        let username_dave = format!("{}dave", PREFIX);
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
                    (chain_id, user_name, full_name, description, main_url, avatar) 
                    values 
                    ($1, $2, $3, $4, $5, $6)
                    returning id
                "
            )
            .bind("chain_id123")
            .bind(username_dave)
            .bind(format!("{}Dave Choi", PREFIX))
            .bind(format!("{}I am a chef", PREFIX))
            .bind(Some(format!("{}http://chef.com", PREFIX)))
            .bind(None::<Vec<u8>>)
            .fetch_all(conn)
            .await;
        }

        let username_jill = format!("{}jill", PREFIX);
        let profile_dave = sqlx::query_as::<_, ProfileQueryResult>(
            r"
                select *
                from profile
                where user_name = $1
            "
        )
        .bind(username_jill.clone())
        .fetch_optional(conn)
        .await
        .unwrap();

        if let None = profile_dave {
            info!("profile_dave missing, inserting now");
            _ = sqlx::query_as::<_, EntityId>(
                r"
                    insert into Profile 
                    (chain_id, user_name, full_name, description, main_url, avatar) 
                    values 
                    ($1, $2, $3, $4, $5, $6)
                    returning id
                "
            )
            .bind("chain_id123")
            .bind(username_jill)
            .bind(format!("{}Jill Simon", PREFIX))
            .bind(format!("{}I am a developer", PREFIX))
            .bind(Some(format!("{}http://dev.com", PREFIX)))
            .bind(None::<Vec<u8>>)
            .fetch_all(conn)
            .await;
        }

        Ok(())
    }

    /// Set local fixtures data
    async fn set_local_fixture_data(db_repo: DbRepo) -> Fixtures {
        setup_db_test_data(db_repo.clone()).await.unwrap();

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

            let user_name = format!("{}insert_tester", PREFIX);
            let full_name = format!("{}Insert Tester", PREFIX);
            let description = format!("{}Insert Test description", PREFIX);
            let profile_id = fixtures.db_repo
                .insert_profile(ProfileCreate {
                    chain_id: "chain_id123".to_string(),
                    user_name: user_name.clone(),
                    full_name: full_name.clone(),
                    description: description.clone(),
                    main_url: Some("http://whatever.com".to_string()),
                    avatar: Some(vec![]),
                }).await
                .unwrap();

            let profile = sqlx::query_as::<_, ProfileQueryResult>(
                r"
                    select * from profile where id = $1
                "
            )
            .bind(profile_id)
            .fetch_one(fixtures.db_repo.get_conn())
            .await
            .unwrap();

            assert!(profile.id > 0);
            assert!(profile.user_name == user_name);
            assert!(profile.full_name == full_name);
            assert!(profile.description == description);
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
            let username_jill = format!("{}jill", PREFIX);
            let username_jill_str = username_jill.as_str();            
            let profile = fixtures.db_repo.query_profile_query_profile_by_user_name(username_jill_str).await.unwrap().unwrap();

            let full_name = format!("{}Update Tester", PREFIX);
            let description = format!("{}Update Test description", PREFIX);
            _ = fixtures.db_repo
                .update_profile(
                    profile.id, 
                    ProfileUpdate {                    
                        full_name: full_name.clone(),
                        description: description.clone(),
                        main_url: Some("http://updater.com".to_string()),
                        avatar: Some(vec![]),
                    }
                ).await;

            let profile_result = sqlx::query_as::<_, ProfileQueryResult>(
                r"
                    select * from profile where id = $1
                "
            )
            .bind(profile.id)
            .fetch_one(fixtures.db_repo.get_conn())
            .await
            .unwrap();

            assert!(profile_result.id == profile.id);
            assert!(profile_result.full_name == full_name);
            assert!(profile_result.description == description);

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
            let username = format!("{}jill", PREFIX);
            let username_str = username.as_str(); 

            let profile = fixtures.db_repo
                .query_profile_query_profile_by_user_name(username_str)
                .await
                .unwrap()
                .unwrap();

            assert!(profile.user_name == username_str);
        }

        #[test]
        fn query_profile_by_user_name() {
            RT.block_on(test_query_profile_by_user_name_body())
        }
    }
}