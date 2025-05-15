use diesel::{deserialize::Queryable, insert_into, prelude::Insertable, query_builder::AsChangeset, QueryDsl, RunQueryDsl, Selectable, SelectableHelper};

use crate::{connectors::ServiceType, schema::integrations::group_id};

use super::group::Group;

#[derive(Debug)]
#[derive(Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::integrations)]
#[diesel(check_for_backend(crate::db::Backend))]
pub struct Integration {
    pub id: i32,
    pub service: ServiceType,
    pub group_id: i32,
}

impl Integration {
    
    pub fn new(group: &Group, conn: &mut crate::db::PooledConnection, service: ServiceType) -> Self {
        insert_into(crate::schema::integrations::table)
            .values(&NewIntegration {
                group_id: group.id,
                service,
            })
            .returning(Integration::as_returning())
            .get_result(conn)
            .expect("Error saving new oauth integration")
    }

    pub fn save(&self, conn: &mut crate::db::PooledConnection) -> Self {
        diesel::update(crate::schema::integrations::table.find(self.id))
            .set(self)
            .returning(Integration::as_returning())
            .get_result(conn)
            .expect("Error saving oauth integration")
    }

    pub fn delete(&self, conn: &mut crate::db::PooledConnection) -> usize {
        diesel::delete(crate::schema::integrations::table)
            .execute(conn)
            .expect("Error deleting oauth integration")
    }

}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::integrations)]
struct NewIntegration {
    service: ServiceType,
    group_id: i32,
}