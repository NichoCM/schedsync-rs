use diesel::{deserialize::FromSqlRow, expression::AsExpression, serialize::ToSql, Queryable};
use oauth2::Oauth2Service;

pub mod oauth2;
pub mod caldav;

#[derive(Debug, Clone, FromSqlRow, AsExpression)]
#[diesel(sql_type = diesel::sql_types::SmallInt)]
pub enum ServiceType {
    Google(Oauth2Service),
    Outlook(Oauth2Service),
    Apple,
}

/**
 * Convert an i16 used in the database to a ServiceType.
 */
impl Queryable<diesel::sql_types::SmallInt, crate::db::Backend> for ServiceType {
    type Row = i16;
    fn build(row: Self::Row) -> Result<ServiceType, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        match row {
            1 => Ok(Self::Google(Oauth2Service::Google)),
            2 => Ok(Self::Outlook(Oauth2Service::Outlook)),
            3 => Ok(Self::Apple),
            _ => Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid ServiceType value")))
        }
    }
}

impl ToSql<diesel::sql_types::SmallInt, crate::db::Backend> for ServiceType {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, crate::db::Backend>) -> diesel::serialize::Result {
        match self {
            ServiceType::Google(_) => {
                let _ = ToSql::<diesel::sql_types::SmallInt, crate::db::Backend>::to_sql(&1, out);
                Ok(diesel::serialize::IsNull::No)
            },
            ServiceType::Outlook(_) => {
                let _ = ToSql::<diesel::sql_types::SmallInt, crate::db::Backend>::to_sql(&2, out);
                Ok(diesel::serialize::IsNull::No)
            },
            ServiceType::Apple => {
                let _ = ToSql::<diesel::sql_types::SmallInt, crate::db::Backend>::to_sql(&3, out);
                Ok(diesel::serialize::IsNull::No)
            },
        }
    }
}

impl ServiceType {
    
    /**
     * Get the service type from an Oauth2Service.
     */
    pub fn from_oauth2(service: Oauth2Service) -> Self {
        match service {
            Oauth2Service::Google => Self::Google(service),
            Oauth2Service::Outlook => Self::Outlook(service),
        }
    }
}