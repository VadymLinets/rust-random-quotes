use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(Quotes::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Quotes::Tags)
                            .array(ColumnType::Text)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                TableAlterStatement::new()
                    .table(Quotes::Table)
                    .drop_column(Alias::new(Quotes::Tags.to_string()))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Quotes {
    Table,
    Tags,
}
