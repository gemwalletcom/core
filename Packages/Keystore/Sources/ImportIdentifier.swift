// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import WalletCore

internal import struct Formatters.MnemonicFormatter

public enum ImportIdentifier {
    case phrase(words: [String])
    case single(chain: Chain, words: [String])
    case privateKey(chain: Chain, key: String)
    case address(address: String, chain: Chain)

    func walletId() throws -> WalletId {
        let (chain, address) = try deriveAddress()
        switch self {
        case .phrase: return .multicoin(address: address)
        case .single: return .single(chain: chain, address: address)
        case .privateKey: return .privateKey(chain: chain, address: address)
        case .address: return .view(chain: chain, address: address)
        }
    }

    public func deriveAddress() throws -> (Chain, String) {
        switch self {
        case let .phrase(words):
            return try deriveFromMnemonic(words: words, chain: .ethereum)
        case let .single(chain, words):
            return try deriveFromMnemonic(words: words, chain: chain)
        case let .privateKey(chain, key):
            let privateKey = try WalletKeyStore.decodeKey(key, chain: chain)
            let address = chain.coinType.deriveAddress(privateKey: privateKey)
            return (chain, address)
        case let .address(address, chain):
            return (chain, address)
        }
    }

    private func deriveFromMnemonic(words: [String], chain: Chain) throws -> (Chain, String) {
        let mnemonic = MnemonicFormatter.fromArray(words: words)
        guard let wallet = HDWallet(mnemonic: mnemonic, passphrase: "") else {
            throw AnyError("Invalid mnemonic")
        }
        let key = wallet.getKeyForCoin(coin: chain.coinType)
        let address = chain.coinType.deriveAddress(privateKey: key)
        return (chain, address)
    }
}

public extension ImportIdentifier {
    static func from(_ type: KeystoreImportType) -> ImportIdentifier {
        switch type {
        case let .phrase(words, _): .phrase(words: words)
        case let .single(words, chain): .single(chain: chain, words: words)
        case let .privateKey(key, chain): .privateKey(chain: chain, key: key)
        case let .address(address, chain): .address(address: address, chain: chain)
        }
    }
}
