use strum::AsRefStr;

pub const X_AUTH_STATUS: &str = "X-Auth-Status";

#[derive(AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum AuthStatus {
    Valid,
    Invalid,
}
