use diesel::{deserialize::Queryable, insert_into, prelude::Insertable, query_builder::AsChangeset, ExpressionMethods, QueryDsl, RunQueryDsl, Selectable, SelectableHelper};
use uuid::Uuid;

use super::group::Group;

#[derive(Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::oauth2_states)]
#[diesel(check_for_backend(crate::db::Backend))]
pub struct Oauth2State {
    id: i32,
    pub state: String,
    group_id: i32,
    expires_at: chrono::NaiveDateTime,
}

impl Oauth2State {
    pub fn new(
        group: &Group,
        conn: &mut crate::db::Connection,
    ) -> Self {
        let mut i = 0;
        // Try to generate a new state up to 3 times
        loop {
            let state = Oauth2State::generate(group, conn);
            match state {
                Ok(state) => break state,
                Err(e) => {
                    i += 1;
                    if i > 3 {
                        panic!("Could not generate a new state after 3 attempts: {:?}", e);
                    }
                }
            }
        }
    }

    pub fn delete(self, conn: &mut crate::db::Connection) -> Result<usize, diesel::result::Error> {
        use crate::schema::oauth2_states::dsl;
        diesel::delete(dsl::oauth2_states.find(self.id))
            .execute(conn)
    }

    /**
     * Generate a new state
     */
    fn generate(
        group: &Group,
        conn: &mut crate::db::Connection,
    ) -> Result<Oauth2State, diesel::result::Error>{
        insert_into(crate::schema::oauth2_states::table)
            .values(&CreateOauth2State {
                group_id: group.id,
                state: Uuid::new_v4().to_string(),
                expires_at: chrono::Utc::now().naive_utc() + chrono::Duration::minutes(5),
            })
            .returning(Oauth2State::as_returning())
            .get_result(conn)
    }

    /**
     * Find a state by the state string returned from the OAuth2 service.
     */
    pub fn find_by_string(value: &String, conn: &mut crate::db::Connection) -> Option<Oauth2State> {
        use crate::schema::oauth2_states::dsl;
        let result = dsl::oauth2_states.select(Oauth2State::as_select())
            .filter(dsl::state.eq(value))
            .first::<Oauth2State>(conn);
        match result {
            Ok(result) => {
                if result.expires_at < chrono::Utc::now().naive_utc() {
                    let _ = result.delete(conn);
                    return None;
                } else {
                    return Some(result);
                }
            },
            Err(_) => None,
        }
    }

    pub fn get_group(&self, conn: &mut crate::db::Connection) -> Group {
        Group::find_by_id(self.group_id, conn).unwrap()
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::oauth2_states)]
struct CreateOauth2State {
    group_id: i32,
    state: String,
    expires_at: chrono::NaiveDateTime,
}