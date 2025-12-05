use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const TABLE_TERMS: &str = "terms";
const TABLE_USER_AGREEMENTS: &str = "user_agreements";

const INDEX_TERMS_GROUP_VERSION: &str = "idx_terms_group_version";

const FK_USER_AGREEMENTS_TERM_OF_USE_ID: &str = "fk-user_agreements-term_of_use_id";
const INDEX_USER_AGREEMENTS_USER_TERM: &str = "idx_user_agreements_user_term";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TABLE_TERMS)
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(string("group"))
                    .col(string("url"))
                    .col(unsigned("version"))
                    .col(text("info").null())
                    .col(date_time("created_at").not_null())
                    .index(
                        Index::create()
                            .unique()
                            .name(INDEX_TERMS_GROUP_VERSION)
                            .col("group")
                            .col("version"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TABLE_USER_AGREEMENTS)
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(unsigned("term_of_use_id"))
                    .col(unsigned("user_id"))
                    .col(date_time("agreed_at").not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_USER_AGREEMENTS_TERM_OF_USE_ID)
                            .from(TABLE_USER_AGREEMENTS, "term_of_use_id")
                            .to(TABLE_TERMS, "id"),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name(INDEX_USER_AGREEMENTS_USER_TERM)
                            .col("user_id")
                            .col("term_of_use_id"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name(FK_USER_AGREEMENTS_TERM_OF_USE_ID)
                    .table(TABLE_USER_AGREEMENTS)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(TABLE_USER_AGREEMENTS).to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(INDEX_TERMS_GROUP_VERSION)
                    .table(TABLE_TERMS)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(TABLE_TERMS).to_owned())
            .await
    }
}
