// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Foundation
import BigInt
import Primitives
import PrimitivesTestKit

import PrimitivesComponentsTestKit
@testable import PrimitivesComponents

struct BalanceViewModelTests {

    @Test
    func total() {
        let tronBalance = Balance(
            available: BigInt(1),
            frozen: BigInt(1_000_000),
            locked: BigInt(0),
            staked: BigInt(0),
            pending: BigInt(0),
            rewards: BigInt(5_496),
            metadata: BalanceMetadata(
                votes: 1,
                energyAvailable: .zero,
                energyTotal: .zero,
                bandwidthAvailable: .zero,
                bandwidthTotal: .zero
            )
        )
        let tronModel = BalanceViewModel(asset: .mockTron(), balance: tronBalance, formatter: .auto)

        let bnbBalance = Balance(
            available: BigInt(1_000_000),
            frozen: BigInt(0),
            locked: BigInt(0),
            staked: BigInt(5_000_000),
            pending: BigInt(0),
            rewards: BigInt(100_000)
        )
        let bnbModel = BalanceViewModel(asset: .mockBNB(), balance: bnbBalance, formatter: .auto)

        #expect(tronModel.total == BigInt(1_005_497))
        #expect(bnbModel.total == BigInt(6_100_000))
    }

    @Test
    func balanceTextWithSymbol() {
        let model = BalanceViewModel.mock(
            asset: .mockEthereum(),
            balance: .mock(staked: BigInt(1_000_000_000_000_000_000), earn: BigInt(2_000_000_000_000_000_000)),
            formatter: .medium
        )
        #expect(model.balanceTextWithSymbol(for: .stake) == "1.00 ETH")
        #expect(model.balanceTextWithSymbol(for: .earn) == "2.00 ETH")
    }

    @Test
    func hasStakingResources() {
        #expect(BalanceViewModel.mock(asset: .mockTron()).hasStakingResources == true)
        #expect(BalanceViewModel.mock(asset: .mockTronUSDT()).hasStakingResources == true)
        #expect(BalanceViewModel.mock(asset: .mockEthereum()).hasStakingResources == false)
        #expect(BalanceViewModel.mock(asset: .mockBNB()).hasStakingResources == false)
    }
}
