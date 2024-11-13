#[derive(Default, Debug)]
pub struct WithId<T> {
    pub id: String,
    pub obj: T,
}

impl<T> WithId<T> {
    pub fn new(id: String, obj: T) -> Self {
        WithId { id, obj }
    }
}
