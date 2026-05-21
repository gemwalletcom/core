use sui_types::Address as SdkAddress;

use gem_encoding::protobuf::{proto_decode, proto_encode};

// Mirrors https://github.com/MystenLabs/sui-apis/blob/main/proto/sui/rpc/v2/move_package_service.proto
#[derive(Clone, Debug, Default)]
pub struct GetFunctionRequest {
    pub package_id: Option<String>,
    pub module_name: Option<String>,
    pub name: Option<String>,
}

impl GetFunctionRequest {
    pub fn new(package_id: &SdkAddress, module: &str, name: &str) -> Self {
        Self {
            package_id: Some(package_id.to_string()),
            module_name: Some(module.to_string()),
            name: Some(name.to_string()),
        }
    }
}

proto_encode!(GetFunctionRequest {
    1 => package_id: optional_string,
    2 => module_name: optional_string,
    3 => name: optional_string,
});

#[derive(Clone, Debug, Default)]
pub struct GetFunctionResponse {
    pub function: Option<FunctionDescriptor>,
}

proto_decode!(GetFunctionResponse {
    1 => function: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct FunctionDescriptor {
    pub parameters: Vec<OpenSignature>,
}

proto_decode!(FunctionDescriptor {
    8 => parameters: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct OpenSignature {
    pub reference: Option<i32>,
}

proto_decode!(OpenSignature {
    1 => reference: optional_varint_i32,
});

pub mod open_signature {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum Reference {
        Unknown = 0,
        Immutable = 1,
        Mutable = 2,
    }

    impl TryFrom<i32> for Reference {
        type Error = i32;

        fn try_from(value: i32) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(Self::Unknown),
                1 => Ok(Self::Immutable),
                2 => Ok(Self::Mutable),
                _ => Err(value),
            }
        }
    }
}
