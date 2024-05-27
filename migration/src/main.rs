use sea_orm_migration::prelude::*;

pub mod m1716794372_create_quotes_table;

#[async_std::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
