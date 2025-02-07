use alloy_core::sol;
use alloy_sol_types::SolCall;
use primitives::EVMChain;

// https://www.multicall3.com/
sol! {
    interface IMulticall3 {
        struct Call {
          address target;
          bytes callData;
        }

        struct Call3 {
          address target;
          bool allowFailure;
          bytes callData;
        }

        struct Call3Value {
          address target;
          bool allowFailure;
          uint256 value;
          bytes callData;
        }

        struct Result {
          bool success;
          bytes returnData;
        }

        function aggregate(Call[] calldata calls)
          external
          payable
          returns (uint256 blockNumber, bytes[] memory returnData);

        function aggregate3(Call3[] calldata calls) external payable returns (Result[] memory returnData);

        function aggregate3Value(Call3Value[] calldata calls)
          external
          payable
          returns (Result[] memory returnData);

        function tryAggregate(bool requireSuccess, Call[] calldata calls)
          external
          payable
          returns (Result[] memory returnData);
    }
}

pub fn create_call3(target: &str, call: impl SolCall) -> IMulticall3::Call3 {
    IMulticall3::Call3 {
        target: target.parse().unwrap(),
        allowFailure: true,
        callData: call.abi_encode().into(),
    }
}

pub fn decode_call3_return<T: SolCall>(result: &IMulticall3::Result) -> Result<T::Return, anyhow::Error> {
    if result.success {
        let decoded = T::abi_decode_returns(&result.returnData, true).map_err(|e| anyhow::anyhow!("{:?} abi decode error: {:?}", T::SIGNATURE, e))?;
        Ok(decoded)
    } else {
        Err(anyhow::anyhow!(format!("{:?} failed", T::SIGNATURE)))
    }
}

pub fn deployment_by_chain(chain: &EVMChain) -> &'static str {
    match chain {
        EVMChain::Ethereum
        | EVMChain::Base
        | EVMChain::Optimism
        | EVMChain::Arbitrum
        | EVMChain::AvalancheC
        | EVMChain::Fantom
        | EVMChain::SmartChain
        | EVMChain::Polygon
        | EVMChain::OpBNB
        | EVMChain::Gnosis
        | EVMChain::Manta
        | EVMChain::Blast
        | EVMChain::Linea
        | EVMChain::Mantle
        | EVMChain::Celo
        | EVMChain::World
        | EVMChain::Sonic
        | EVMChain::Unichain => "0xcA11bde05977b3631167028862bE2a173976CA11",
        EVMChain::ZkSync | EVMChain::Abstract => "0xF9cda624FBC7e059355ce98a31693d299FACd963",
    }
}
