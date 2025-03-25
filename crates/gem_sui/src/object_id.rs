use sui_types::{Address, ObjectId};

pub struct ObjectID {
    bytes: [u8; 32],
}

impl ObjectID {
    pub fn from(byte: u8) -> Self {
        let mut bytes = [0u8; 32];
        bytes[31] = byte;
        Self { bytes }
    }

    pub fn addr(&self) -> Address {
        Address::new(self.bytes)
    }

    pub fn id(&self) -> ObjectId {
        ObjectId::new(self.bytes)
    }
}

impl From<u8> for ObjectID {
    fn from(byte: u8) -> Self {
        ObjectID::from(byte)
    }
}
