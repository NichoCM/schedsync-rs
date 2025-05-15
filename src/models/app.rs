use diesel::{deserialize::Queryable, insert_into, prelude::Insertable, query_builder::AsChangeset, ExpressionMethods, QueryDsl, RunQueryDsl, Selectable, SelectableHelper};
use uuid::Uuid;

use super::{app_key::AppKey, group::Group};

#[derive(Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::apps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct App {
    pub id: i32,
    pub client_id: String,
}

impl App {

    pub fn new(conn: &mut crate::db::Connection) -> Self {
        insert_into(crate::schema::apps::table)
        .values(&NewApp {
            client_id: Uuid::new_v4().to_string(),
        })
        .returning(App::as_returning())
        .get_result(conn)
        .expect("Error saving new app key")
    }

    pub fn find_by_client_id(client_id: String, conn: &mut crate::db::PooledConnection) -> Option<App> {
        use crate::schema::apps::dsl;
        let Ok(result) = dsl::apps.select(App::as_select())
            .filter(dsl::client_id.eq(client_id))
            .first::<App>(conn)
        else {
            return None;
        };
        Some(result)
    }

    /**
     * Create a new group for this app.
     */
    pub fn create_group(&self, conn: &mut crate::db::PooledConnection) -> Group {
        Group::new(self.id, conn)
    }

    /**
     * Create a new key for this app
     */
    pub fn create_key(&self, conn: &mut crate::db::PooledConnection) -> AppKey {
        AppKey::new(conn, self.id)
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::apps)]
struct NewApp {
    client_id: String,
}