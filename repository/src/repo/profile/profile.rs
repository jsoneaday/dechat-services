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
pub trait QueryProfileFn {
    async fn query_profile(
        &self,
        user_name: String
    ) -> Result<Option<ProfileQueryResult>, sqlx::Error>;
}

#[async_trait]
impl QueryProfileFn for DbRepo {
    async fn query_profile(
        &self,
        user_name: String
    ) -> Result<Option<ProfileQueryResult>, sqlx::Error> {
        private_members::query_profile_by_user_name_inner(self.get_conn(), user_name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use std::sync::{ Arc, RwLock };

    #[derive(Clone)]
    #[allow(unused)]
    struct Fixtures {
        profiles: Vec<ProfileQueryResult>,
        db_repo: DbRepo
    }

    #[allow(unused)]
    const PREFIX: &str = "Test profile";

    lazy_static! {
        static ref FIXTURES: Arc<RwLock<Option<Fixtures>>> = Arc::new(RwLock::new(None));
    }

    async fn setup_db_data(_db_repo: DbRepo) -> Result<(), Box<dyn std::error::Error>> {        
        Ok(())
    }

    async fn setup_local_data(db_repo: DbRepo) -> Fixtures {
        setup_db_data(db_repo.clone()).await.unwrap();

        let profiles = sqlx
            ::query_as::<_, ProfileQueryResult>(
                "select * from profile where description like 'Test profile%'"
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

                *fx = Some(setup_local_data(db_repo).await);
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
}