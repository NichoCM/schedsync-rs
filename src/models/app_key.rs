use diesel::{deserialize::Queryable, insert_into, prelude::Insertable, query_builder::AsChangeset, ExpressionMethods, QueryDsl, RunQueryDsl, Selectable, SelectableHelper};
use uuid::Uuid;

use super::app::App;

#[derive(Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::app_keys)]
#[diesel(check_for_backend(crate::db::Backend))]
pub struct AppKey {
    pub id: i32,
    pub app_id: i32,
    pub key: String,
    pub key_preview: String,
}

impl AppKey {
    pub fn new(conn: &mut crate::db::PooledConnection, app_id: i32) -> Self {
        let key = Uuid::new_v4().to_string();
        let mut key_preview: String = key.clone().chars().take(5).collect();
        key_preview.push_str("...");
        key_preview.push_str(&key.chars().skip(key.len() - 5).collect::<String>());
        insert_into(crate::schema::app_keys::table)
            .values(&NewAppKey {
                app_id,
                key,
                key_preview,
            })
            .returning(AppKey::as_returning())
            .get_result(conn)
            .expect("Error saving new app key")
    }

    pub fn save(&self, conn: &mut crate::db::PooledConnection) -> Self {
        diesel::update(crate::schema::app_keys::table.find(self.id))
            .set(self)
            .returning(AppKey::as_returning())
            .get_result(conn)
            .expect("Error saving app key")
    }

    pub fn delete(&self, conn: &mut crate::db::PooledConnection) -> usize {
        diesel::delete(crate::schema::app_keys::table)
            .execute(conn)
            .expect("Error deleting app key")
    }
    
    /**
     * Find an app key by its key and app id.
     */
    pub fn find_by_app_key(
        app: &App,
        authorization_key: &String,
        conn: &mut crate::db::PooledConnection
    ) -> Option<AppKey> {
        use crate::schema::app_keys::dsl::*;
        let Ok(result) = app_keys.select(AppKey::as_select())
            .filter(key.eq(authorization_key))
            .filter(app_id.eq(app.id))
            .first::<AppKey>(conn)
        else {
            return None;
        };
        Some(result)
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::app_keys)]
struct NewAppKey{
    app_id: i32,
    key: String,
    key_preview: String,
}