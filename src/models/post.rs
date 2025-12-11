use serde::Serialize;
#[derive(Default)]
pub struct PostCreate {
    pub title: String,
    pub tags: Vec<String>,
    pub content: Vec<u8>,
}
#[derive(Serialize)]
pub struct PostId {
    pub id: i32,
}
impl PostId {
    pub fn new(id: i32) -> Self {
        return Self { id };
    }
}
