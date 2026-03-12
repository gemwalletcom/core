// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.Config
import class Gemstone.Explorer
import Primitives
import GemstonePrimitives
import Preferences

public struct ExplorerService {
    private let preferences: any ExplorerPreferencesStorable

    public static let standard: ExplorerService = ExplorerService()

    public init(preferences: any ExplorerPreferencesStorable = ExplorerPreferences()) {
        self.preferences = preferences
    }

    private func explorerNameOrDefault(chain: Chain) -> String {
        let name = preferences.get(chain: chain)
        let explorers = Self.explorers(chain: chain)
        return explorers.first(where: { $0 == name }) ?? explorers.first!
    }

    public static func explorers(chain: Chain) -> [String] {
        Gemstone.Config.shared.getBlockExplorers(chain: chain.id)
    }

    public func transactionUrl(chain: Chain, hash: String) -> BlockExplorerLink {
        let name = explorerNameOrDefault(chain: chain)
        let explorer = Gemstone.Explorer(chain: chain.id)
        return makeLink(name: name, url: explorer.getTransactionUrl(explorerName: name, transactionId: hash))!
    }

    public func swapTransactionUrl(chain: Chain, provider: String, identifier: String) -> BlockExplorerLink? {
        let name = explorerNameOrDefault(chain: chain)
        let explorer = Gemstone.Explorer(chain: chain.id)
        guard let url = explorer.getTransactionSwapUrl(
            explorerName: name,
            transactionId: identifier,
            providerId: provider
        ) else {
            return nil
        }
        return BlockExplorerLink(name: url.name, link: url.url)
    }

    public func addressUrl(chain: Chain, address: String) -> BlockExplorerLink {
        let name = explorerNameOrDefault(chain: chain)
        let explorer = Gemstone.Explorer(chain: chain.id)
        return makeLink(name: name, url: explorer.getAddressUrl(explorerName: name, address: address))!
    }

    public func tokenUrl(chain: Chain, address: String) -> BlockExplorerLink? {
        let name = explorerNameOrDefault(chain: chain)
        let explorer = Gemstone.Explorer(chain: chain.id)
        return makeLink(name: name, url: explorer.getTokenUrl(explorerName: name, address: address))
    }

    public func validatorUrl(chain: Chain, address: String) -> BlockExplorerLink? {
        let name = explorerNameOrDefault(chain: chain)
        let explorer = Gemstone.Explorer(chain: chain.id)
        return makeLink(name: name, url: explorer.getValidatorUrl(explorerName: name, address: address))
    }

    private func makeLink(name: String, url: String?) -> BlockExplorerLink? {
        guard let url, let parsed = URL(string: url) else { return nil }
        return BlockExplorerLink(name: name, link: parsed.absoluteString)
    }
}

// MARK: - ExplorerStorable

extension ExplorerService: ExplorerPreferencesStorable {
    public func set(chain: Chain, name: String) {
        preferences.set(chain: chain, name: name)
    }

    public func get(chain: Chain) -> String? {
        preferences.get(chain: chain)
    }
}

// MARK: - ExplorerLinkFetchable

extension ExplorerService: ExplorerLinkFetchable { }
