use chrono::NaiveDateTime;
use diesel::{deserialize::Queryable, insert_into, prelude::Insertable, query_builder::AsChangeset, QueryDsl, RunQueryDsl, Selectable, SelectableHelper};

use crate::connectors::oauth2::Oauth2Service;

use super::integration::Integration;

#[derive(Debug)]
#[derive(Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::oauth_integrations)]
#[diesel(check_for_backend(crate::db::Backend))]
pub struct OauthIntegration {
    pub id: i32,
    pub service: Oauth2Service,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: NaiveDateTime,
}

impl OauthIntegration {

    pub fn new(
        integration: &Integration,
        conn: &mut crate::db::PooledConnection,
        service: Oauth2Service,
        access_token: String,
        refresh_token: String,
        expires_at: chrono::NaiveDateTime
    ) -> Self {
        insert_into(crate::schema::oauth_integrations::table)
            .values(&NewOauthIntegration {
                integration_id: integration.id,
                service,
                access_token,
                refresh_token,
                expires_at,
            })
            .returning(OauthIntegration::as_returning())
            .get_result(conn)
            .expect("Error saving new oauth integration")
    }

    pub fn save(&self, conn: &mut crate::db::PooledConnection) -> Self {
        diesel::update(crate::schema::oauth_integrations::table.find(self.id))
            .set(self)
            .returning(OauthIntegration::as_returning())
            .get_result(conn)
            .expect("Error saving oauth integration")
    }

    pub fn delete(&self, conn: &mut crate::db::PooledConnection) -> usize {
        diesel::delete(crate::schema::oauth_integrations::table)
            .execute(conn)
            .expect("Error deleting oauth integration")
    }

}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::oauth_integrations)]
struct NewOauthIntegration {
    integration_id: i32,
    access_token: String,
    service: Oauth2Service,
    refresh_token: String,
    expires_at: chrono::NaiveDateTime,
}