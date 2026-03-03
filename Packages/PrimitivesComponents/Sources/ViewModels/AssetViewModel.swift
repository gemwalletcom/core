import Foundation
import Primitives
import Components

public struct AssetViewModel: Sendable, Identifiable {
    public let asset: Asset

    public init(asset: Asset) {
        self.asset = asset
    }

    public var id: String {
        asset.id.identifier
    }

    public var title: String {
        asset.name == asset.symbol ? asset.name : String(format: "%@ (%@)", asset.name, asset.symbol)
    }

    public var name: String {
        asset.name
    }

    public var symbol: String {
        asset.symbol
    }

    public var subtitleSymbol: String? {
        asset.name == asset.symbol ? nil : asset.symbol
    }

    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: asset.id).assetImage
    }
    
    public var networkAssetImage: AssetImage {
        AssetIdViewModel(assetId: asset.id).networkAssetImage
    }
    
    public var networkName: String {
        asset.chain.asset.name
    }
    
    public var networkFullName: String {
        switch asset.id.type {
        case .native: networkName
        case .token: "\(networkName) (\(asset.type.rawValue))"
        }
    }
}
