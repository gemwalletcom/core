// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension TransferDataType {
    var asset: Asset {
        switch self {
        case .transfer(let asset),
             .deposit(let asset),
             .withdrawal(let asset),
             .swap(let asset, _, _),
             .stake(let asset, _),
             .account(let asset, _),
             .perpetual(let asset, _),
             .earn(let asset, _, _),
             .tokenApprove(let asset, _),
             .generic(let asset, _, _):
            return asset
        case .transferNft(let asset):
            return asset.chain.asset
        }
    }
}
