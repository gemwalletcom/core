pub type AlienError = gem_swapper::AlienError;

#[uniffi::remote(Enum)]
pub enum AlienError {
    RequestError { msg: String },
    ResponseError { msg: String },
    SigningError { msg: String },
}
