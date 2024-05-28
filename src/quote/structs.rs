use serde::{Deserialize, Serialize};

use crate::database::structs::quotes::Model as Quotes;

#[derive(PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Quote {
    id: String,
    quote: String,
    author: String,
    tags: Vec<String>,
    likes: i32,
}

pub fn from_database_quote_to_quote(quote: Quotes) -> Quote {
    Quote {
        id: quote.id,
        quote: quote.quote,
        author: quote.author,
        tags: quote.tags,
        likes: quote.likes,
    }
}
