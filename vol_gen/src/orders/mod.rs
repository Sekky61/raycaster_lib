mod linear_order;
mod z_order;

pub use linear_order::LinearCoordIterator;
pub use z_order::ZCoordIterator;

// Order of samples in resulting array
// Generally, the fastest growing axis is Z axis
#[derive(Debug)]
pub enum SampleOrder {
    // Samples ordered by axis (x,y,z)
    Linear,

    // Samples ordered by blocks
    // Blocks are ordered lineary
    // and data inside blocks is also ordered lineary
    Z(u8), // todo parametrize overlap
}
