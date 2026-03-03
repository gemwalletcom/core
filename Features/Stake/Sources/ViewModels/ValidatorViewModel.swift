// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Primitives
import Components
import Style
import Localization
import Formatters
import ExplorerService
import PrimitivesComponents

public struct ValidatorViewModel {

    public let validator: DelegationValidator
    private let imageFormatter = AssetImageFormatter()
    private let exploreService: ExplorerService

    public init(
        validator: DelegationValidator,
        exploreService: ExplorerService = .standard
    ) {
        self.validator = validator
        self.exploreService = exploreService
    }

    public var name: String {
        switch validator.providerType {
        case .stake:
            if validator.name.isEmpty {
                return AddressFormatter(style: .short, address: validator.id, chain: validator.chain).value()
            }
            return validator.name
        case .earn:
            return validator.name
        }
    }

    public var aprModel: AprViewModel {
        AprViewModel(apr: validator.apr)
    }

    public var imageUrl: URL? {
        switch validator.providerType {
        case .stake:
            imageFormatter.getValidatorUrl(chain: validator.chain, id: validator.id)
        case .earn:
            nil
        }
    }

    public var image: Image? {
        switch validator.providerType {
        case .stake:
            nil
        case .earn:
            switch YieldProvider(rawValue: validator.id) {
            case .yo: Images.EarnProviders.yo
            case .none: nil
            }
        }
    }

    public var validatorImage: AssetImage {
        switch validator.providerType {
        case .stake:
            return AssetImage(
                type: String(name.first ?? " "),
                imageURL: imageUrl
            )
        case .earn:
            return AssetImage(placeholder: image)
        }
    }

    public var url: URL? {
        switch validator.providerType {
        case .stake:
            guard validator.id != DelegationValidator.systemId else { return nil }
            return exploreService.validatorUrl(chain: validator.chain, address: validator.id)?.url
        case .earn:
            return nil
        }
    }
}
