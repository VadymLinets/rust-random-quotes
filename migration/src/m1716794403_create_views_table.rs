use super::m1716794372_create_quotes_table::Quotes;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Views::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Views::UserId).text().not_null())
                    .col(ColumnDef::new(Views::QuoteId).text().not_null())
                    .col(ColumnDef::new(Views::Liked).boolean().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Views::Table, Views::QuoteId)
                            .to(Quotes::Table, Quotes::Id),
                    )
                    .primary_key(Index::create().col(Views::UserId).col(Views::QuoteId))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Views::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Views {
    Table,
    UserId,
    QuoteId,
    Liked,
}
