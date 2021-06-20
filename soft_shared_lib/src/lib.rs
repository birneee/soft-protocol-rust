pub mod constants;
pub mod packet;
pub mod error;
pub mod soft_error_code;
pub mod packet_view;

#[macro_use]
extern crate num_derive;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
