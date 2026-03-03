// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
@preconcurrency import WalletCore

internal import struct Formatters.MnemonicFormatter
internal import WalletCorePrimitives
internal import func Gemstone.decodePrivateKey

struct WalletKeyStore: Sendable {
    private let keyStore: WalletCore.KeyStore
    private let directory: URL

    func createWallet() throws -> [String] {
        guard let wallet = HDWallet(strength: 128, passphrase: "") else {
            throw AnyError("Unable to create wallet")
        }
        return wallet.mnemonic.split(separator: " ").map { String($0) }
    }

    init(directory: URL) {
        self.directory = directory
        keyStore = try! WalletCore.KeyStore(keyDirectory: directory)
    }

    func importWallet(
        id: WalletId,
        name: String,
        words: [String],
        chains: [Chain],
        password: String,
        source: WalletSource
    ) throws -> Primitives.Wallet {
        let wallet = try keyStore.import(
            mnemonic: MnemonicFormatter.fromArray(words: words),
            name: name,
            encryptPassword: password,
            coins: []
        )
        return try addCoins(id: id, wallet: wallet, existingChains: [], newChains: chains, password: password, source: source)
    }

    static func decodeKey(_ key: String, chain: Chain) throws -> PrivateKey {
        let bytes = try decodePrivateKey(chain: chain.rawValue, value: key)
        guard let privateKey = PrivateKey(data: bytes) else {
            throw AnyError("Invalid private key")
        }
        return privateKey
    }

    func importPrivateKey(id: WalletId, name: String, key: String, chain: Chain, password: String, source: WalletSource) throws -> Primitives.Wallet {
        let privateKey = try Self.decodeKey(key, chain: chain)
        let wallet = try keyStore.import(privateKey: privateKey, name: name, password: password, coin: chain.coinType)

        let address = chain.coinType.deriveAddress(privateKey: privateKey)
        let account = Primitives.Account(
            chain: chain,
            address: address,
            derivationPath: chain.coinType.derivationPath(), // not applicable
            extendedPublicKey: nil
        )
        return Primitives.Wallet(
            id: id.id,
            externalId: wallet.id,
            name: wallet.key.name,
            index: 0,
            type: id.walletType,
            accounts: [account],
            order: 0,
            isPinned: false,
            imageUrl: nil,
            source: source
        )
    }

    func addCoins(
        id: WalletId,
        wallet: WalletCore.Wallet,
        existingChains: [Chain],
        newChains: [Chain],
        password: String,
        source: WalletSource
    ) throws -> Primitives.Wallet {
        let allChains = existingChains + newChains
        let exclude = [Chain.solana]
        let coins = allChains.filter { !exclude.contains($0) }.map { $0.coinType }.asSet().asArray()
        let existingCoinTypes = existingChains.map({ $0.coinType }).asSet()
        let newCoinTypes = newChains.map({ $0.coinType }).asSet()

        // Tricky wallet core implementation. By default is coins: [], it will create ethereum
        // if single chain, remove all to simplify
        if existingChains.isEmpty && newChains.count == 1 {
            let _ = try keyStore.removeAccounts(wallet: wallet, coins: [.ethereum] + exclude.map { $0.coinType }, password: password)
        }
        if newChains.contains(.solana) && !existingChains.contains(.solana) {
            // By default solana derived a wrong derivation path, need to adjust use a new one
            let _ = try wallet.getAccount(password: password, coin: .solana, derivation: .solanaSolana)
        }
        if newChains.isNotEmpty && newCoinTypes.subtracting(existingCoinTypes).isNotEmpty {
            let _ = try keyStore.addAccounts(wallet: wallet, coins: coins, password: password)
        }

        let accounts = allChains.compactMap { chain in
            wallet.accounts.filter({ $0.coin == chain.coinType }).first?.mapToAccount(chain: chain)
        }

        return Wallet(
            id: id.id,
            externalId: wallet.id,
            name: wallet.key.name,
            index: 0,
            type: id.walletType,
            accounts: accounts,
            order: 0,
            isPinned: false,
            imageUrl: nil,
            source: source
        )
    }

    func addChains(
        wallet: Primitives.Wallet,
        existingChains: [Chain],
        newChains: [Chain],
        password: String
    ) throws -> Primitives.Wallet {
        try addCoins(
            id: try WalletId.from(id: wallet.id),
            wallet: try getWallet(id: wallet.keystoreId),
            existingChains: existingChains,
            newChains: newChains,
            password: password,
            source: wallet.source
        )
    }

    private func getWallet(id: String) throws -> WalletCore.Wallet {
        guard let wallet = keyStore.wallets.filter({ $0.id == id }).first else {
            throw KeystoreError.unknownWalletInWalletCoreList
        }
        return wallet
    }

    func deleteWallet(id: String, password: String) throws {
        let wallet = try getWallet(id: id)
        try keyStore.delete(wallet: wallet, password: password)
    }

    func getPrivateKey(id: String, type: Primitives.WalletType, chain: Chain, password: String) throws -> Data {
        let wallet = try getWallet(id: id)
        switch type {
        case .multicoin, .single:
            guard
                let hdwallet = wallet.key.wallet(password: Data(password.utf8))
            else {
                throw KeystoreError.unknownWalletInWalletCore
            }
            switch chain {
            case .solana:
                return hdwallet.getKeyDerivation(coin: chain.coinType, derivation: .solanaSolana).data
            default:
                return hdwallet.getKeyForCoin(coin: chain.coinType).data
            }
        case .privateKey:
            return try wallet.privateKey(password: password, coin: chain.coinType).data
        case .view:
            throw KeystoreError.invalidPrivateKey
        }
    }

    func getMnemonic(walletId: String, password: String) throws -> [String] {
        let wallet = try getWallet(id: walletId)
        guard
            let hdwallet = wallet.key.wallet(password: Data(password.utf8))
        else {
            throw KeystoreError.unknownWalletInWalletCore
        }

        return MnemonicFormatter.toArray(string: hdwallet.mnemonic)
    }

    func sign(hash: Data, walletId: String, type: Primitives.WalletType, password: String, chain: Chain) throws -> Data {
        var privateKey = try getPrivateKey(id: walletId, type: type, chain: chain, password: password)
        defer {
            privateKey.zeroize()
        }
        guard
            let privateKey = PrivateKey(data: privateKey),
            let signature = privateKey.sign(digest: hash, curve: chain.coinType.curve)
        else {
            throw AnyError("no data signed")
        }
        return signature
    }

    func destroy() throws {
        try keyStore.destroy()
    }
}

extension WalletCore.Wallet {
    var id: String {
        return key.identifier!
    }
}

extension WalletCore.Account {
    func mapToAccount(chain: Chain) -> Primitives.Account {
        return Account(
            chain: chain,
            address: chain.shortAddress(address: address),
            derivationPath: derivationPath,
            extendedPublicKey: extendedPublicKey
        )
    }
}
