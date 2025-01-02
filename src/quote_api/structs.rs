use serde::{Deserialize, Serialize};

use crate::database::structs::quotes::Model as Quotes;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Quote {
    pub id: i64,
    pub quote: String,
    pub author: String,
    pub tags: Option<Vec<String>>,
}

pub fn to_database(quote: Quote) -> Quotes {
    Quotes {
        id: quote.id.to_string(),
        quote: quote.quote,
        author: quote.author,
        tags: quote.tags.unwrap_or_default(),
        likes: 0i32,
    }
}
