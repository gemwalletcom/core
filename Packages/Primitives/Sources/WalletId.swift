// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum WalletId: Equatable, Hashable, Sendable {
    case multicoin(address: String)
    case single(chain: Chain, address: String)
    case privateKey(chain: Chain, address: String)
    case view(chain: Chain, address: String)

    public var id: String {
        switch self {
        case let .multicoin(address):
            return "\(walletType.rawValue)_\(address)"
        case let .single(chain, address),
             let .privateKey(chain, address),
             let .view(chain, address):
            return "\(walletType.rawValue)_\(chain.rawValue)_\(address)"
        }
    }

    public var walletType: WalletType {
        switch self {
        case .multicoin: .multicoin
        case .single: .single
        case .privateKey: .privateKey
        case .view: .view
        }
    }

    public var address: String {
        switch self {
        case let .multicoin(address),
             let .single(_, address),
             let .privateKey(_, address),
             let .view(_, address):
            return address
        }
    }

    public var chain: Chain? {
        switch self {
        case .multicoin:
            return nil
        case let .single(chain, _),
             let .privateKey(chain, _),
             let .view(chain, _):
            return chain
        }
    }

    public static func from(id: String) throws -> WalletId {
        let parts = id.split(separator: "_", maxSplits: 2).map(String.init)
        guard parts.count >= 2 else {
            throw AnyError("invalid wallet id format: expected at least 2 parts separated by '_', got: \(id)")
        }
        guard let walletType = WalletType(rawValue: parts[0]) else {
            throw AnyError("invalid wallet type: \(parts[0])")
        }

        switch walletType {
        case .multicoin:
            let address = parts.dropFirst().joined(separator: "_")
            return .multicoin(address: address)
        case .single, .privateKey, .view:
            guard parts.count == 3 else {
                throw AnyError("invalid wallet id format for \(walletType): expected 3 parts, got \(parts.count) in \(id)")
            }
            guard let chain = Chain(rawValue: parts[1]) else {
                throw AnyError("invalid chain: \(parts[1])")
            }
            return Self.make(walletType: walletType, chain: chain, address: parts[2])
        }
    }

    public static func make(walletType: WalletType, chain: Chain, address: String) -> WalletId {
        switch walletType {
        case .multicoin: .multicoin(address: address)
        case .single: .single(chain: chain, address: address)
        case .privateKey: .privateKey(chain: chain, address: address)
        case .view: .view(chain: chain, address: address)
        }
    }

    public static func from(type: WalletType, accounts: [Account]) throws -> WalletId {
        switch type {
        case .multicoin:
            guard let address = accounts.first(where: { $0.chain == .ethereum })?.address else {
                throw AnyError("multicoin wallet requires an ethereum account")
            }
            return .multicoin(address: address)
        case .single, .privateKey, .view:
            guard let account = accounts.first else {
                throw AnyError("\(type) wallet requires at least one account")
            }
            return make(walletType: type, chain: account.chain, address: account.address)
        }
    }
}

extension WalletId: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let string = try container.decode(String.self)
        self = try Self.from(id: string)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(id)
    }
}
