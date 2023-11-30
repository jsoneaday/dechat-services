use crate::repo::base::{ DbRepo, DbConnGetter };
use crate::repo::profile::model::ProfileQueryResult;
use async_trait::async_trait;
use sqlx::{ Pool, Postgres };
use mockall::automock;
use mockall::predicate::*;

mod private_members {
    use super::*;

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
        _ = sqlx::query_as::<_, EntityId>(
            r"
            insert into Profile 
            (user_name, full_name, description, main_url, avatar) 
            values 
            ($1, $2, $3, $4, $5, $6)
            returning id
            "
        )
        .bind(format!("{}dave", PREFIX))
        .bind(format!("{}Dave Choi", PREFIX))
        .bind(Some(format!("{}I am a chef", PREFIX)))
        .bind(Some(format!("{}http://test.com", PREFIX)))
        .bind(None::<Vec<u8>>)
        .fetch_all(db_repo.get_conn())
        .await;

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
                setup_fixtures().await;
            });

            rt
        };
    }

    #[allow(unused)]
    fn fixtures() -> Fixtures {
        Arc::clone(&FIXTURES).read().unwrap().clone().unwrap()
    }

    mod test_query_profile_by_user {
        use super::*;

        async fn test_insert_profile_body() {
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
        fn test_insert_profile() {
            RT.block_on(test_insert_profile_body())
        }
    }
}