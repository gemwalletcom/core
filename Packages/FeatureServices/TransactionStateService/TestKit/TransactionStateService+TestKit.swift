// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives
import TransactionStateService
import Store
import StakeService
import EarnService
import Blockchain
import NFTService
import ChainService
import BalanceService
import NativeProviderService
import StoreTestKit
import StakeServiceTestKit
import NFTServiceTestKit
import ChainServiceTestKit
import BalanceServiceTestKit
import SwapServiceTestKit

public extension TransactionStateService {
    static func mock(
        transactionStore: TransactionStore = .mock(),
        swapper: any GemSwapperProtocol = GemSwapperMock(),
        stakeService: StakeService = .mock(),
        earnService: EarnService = .mock(),
        nftService: NFTService = .mock(),
        chainServiceFactory: any ChainServiceFactorable = ChainServiceFactoryMock()
    ) -> TransactionStateService {
        TransactionStateService(
            transactionStore: transactionStore,
            swapper: swapper,
            stakeService: stakeService,
            earnService: earnService,
            nftService: nftService,
            chainServiceFactory: chainServiceFactory,
            balanceUpdater: .mock()
        )
    }
}

public extension EarnService {
    static func mock(
        store: StakeStore = .mock()
    ) -> EarnService {
        let provider = NativeProvider(url: Constants.apiURL, requestInterceptor: EmptyRequestInterceptor())
        return EarnService(store: store, gatewayService: GatewayService(provider: provider))
    }
}
