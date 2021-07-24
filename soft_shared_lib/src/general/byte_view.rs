use crate::error::Result;

/// Trait for view types to an underlying buffer
///
/// This is an unsized type, meaning that it must always be used behind a pointer like & or Box. For an owned version of this type, see ByteViewBuf.
pub trait ByteView {
    // get view from buffer
    fn try_from_buf(buf: &[u8]) -> Result<&Self>;
    // get mutable view from buffer
    fn try_from_buf_mut(buf: &mut [u8]) -> Result<&mut Self>;
    /// get the byte buffer of the view
    fn buf(&self) -> &[u8];
    /// get the mutable byte buffer of the view
    fn buf_mut(&mut self) -> &mut [u8];
}