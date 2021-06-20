
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ServerState {
    Running,
    Stopping,
    Stopped,
    //Error(ServerError),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ServerError {

}