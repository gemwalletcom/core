// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Primitives
import Transfer
import Keystore
import SwapService
import Swap
import ScanService
import WalletConnector
import WalletService
import ChainService
import StakeService
import EarnService
import Stake
import NameService
import BalanceService
import PriceService
import TransactionStateService
import Assets
import FiatConnect
import WalletConnectorService
import AddressNameService
import ActivityService
import EventPresenterService
import Preferences
import PrimitivesComponents
import GemAPI
import AssetsService

public struct ViewModelFactory: Sendable {
    let keystore: any Keystore
    let chainServiceFactory: ChainServiceFactory
    let scanService: ScanService
    let swapService: SwapService
    let assetsEnabler: any AssetsEnabler
    let priceUpdater: any PriceUpdater
    let walletService: WalletService
    let stakeService: StakeService
    let earnService: EarnService
    let amountService: AmountService
    let nameService: NameService
    let balanceService: BalanceService
    let priceService: PriceService
    let transactionStateService: TransactionStateService
    let addressNameService: AddressNameService
    let activityService: ActivityService
    let eventPresenterService: EventPresenterService
    let fiatService: any GemAPIFiatService
    let assetsService: AssetsService

    public init(
        keystore: any Keystore,
        chainServiceFactory: ChainServiceFactory,
        scanService: ScanService,
        swapService: SwapService,
        assetsEnabler: any AssetsEnabler,
        priceUpdater: any PriceUpdater,
        walletService: WalletService,
        stakeService: StakeService,
        earnService: EarnService,
        amountService: AmountService,
        nameService: NameService,
        balanceService: BalanceService,
        priceService: PriceService,
        transactionStateService: TransactionStateService,
        addressNameService: AddressNameService,
        activityService: ActivityService,
        eventPresenterService: EventPresenterService,
        fiatService: any GemAPIFiatService,
        assetsService: AssetsService
    ) {
        self.keystore = keystore
        self.chainServiceFactory = chainServiceFactory
        self.scanService = scanService
        self.swapService = swapService
        self.assetsEnabler = assetsEnabler
        self.priceUpdater = priceUpdater
        self.walletService = walletService
        self.stakeService = stakeService
        self.earnService = earnService
        self.amountService = amountService
        self.nameService = nameService
        self.balanceService = balanceService
        self.priceService = priceService
        self.transactionStateService = transactionStateService
        self.addressNameService = addressNameService
        self.activityService = activityService
        self.eventPresenterService = eventPresenterService
        self.fiatService = fiatService
        self.assetsService = assetsService
    }
    
    @MainActor
    public func confirmTransferScene(
        wallet: Wallet,
        data: TransferData,
        confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
        simulation: SimulationResult? = nil,
        onComplete: VoidAction
    ) -> ConfirmTransferSceneViewModel {
        let confirmService = ConfirmServiceFactory.create(
            keystore: keystore,
            chainServiceFactory: chainServiceFactory,
            assetsEnabler: assetsEnabler,
            scanService: scanService,
            balanceService: balanceService,
            assetsService: assetsService,
            priceService: priceService,
            transactionStateService: transactionStateService,
            addressNameService: addressNameService,
            activityService: activityService,
            eventPresenterService: eventPresenterService,
            chain: data.chain
        )
        let simulationService = ConfirmSimulationServiceFactory.create(
            addressNameService: addressNameService,
            assetsService: assetsService
        )

        return ConfirmTransferSceneViewModel(
            wallet: wallet,
            data: data,
            confirmService: confirmService,
            simulationService: simulationService,
            confirmTransferDelegate: confirmTransferDelegate,
            simulation: simulation,
            onComplete: onComplete
        )
    }
    
    @MainActor
    public func recipientScene(
        wallet: Wallet,
        asset: Asset,
        type: RecipientAssetType,
        onRecipientDataAction: RecipientDataAction,
        onTransferAction: TransferDataAction
    ) -> RecipientSceneViewModel {
        RecipientSceneViewModel(
            wallet: wallet,
            asset: asset,
            walletService: walletService,
            nameService: nameService,
            type: type,
            onRecipientDataAction: onRecipientDataAction,
            onTransferAction: onTransferAction
        )
    }
    
    @MainActor
    public func amountScene(
        input: AmountInput,
        wallet: Wallet,
        onTransferAction: TransferDataAction
    ) -> AmountSceneViewModel {
        AmountSceneViewModel(
            input: input,
            wallet: wallet,
            service: amountService,
            onTransferAction: onTransferAction
        )
    }
    
    @MainActor
    public func fiatScene(
        assetAddress: AssetAddress,
        walletId: WalletId,
        type: FiatQuoteType = .buy,
        amount: Int? = nil
    ) -> FiatSceneViewModel {
        FiatSceneViewModel(
            fiatService: fiatService,
            assetAddress: assetAddress,
            walletId: walletId,
            type: type,
            amount: amount
        )
    }
    
    @MainActor
    public func swapScene(
        input: SwapInput,
        onSwap: @escaping (TransferData) -> Void
    ) -> SwapSceneViewModel {
        SwapSceneViewModel(
            input: input,
            balanceUpdater: balanceService,
            priceUpdater: priceUpdater,
            swapQuotesProvider: SwapQuotesProvider(swapService: swapService),
            swapQuoteDataProvider: SwapQuoteDataProvider(keystore: keystore, swapService: swapService),
            onSwap: onSwap
        )
    }

    @MainActor
    public func stakeScene(
        wallet: Wallet,
        chain: Chain
    ) -> StakeSceneViewModel {
        StakeSceneViewModel(
            wallet: wallet,
            chain: StakeChain(rawValue: chain.rawValue)!, // Expected Only StakeChain accepted.
            currencyCode: Preferences.standard.currency,
            stakeService: stakeService
        )
    }

    @MainActor
    public func earnScene(
        wallet: Wallet,
        asset: Asset
    ) -> EarnSceneViewModel {
        EarnSceneViewModel(
            wallet: wallet,
            asset: asset,
            currencyCode: Preferences.standard.currency,
            earnService: earnService
        )
    }

    @MainActor
    public func delegationScene(
        wallet: Wallet,
        delegation: Delegation,
        asset: Asset,
        validators: [DelegationValidator],
        onAmountInputAction: AmountInputAction,
        onTransferAction: TransferDataAction
    ) -> DelegationSceneViewModel {
        DelegationSceneViewModel(
            wallet: wallet,
            model: DelegationViewModel(delegation: delegation, asset: asset, formatter: .auto, currencyCode: Preferences.standard.currency),
            asset: asset,
            validators: validators,
            onAmountInputAction: onAmountInputAction,
            onTransferAction: onTransferAction
        )
    }

    @MainActor
    public func signMessageScene(
        payload: SignMessagePayload,
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate
    ) -> SignMessageSceneViewModel {
        SignMessageSceneViewModel(
            keystore: keystore,
            addressNameService: addressNameService,
            payload: payload,
            confirmTransferDelegate: confirmTransferDelegate
        )
    }
    
}
