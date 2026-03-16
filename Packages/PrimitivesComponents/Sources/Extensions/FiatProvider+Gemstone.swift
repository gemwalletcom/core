// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Style
import SwiftUI

public extension FiatProvider {
    var providerType: FiatProviderName? {
        FiatProviderName(rawValue: id)
    }

    var image: Image {
        switch providerType {
        case .moonPay: Images.Fiat.moonpay
        case .transak: Images.Fiat.transak
        case .banxa: Images.Fiat.banxa
        case .mercuryo: Images.Fiat.mercuryo
        case .paybis: Images.Fiat.paybis
        case .flashnet: Images.Fiat.cashapp
        case nil: Images.name(name.lowercased().replacingOccurrences(of: " ", with: "_"))
        }
    }
}
