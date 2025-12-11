use domain::entities::TermOfUse;

use crate::database::postgres::data::models::terms;

impl From<terms::Model> for TermOfUse {
    fn from(value: terms::Model) -> Self {
        TermOfUse {
            id: value.id,
            url: value.url,
            group: value.group,
            version: value.version as u32,
            info: value.info,
            created_at: value.created_at,
        }
    }
}
