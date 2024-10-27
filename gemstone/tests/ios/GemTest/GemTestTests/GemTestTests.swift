// Copyright (c). Gem Wallet. All rights reserved.

import XCTest
import Gemstone
@testable import GemTest

final class GemTestTests: XCTestCase {


    func testLoadFFI() async throws {
        let result = Gemstone.libVersion()
        XCTAssertFalse(result.isEmpty)
    }

    func testGetExplorerName() {
        let chain = "bitcoin" // Primitive::Chain::Bitcion as_str()
        let explorers = Config().getBlockExplorers(chain: chain)

        XCTAssertEqual(explorers.count, 2)
        XCTAssertEqual(explorers[1], "Mempool")

        let explorer = Explorer(chain: chain)
        let txUrl = explorer.getTransactionUrl(explorerName: explorers[1], transactionId: "813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4")

        XCTAssertEqual(txUrl, "https://mempool.space/tx/813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4")

    }

    func testSplitStake() throws {

        let input = SuiStakeInput(
            sender: "0xe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2",
            validator: "0x61953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab",
            stakeAmount: 1_000_000_000,
            gas: SuiGas(
                budget: 20_000_000,
                price: 750
            ),
            coins: [SuiCoin(
                coinType: "0x2::sui::SUI",
                balance: 2_500_000_000,
                objectRef: SuiObjectRef(
                    objectId: "0x48a4367b1e0b4ad996375296e4471ae8d8df2576f90636b8e84051d97c1a363a",
                    digest: "9JHLrv8f3G3NFfWwWt54oHjdZYXD8VaNoZYXu3fV8pmB",
                    version: 65307031
                )
            )]
        )

        let output = try suiEncodeSplitStake(input: input)

        XCTAssertEqual(output.txData.hexString(), "000003000800ca9a3b0000000001010000000000000000000000000000000000000000000000000000000000000005010000000000000001002061953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab020200010100000000000000000000000000000000000000000000000000000000000000000000030a7375695f73797374656d11726571756573745f6164645f7374616b6500030101000300000000010200e6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c20148a4367b1e0b4ad996375296e4471ae8d8df2576f90636b8e84051d97c1a363a9781e40300000000207b4ad8fc8964fdfbdf1c41f4c2ba993d607ae8ed85d260c500b8f4c5855ce0b6e6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2ee02000000000000002d31010000000000")
        XCTAssertEqual(output.hash.hexString(), "0437f7744c84f9226be8a09a76e9bedf4ca961f1937a354f72d5352266a72795")

    }

    func testConvertHRP() throws {
        let address = "cosmos1fxygpgus4nd5jmfl5j7fh5y8hyy53z8u95dzx7"

        XCTAssertEqual(try cosmosConvertHrp(address: address, hrp: "noble"), "noble1fxygpgus4nd5jmfl5j7fh5y8hyy53z8udhc27s")
    }

    func testDecodingBscDelegations() throws {
        let result = Data(hex:  "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000ee448667ffc3d15ca023a6deef2d0faf084c0716000000000000000000000000773760b0708a5cc369c346993a0c225d8e4043b10000000000000000000000000000000000000000000000000de0b6b3b015a6430000000000000000000000000000000000000000000000000dd62dce1850f388000000000000000000000000ee448667ffc3d15ca023a6deef2d0faf084c0716000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a0000000000000000000000000000000000000000000000000e09ef1d9101a1740000000000000000000000000000000000000000000000000e028d70463b87f8")!

        let delegations = try Gemstone.bscDecodeDelegationsReturn(result: result)

        XCTAssertEqual(delegations.count, 2)
        XCTAssertEqual(delegations[1].validatorAddress, "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A")
        XCTAssertEqual(delegations[1].amount, "1011602501587280244")
    }

    func testGetEvmChainConfig() throws {
        let config = Config().getEvmChainConfig(chain: "zksync")

        XCTAssertFalse(config.isOpstack)        
        XCTAssertEqual(config.swapWhitelistContracts.count, 1)
    }


    func testEncodeJsonRpcRequest() throws {
        let params = ["0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5", "latest"]
        let data = try JSONSerialization.data(withJSONObject: params, options: [])
        let string = String(data: data, encoding: .utf8)!

        let request = JsonRpcRequest(method: "eth_getBalance", params: string, id: 1)
        let encoded = try request.encode()
        let expected = """
        {"id":1,"jsonrpc":"2.0","method":"eth_getBalance","params":["0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5","latest"]}
        """

        XCTAssertEqual(String(data: encoded, encoding: .utf8), expected)
    }

    func testDeocdingJsonRpcResult() throws {
        let response = """
        {"jsonrpc": "2.0","id": 1,"result": "0x21e3bb1a6"}
        """

        let result = try JSONDecoder().decode(JsonRpcResult.self, from: response.data(using: .utf8)!)
        guard
            case .value(let response) = result,
            let data = response.result
        else {
            XCTFail("unexpected response: \(response)")
            return
        }

        XCTAssertEqual(data, "0x21e3bb1a6")
    }

    func testDecodingJsonRpcError() throws {
        let response = """
        {
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": 3,
                "data": "0x08c379a000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000015746f416464726573735f6f75744f66426f756e64730000000000000000000000",
                "message": "execution reverted: revert: toAddress_outOfBounds"
            }
        }
        """

        let result = try JSONDecoder().decode(JsonRpcResult.self, from: response.data(using: .utf8)!)
        guard
            case .error(let error) = result
        else {
            XCTFail("unexpected response: \(response)")
            return
        }

        XCTAssertEqual(error.error, JsonRpcError(code: 3, message: "execution reverted: revert: toAddress_outOfBounds"))
    }
}
