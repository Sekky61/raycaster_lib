#[derive(PartialEq, Eq)]
pub struct BufferStatus(pub bool, pub bool);

impl BufferStatus {
    pub fn new() -> BufferStatus {
        BufferStatus(false, false)
    }
}
