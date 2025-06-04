use georm::Georm;

#[derive(Debug, Georm, PartialEq, Eq, Default)]
#[georm(
    table = "biographies",
    one_to_one = [{
        name = "author", remote_id = "biography_id", table = "authors", entity = Author
    }]
)]
pub struct Biography {
    #[georm(id)]
    pub id: i32,
    pub content: String,
}

#[derive(Debug, Georm, PartialEq, Eq, Default)]
#[georm(table = "authors")]
pub struct Author {
    #[georm(id)]
    pub id: i32,
    pub name: String,
    #[georm(relation = {entity = Biography, table = "biographies", name = "biography", nullable = true})]
    pub biography_id: Option<i32>,
}

impl PartialOrd for Author {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for Author {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Debug, Georm, PartialEq, Eq, Default)]
#[georm(
    table = "books",
    one_to_many = [
        { name = "reviews", remote_id = "book_id", table = "reviews", entity = Review }
    ],
    many_to_many = [{
        name = "genres",
        table = "genres",
        entity = Genre,
        link = { table = "book_genres", from = "book_id", to = "genre_id" }
    }]
)]
pub struct Book {
    #[georm(id)]
    ident: i32,
    title: String,
    #[georm(relation = {entity = Author, table = "authors", name = "author"})]
    author_id: i32,
}

impl PartialOrd for Book {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.ident.cmp(&other.ident))
    }
}

impl Ord for Book {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ident.cmp(&other.ident)
    }
}

#[derive(Debug, Georm, PartialEq, Eq)]
#[georm(table = "reviews")]
pub struct Review {
    #[georm(id)]
    pub id: i32,
    #[georm(relation = {entity = Book, table = "books", remote_id = "ident", name = "book"})]
    pub book_id: i32,
    pub review: String,
}

#[derive(Debug, Georm, PartialEq, Eq)]
#[georm(
    table = "genres",
    many_to_many = [{
        name = "books",
        table = "books",
        entity = Book,
        remote_id = "ident",
        link = { table = "book_genres", from = "genre_id", to = "book_id" }
    }]
)]
pub struct Genre {
    #[georm(id)]
    id: i32,
    name: String,
}
