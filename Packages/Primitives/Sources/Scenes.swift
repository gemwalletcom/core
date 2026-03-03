// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct Scenes {
    public struct CreateWallet: Hashable, Codable {
        public init() {}
    }

    public struct ImportWallet: Hashable, Codable {
        public init() {}
    }
    
    public struct ImportWalletType: Hashable, Codable {
        public init() {}
    }
    
    public struct SecurityReminder: Hashable, Codable {
        public init() {}
    }

    public struct Notifications: Hashable, Codable {
        public init() {}
    }

    public struct InAppNotifications: Hashable, Codable {
        public init() {}
    }

    public struct PriceAlerts: Hashable, Codable {
        public init() {}
    }

    public struct Chains: Hashable, Codable {
        public init() {}
    }

    public struct AboutUs: Hashable, Codable {
        public init() {}
    }

    public struct Developer: Hashable, Codable {
        public init() {}
    }

    public struct Security: Hashable, Codable {
        public init() {}
    }

    public struct Currency: Hashable, Codable {
        public init() {}
    }

    public struct Preferences: Hashable, Codable {
        public init() {}
    }

    public struct AppIcon: Hashable, Codable {
        public init() {}
    }

    public struct WalletConnect: Hashable, Codable {
        public init() {}
    }

    public struct SelectWallet: Hashable, Codable {
        public init() {}
    }

    public struct NetworksSelector: Hashable, Codable {
        public init() {}
    }

    public struct VerifyPhrase: Hashable, Codable {
        public let words: [String]

        public init(words: [String]) {
            self.words = words
        }
    }

    public struct WalletProfile: Hashable, Codable {
        public let wallet: Wallet

        public init(wallet: Wallet) {
            self.wallet = wallet
        }
    }

    public struct WalletDetail: Hashable, Codable {
        public let wallet: Wallet

        public init(wallet: Wallet) {
            self.wallet = wallet
        }
    }

    public struct WalletSelectImage: Hashable, Codable {
        public let wallet: Wallet

        public init(wallet: Wallet) {
            self.wallet = wallet
        }
    }

    public struct Price: Hashable, Codable {
        public let asset: Primitives.Asset

        public init(asset: Primitives.Asset) {
            self.asset = asset
        }
    }

    public struct Asset: Hashable, Codable {
        public let asset: Primitives.Asset

        public init(asset: Primitives.Asset) {
            self.asset = asset
        }
    }

    public struct ChainSettings: Hashable, Codable {
        public let chain: Primitives.Chain

        public init(chain: Primitives.Chain) {
            self.chain = chain
        }
    }
    
    public struct Collection: Hashable, Codable, Sendable {
        public let id: String
        public let name: String

        public init(id: String, name: String) {
            self.id = id
            self.name = name
        }
    }

    public struct UnverifiedCollections: Hashable, Codable, Sendable {
        public init() {}
    }

    public struct Collectible: Hashable, Codable, Sendable {
        public let assetData: NFTAssetData

        public init(assetData: NFTAssetData) {
            self.assetData = assetData
        }
    }
    
    public struct Perpetuals: Hashable, Codable {
        public init() {}
    }

    public struct AssetsResults: Hashable, Codable {
        public let searchQuery: String
        public let tag: String?

        public init(searchQuery: String, tag: String?) {
            self.searchQuery = searchQuery
            self.tag = tag
        }
    }

    public struct Referral: Hashable, Codable {
        public let code: String?
        public let giftCode: String?

        public init(code: String? = nil, giftCode: String? = nil) {
            self.code = code
            self.giftCode = giftCode
        }
    }
    
    public struct Perpetual: Hashable, Codable {
        public let asset: Primitives.Asset

        public init(_ asset: Primitives.Asset) {
            self.asset = asset
        }

        public init(_ perpetualData: PerpetualData) {
            self.asset = perpetualData.asset
        }
    }
    
    public struct Transaction: Hashable, Codable {
        public let transaction: TransactionExtended

        public init(transaction: TransactionExtended) {
            self.transaction = transaction
        }
    }

    public struct AssetPriceAlert: Hashable, Codable {
        public let asset: Primitives.Asset
        public let price: Double?

        public init(asset: Primitives.Asset, price: Double? = nil) {
            self.asset = asset
            self.price = price
        }
    }

    public struct Contacts: Hashable, Codable {
        public init() {}
    }

    public struct Contact: Hashable, Codable {
        public let contact: ContactData

        public init(contact: ContactData) {
            self.contact = contact
        }
    }

    public struct ContactAddress: Hashable, Codable {
        public let address: Primitives.ContactAddress

        public init(address: Primitives.ContactAddress) {
            self.address = address
        }
    }
}
