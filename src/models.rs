mod response;
pub use response::*;
pub mod post;

#[derive(serde::Deserialize)]
pub struct Pagenigation {
    pub cursor: Option<i32>,
    #[serde(default = "default_page_size")]
    pub page_size: i32,
}
fn default_page_size() -> i32 {
    8
}
