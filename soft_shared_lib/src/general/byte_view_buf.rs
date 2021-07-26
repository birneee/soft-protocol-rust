use crate::general::byte_view::ByteView;
use std::ops::{Deref, DerefMut};
use std::convert::TryFrom;
use crate::error::ErrorType;
use crate::error::Result;
use std::marker::PhantomData;

/// An owned ByteView
pub struct ByteViewBuf<T: ByteView + ?Sized> {
    buf: Vec<u8>,
    phantom: PhantomData<T>
}

impl<T: ByteView + ?Sized> TryFrom<Vec<u8>> for ByteViewBuf<T> {
    type Error = ErrorType;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        T::try_from_buf(&value)?; //validate type
        Ok(ByteViewBuf {
            buf: value,
            phantom: PhantomData::default()
        })
    }
}

impl<T: ByteView + ?Sized> Into<Vec<u8>> for ByteViewBuf<T> {
    fn into(self) -> Vec<u8> {
        self.buf
    }
}

impl<T: ByteView + ?Sized> Deref for ByteViewBuf<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        T::try_from_buf(&self.buf).unwrap()
    }
}

impl<T: ByteView + ?Sized> DerefMut for ByteViewBuf<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        T::try_from_buf_mut(&mut self.buf).unwrap()
    }
}