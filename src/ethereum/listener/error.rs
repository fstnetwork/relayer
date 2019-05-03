#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Timer error: {}", _0)]
    Timer(tokio_timer::Error),

    #[fail(display = "Invalid interval value: {:?}", _0)]
    InvalidInterval(std::time::Duration),
}

impl From<tokio_timer::Error> for Error {
    fn from(error: tokio_timer::Error) -> Error {
        Error::Timer(error)
    }
}
