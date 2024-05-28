use serde::{Deserialize, Serialize};

use crate::database::structs::quotes::Model as Quotes;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Quote {
    #[serde(rename = "_id")]
    pub id: String,
    pub content: String,
    pub author: String,
    pub tags: Vec<String>,
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
