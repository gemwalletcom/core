// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.SignMessage
import class Gemstone.MessageSigner
import Testing

struct MessageSignerTests {
    @Test
    func base58Preview() throws {
        let data = "X3CUgCGzyn43DTAbUKnTMDzcGWMooJT2hPSZinjfN1QUgVNYYfeoJ5zg6i4Nd5coKGUrNpEYVoD".data(using: .utf8)!
        let signer = MessageSigner(message: SignMessage(chain: "solana", signType: .base58, data: data))

        #expect(try signer.hash().encodeString() == "This is an example message to be signed - 1747125759060")
        #expect(signer.plainPreview() == "This is an example message to be signed - 1747125759060")
        #expect(try signer.payloadPreview(simulationPayload: []) == nil)
    }

    @Test
    func eip191SiweUsesPayloadPreview() throws {
        let message = """
        thepoc.xyz wants you to sign in with your Ethereum account:
        0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4

        Sign in with different chain ID

        URI: https://thepoc.xyz
        Version: 1
        Chain ID: 1
        Nonce: byjof9dwrao97skautdxhb
        Issued At: 2026-03-09T15:48:34.458Z
        """

        let signer = MessageSigner(
            message: SignMessage(chain: "ethereum", signType: .eip191, data: Data(message.utf8))
        )

        let preview = try signer.payloadPreview(simulationPayload: [])

        #expect(preview?.primary.map(\.value) == ["thepoc.xyz", "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4"])
        #expect(preview?.secondary.count == 5)
    }
}
