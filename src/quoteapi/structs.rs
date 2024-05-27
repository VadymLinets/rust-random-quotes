use serde::Deserialize;

use crate::database::structs::quotes::Model as Quotes;

#[derive(Deserialize)]
pub struct Quote {
    id: String,
    content: String,
    author: String,
    tags: Vec<String>,
}

pub fn to_database(quote: Quote) -> Quotes {
    Quotes {
        id: quote.id,
        quote: quote.content,
        author: quote.author,
        tags: quote.tags,
        likes: 0,
    }
}
