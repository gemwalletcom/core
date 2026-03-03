import Foundation
import GemstonePrimitives
import Primitives
import WalletCore

public final class LocalKeystore: Keystore, @unchecked Sendable {
    private let walletKeyStore: WalletKeyStore
    private let keystorePassword: KeystorePassword
    private let queue = DispatchQueue(label: "com.gemwallet.keystore", qos: .userInitiated)

    public init(
        directory: String = "keystore",
        keystorePassword: KeystorePassword = LocalKeystorePassword()
    ) {
        do {
            // migrate keystore from documents directory to applocation support directory
            // TODO: delete in 2026
            let fileMigrator = FileMigrator()
            let keystoreURL = try fileMigrator.migrate(
                name: directory,
                fromDirectory: .documentDirectory,
                toDirectory: .applicationSupportDirectory,
                isDirectory: true
            )
            self.walletKeyStore = WalletKeyStore(directory: keystoreURL)
        } catch {
            fatalError("keystore initialization error: \(error)")
        }

        self.keystorePassword = keystorePassword
    }

    public func createWallet() throws -> [String] {
        try walletKeyStore.createWallet()
    }

    public func importWallet(
        name: String,
        type: KeystoreImportType,
        isWalletsEmpty: Bool,
        source: WalletSource
    ) async throws -> Primitives.Wallet {
        let password = try await getOrCreatePassword(createPasswordIfNone: isWalletsEmpty)
        let walletId = try ImportIdentifier.from(type).walletId()

        return try await queue.asyncTask { [walletKeyStore] in
            switch type {
            case .phrase(let words, let chains):
                try walletKeyStore.importWallet(id: walletId, name: name, words: words, chains: chains, password: password, source: source)
            case .single(let words, let chain):
                try walletKeyStore.importWallet(id: walletId, name: name, words: words, chains: [chain], password: password, source: source)
            case .privateKey(let text, let chain):
                try walletKeyStore.importPrivateKey(id: walletId, name: name, key: text, chain: chain, password: password, source: source)
            case .address(let address, let chain):
                Wallet.makeView(name: name, chain: chain, address: address)
            }
        }
    }

    public func setupChains(chains: [Chain], for wallets: [Primitives.Wallet]) throws -> [Primitives.Wallet] {
        let filteredWallets = wallets.filter {
            let enabled = Set($0.accounts.map(\.chain)).intersection(chains).map { $0 }
            let missing = Set(chains).subtracting(enabled)
            return missing.isNotEmpty
        }
        guard filteredWallets.isNotEmpty else {
            return []
        }
        let password = try keystorePassword.getPassword()

        return try filteredWallets
            .prefix(25)
            .map {
                let existingChains = $0.accounts.map(\.chain)
                return try walletKeyStore.addChains(
                    wallet: $0,
                    existingChains: existingChains,
                    newChains: chains.asSet().subtracting(existingChains.asSet()).asArray(),
                    password: password
                )
            }
    }

    public func deleteKey(for wallet: Primitives.Wallet) async throws {
        switch wallet.type {
        case .view: break
        case .multicoin, .single, .privateKey:
            let password = try await getPassword()
            try await queue.asyncTask { [walletKeyStore] in
                do {
                    try walletKeyStore.deleteWallet(id: wallet.keystoreId, password: password)
                } catch let error as KeystoreError {
                    // in some cases wallet already deleted, just ignore
                    switch error {
                    case .unknownWalletInWalletCore,
                        .unknownWalletIdInWalletCore,
                        .unknownWalletInWalletCoreList,
                        .invalidPrivateKey,
                        .invalidPrivateKeyEncoding:
                        break
                    @unknown default:
                        throw error
                    }
                }
            }
        }
    }

    public func getPrivateKey(wallet: Primitives.Wallet, chain: Chain) async throws -> Data {
        let password = try await getPassword()
        return try await queue.asyncTask { [walletKeyStore] in
            try walletKeyStore.getPrivateKey(id: wallet.keystoreId, type: wallet.type, chain: chain, password: password)
        }
    }

    public func getPrivateKeyEncoded(wallet: Primitives.Wallet, chain: Chain) async throws -> String {
        var data = try await getPrivateKey(wallet: wallet, chain: chain)
        defer { data.zeroize() }
        switch chain.type {
        case .bitcoin, .solana:
            return Base58.encodeNoCheck(data: data)
        default:
            return data.hexString.append0x
        }
    }

    public func getMnemonic(wallet: Primitives.Wallet) async throws -> [String] {
        let password = try await getPassword()
        return try await queue.asyncTask { [walletKeyStore] in
            try walletKeyStore.getMnemonic(walletId: wallet.keystoreId, password: password)
        }
    }

    public func getPasswordAuthentication() throws -> KeystoreAuthentication {
        try keystorePassword.getAuthentication()
    }

    public func sign(hash: Data, wallet: Primitives.Wallet, chain: Chain) async throws -> Data {
        let password = try await getPassword()
        return try await queue.asyncTask { [walletKeyStore] in
            try walletKeyStore.sign(
                hash: hash,
                walletId: wallet.keystoreId,
                type: wallet.type,
                password: password,
                chain: chain
            )
        }
    }

    public func destroy() throws {
        try walletKeyStore.destroy()
    }

    @MainActor
    private func getPassword() throws -> String {
        try keystorePassword.getPassword()
    }

    @MainActor
    private func getOrCreatePassword(createPasswordIfNone: Bool) throws -> String {
        let password = try keystorePassword.getPassword()

        guard password.isEmpty, createPasswordIfNone else {
            return password
        }
        let newPassword = try SecureRandom.generateKey(length: 32).hex
        try keystorePassword.setPassword(newPassword, authentication: .none)
        return newPassword
    }
}
