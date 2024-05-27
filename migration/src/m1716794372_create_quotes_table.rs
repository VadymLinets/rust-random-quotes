use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Quotes::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Quotes::Id).text().not_null().primary_key())
                    .col(ColumnDef::new(Quotes::Quote).text().not_null().unique_key())
                    .col(ColumnDef::new(Quotes::Author).text().not_null())
                    .col(ColumnDef::new(Quotes::Likes).integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Quotes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Quotes {
    Table,
    Id,
    Quote,
    Author,
    Likes,
}
