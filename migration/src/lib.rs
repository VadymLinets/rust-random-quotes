pub use sea_orm_migration::prelude::*;

mod m1716794372_create_quotes_table;
mod m1716794403_create_views_table;
mod m1716796965_alter_quotes_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m1716794372_create_quotes_table::Migration),
            Box::new(m1716794403_create_views_table::Migration),
            Box::new(m1716796965_alter_quotes_table::Migration),
        ]
    }
}
