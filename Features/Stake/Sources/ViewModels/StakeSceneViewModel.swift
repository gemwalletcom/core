// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store
import Components
import BigInt
import GemstonePrimitives
import SwiftUI
import Localization
import StakeService
import InfoSheet
import PrimitivesComponents
import Formatters

@MainActor
@Observable
public final class StakeSceneViewModel {
    private let stakeService: any StakeServiceable

    private var delegatitonsState: StateViewType<Bool> = .loading
    private let chain: StakeChain

    private let formatter = ValueFormatter(style: .medium)
    private let recommendedValidators = StakeRecommendedValidators()
    private let currencyCode: String

    public let wallet: Wallet
    public let delegationsQuery: ObservableQuery<DelegationsRequest>
    public let validatorsQuery: ObservableQuery<ValidatorsRequest>
    public let assetQuery: ObservableQuery<AssetRequest>

    public var delegations: [Delegation] { delegationsQuery.value }
    public var validators: [DelegationValidator] { validatorsQuery.value }
    public var assetData: AssetData { assetQuery.value }

    public var isPresentingInfoSheet: InfoSheetType? = .none

    public init(
        wallet: Wallet,
        chain: StakeChain,
        currencyCode: String,
        stakeService: any StakeServiceable
    ) {
        self.wallet = wallet
        self.chain = chain
        self.currencyCode = currencyCode
        self.stakeService = stakeService
        self.delegationsQuery = ObservableQuery(DelegationsRequest(walletId: wallet.walletId, assetId: chain.chain.assetId, providerType: .stake), initialValue: [])
        self.validatorsQuery = ObservableQuery(ValidatorsRequest(chain: chain.chain, providerType: .stake), initialValue: [])
        self.assetQuery = ObservableQuery(AssetRequest(walletId: wallet.walletId, assetId: chain.chain.assetId), initialValue: .with(asset: chain.chain.asset))
    }

    public var stakeInfoUrl: URL {
        Docs.url(.staking(chain.map()))
    }

    var title: String { Localized.Transfer.Stake.title }

    var stakeTitle: String { Localized.Transfer.Stake.title }
    var claimRewardsTitle: String { Localized.Transfer.ClaimRewards.title }
    var assetTitle: String { assetModel.title }
    var delegationsTitle: String { Localized.Stake.delegations }

    var stakeAprModel: AprViewModel {
        let apr = (try? stakeService.stakeApr(assetId: chain.chain.assetId)) ?? .zero
        return AprViewModel(apr: apr)
    }

    var resourcesTitle: String { Localized.Asset.resources }

    var energyTitle: String { ResourceViewModel(resource: .energy).title }
    var energyText: String { balanceModel.energyText }

    var bandwidthTitle: String { ResourceViewModel(resource: .bandwidth).title }
    var bandwidthText: String { balanceModel.bandwidthText }

    var freezeTitle: String { Localized.Transfer.Freeze.title }
    var unfreezeTitle: String { Localized.Transfer.Unfreeze.title }

    var lockTimeTitle: String { Localized.Stake.lockTime }
    var lockTimeValue: String {
        let now = Date.now
        let date = now.addingTimeInterval(chain.lockTime)
        return Self.lockTimeFormatter.string(from: now, to: date) ?? .empty
    }
    var lockTimeInfoSheet: InfoSheetType {
        InfoSheetType.stakeLockTime(assetModel.assetImage.placeholder)
    }

    var aprInfoSheet: InfoSheetType {
        InfoSheetType.stakeApr(assetModel.assetImage.placeholder)
    }

    var minAmountTitle: String { Localized.Stake.minimumAmount }
    var minAmountValue: String? {
        guard chain.minAmount != 0 else { return .none }
        return formatter.string(chain.minAmount, decimals: Int(asset.decimals), currency: asset.symbol)
    }

    var delegationsErrorTitle: String { Localized.Errors.errorOccured }
    var delegationsRetryTitle: String { Localized.Common.tryAgain }
    var emptyDelegationsTitle: String { Localized.Stake.noActiveStaking }

    var showManage: Bool {
        wallet.canSign
    }
    
