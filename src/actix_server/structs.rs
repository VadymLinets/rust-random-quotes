use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserID {
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct UserAndQuoteID {
    pub user_id: String,
    pub quote_id: String,
}
