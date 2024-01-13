use std::ops::Add;

use sha1::digest::{generic_array::ArrayLength, Digest, OutputSizeUser};

pub fn hashsum<T: Digest>(data: &[u8]) -> String
where
    <T as OutputSizeUser>::OutputSize: Add,
    <<T as OutputSizeUser>::OutputSize as Add>::Output: ArrayLength<u8>,
{
    format!("{:x}", T::digest(data))
}
