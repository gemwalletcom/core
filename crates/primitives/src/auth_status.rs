use strum::AsRefStr;

#[derive(AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum AuthStatus {
    Valid,
    Invalid,
}
