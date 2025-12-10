pub struct Success<T>{
    pub data: T,
}
pub struct Error{
    pub message: String,
    pub code: i32,
}