use diesel::{deserialize::Queryable, insert_into, prelude::Insertable, query_builder::AsChangeset, ExpressionMethods, QueryDsl, RunQueryDsl, Selectable, SelectableHelper};
use serde::{Deserialize, Serialize};

#[derive(Clone, Queryable, Selectable, AsChangeset, Deserialize, Serialize)]
#[diesel(table_name = crate::schema::groups)]
#[diesel(check_for_backend(crate::db::Backend))]
pub struct Group {
    pub id: i32,
    pub app_id: i32,
}

impl Group {
    pub fn new(app_id: i32, conn: &mut crate::db::Connection) -> Self {
        insert_into(crate::schema::groups::table)
            .values(&CreateGroup {
                app_id,
            })
            .returning(Group::as_returning())
            .get_result(conn)
            .expect("Error saving new oauth integration")
    }

    pub fn find_by_id(id: i32, conn: &mut crate::db::Connection) -> Option<Group> {
        use crate::schema::groups::dsl;
        let Ok(result) = dsl::groups.select(Group::as_select())
            .filter(dsl::id.eq(id))
            .first::<Group>(conn)
        else {
            return None;
        };
        Some(result)
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::groups)]
struct CreateGroup {
    app_id: i32,
}