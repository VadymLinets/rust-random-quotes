use juniper::GraphQLObject;

#[derive(GraphQLObject)]
pub struct Quote {
    pub id: String,
    pub quote: String,
    pub author: String,
    pub tags: Vec<String>,
    pub likes: i32,
}

#[derive(GraphQLObject)]
pub struct QuoteResult {
    pub success: bool,
    pub errors: Vec<String>,
    pub quote: Option<Quote>,
}

#[derive(GraphQLObject)]
pub struct EmptyResult {
    pub success: bool,
    pub errors: Vec<String>,
}
