use serde::{Deserialize, Serialize};

use crate::database::structs::quotes::Model as Quotes;

#[derive(PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Quote {
    pub id: String,
    pub quote: String,
    pub author: String,
    pub tags: Vec<String>,
    pub likes: i32,
}

pub fn from_database_quote_to_quote(quote: Quotes) -> Quote {
    Quote {
        id: quote.id,
        quote: quote.quote,
        author: quote.author.unwrap_or_default(),
        tags: quote.tags.unwrap_or_default(),
        likes: quote.likes.unwrap_or_default(),
    }
}
