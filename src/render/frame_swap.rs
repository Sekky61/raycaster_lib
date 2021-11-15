#[derive(PartialEq, Eq)]
pub struct BufferStatus(pub bool, pub bool);

impl BufferStatus {
    pub fn new() -> BufferStatus {
        BufferStatus(false, false)
    }

    pub fn get_render_target(&self) -> i32 {
        match self {
            BufferStatus(false, false) => 1,
            BufferStatus(true, false) => 2,
            BufferStatus(false, true) => 1,
            BufferStatus(true, true) => 1, // tu
        }
    }

    pub fn get_finished_target(&self) -> i32 {
        match self {
            BufferStatus(false, false) => 0,
            BufferStatus(true, false) => 1,
            BufferStatus(false, true) => 2,
            BufferStatus(true, true) => 1, // todo
        }
    }
}
