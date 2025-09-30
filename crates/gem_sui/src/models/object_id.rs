use sui_types::Address;

pub struct ObjectId(pub Address);

impl From<u8> for ObjectId {
    fn from(byte: u8) -> Self {
        let mut bytes = [0u8; 32];
        bytes[31] = byte;
        Self(Address::new(bytes))
    }
}

impl From<ObjectId> for Address {
    fn from(val: ObjectId) -> Self {
        val.0
    }
}
