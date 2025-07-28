pub mod moonpay;
pub use self::moonpay::client::MoonPayClient;

pub mod mercuryo;
pub use self::mercuryo::client::MercuryoClient;

pub mod transak;
pub use self::transak::client::TransakClient;

pub mod banxa;
pub use self::banxa::client::BanxaClient;

pub mod paybis;
pub use self::paybis::client::PaybisClient;