    var recommendedCurrentValidator: DelegationValidator? {
        guard let validatorId = recommendedValidators.randomValidatorId(chain: chain.chain) else { return .none }
        return try? stakeService.getValidator(assetId: asset.id, validatorId: validatorId)
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .stake(symbol: assetModel.symbol))
    }

    func navigationDestination(for delegation: DelegationViewModel) -> any Hashable {
        switch delegation.state {
        case .awaitingWithdrawal:
            TransferData(
                type: .stake(asset, .withdraw(delegation.delegation)),
                recipientData: RecipientData(
                    recipient: Recipient(name: delegation.validatorText, address: delegation.delegation.validator.id, memo: ""),
                    amount: .none
                ),
                value: delegation.delegation.base.balanceValue
            )
        case .active, .pending, .inactive, .activating, .deactivating:
            delegation.delegation
        }
    }

    var delegationsSectionTitle: String {
        guard case .data(let delegations) = delegationsState, delegations.isNotEmpty else {
            return .empty
        }
        return delegationsTitle
    }
    
    var delegationsState: StateViewType<[DelegationViewModel]> {
        let delegationModels = delegations.map { DelegationViewModel(delegation: $0, asset: asset, currencyCode: currencyCode) }

        switch delegatitonsState {
        case .noData: return .noData
        case .loading: return delegationModels.isEmpty ? .loading : .data(delegationModels)
        case .data: return delegationModels.isEmpty ? .noData : .data(delegationModels)
        case .error(let error): return .error(error)
        }
    }
    
    var claimRewardsText: String {
        formatter.string(rewardsValue, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    var canClaimRewards: Bool {
        chain.supportClaimRewards && rewardsValue > 0
    }
    
    var claimRewardsDestination: any Hashable {
        let validators = delegations
            .filter { $0.base.rewardsValue > 0 }
            .map { $0.validator }

        return TransferData(
            type: .stake(chain.chain.asset, .rewards(validators)),
            recipientData: RecipientData(
                recipient: Recipient(name: .none, address: "", memo: .none),
                amount: .none
            ),
            value: rewardsValue
        )
    }

    var stakeDestination: any Hashable {
        destination(
            type: .stake(.stake(
                validators: validators,
                recommended: recommendedCurrentValidator
            ))
        )
    }

    var freezeDestination: any Hashable {
        destination(
            type: .freeze(
                data: FreezeData(
                    freezeType: .freeze,
                    resource: .bandwidth
                )
            )
        )
    }

    var unfreezeDestination: any Hashable {
        destination(
            type: .freeze(
                data: FreezeData(
                    freezeType: .unfreeze,
                    resource: .bandwidth
                )
            )
        )
    }

    var showFreeze: Bool { chain == .tron }
    var showUnfreeze: Bool { balanceModel.hasStakingResources }
    var showStake: Bool {
        if showFreeze {
            return balanceModel.hasStakingResources
        }
        return true
    }
    var isStakeEnabled: Bool { validators.isNotEmpty }

    var showTronResources: Bool {
        balanceModel.hasStakingResources
    }
}

// MARK: - Business Logic

extension StakeSceneViewModel {
    func fetch() async {
        delegatitonsState = .loading
        do {
            let acccount = try wallet.account(for: chain.chain)
            try await stakeService.update(walletId: wallet.walletId, chain: chain.chain, address: acccount.address)
            delegatitonsState = .data(true)
        } catch {
            debugLog("Stake scene fetch error: \(error)")
            delegatitonsState = .error(error)
        }
    }
    
    func onLockTimeInfo() {
        isPresentingInfoSheet = lockTimeInfoSheet
    }

    func onAprInfo() {
        isPresentingInfoSheet = aprInfoSheet
    }
}

// MARK: - Private

extension StakeSceneViewModel {
    private static let lockTimeFormatter: DateComponentsFormatter = {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day]
        formatter.unitsStyle = .full
        return formatter
    }()

    private var assetModel: AssetViewModel {
        AssetViewModel(asset: asset)
    }

    private var asset: Asset {
        chain.chain.asset
    }

    private var balanceModel: BalanceViewModel {
        BalanceViewModel(asset: asset, balance: assetData.balance, formatter: formatter)
    }

    private var rewardsValue: BigInt {
        delegations.map { $0.base.rewardsValue }.reduce(0, +)
    }

    private func destination(type: AmountType) -> any Hashable {
        AmountInput(
            type: type,
            asset: asset
        )
    }
}
