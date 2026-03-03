// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Components
import Localization
import PrimitivesComponents
import ExplorerService

@Observable
public final class ValidatorSelectSceneViewModel {
    
    private let type: ValidatorSelectType
    private let chain: Chain
    public let currentValidator: DelegationValidator?
    private let validators: [DelegationValidator]
    public var selectValidator: ((DelegationValidator) -> Void)?
    private let exploreService: ExplorerService = .standard
    public var isPresentingUrl: URL?
    
    private let recommendedValidators = StakeRecommendedValidators()
    
    public init(
        type: ValidatorSelectType,
        chain: Chain,
        currentValidator: DelegationValidator?,
        validators: [DelegationValidator],
        selectValidator: ((DelegationValidator) -> Void)? = nil
    ) {
        self.type = type
        self.chain = chain
        self.currentValidator = currentValidator
        self.validators = validators
        self.selectValidator = selectValidator
    }
    
    public var title: String {
        return Localized.Stake.validators
    }
    
    public var list: [ListItemValueSection<DelegationValidator>] {
        switch type {
        case .stake:
            let recommeneded = recommendedValidators.validatorsSet(chain: chain)
            return [
                listSection(
                    title: Localized.Common.recommended,
                    validators: validators.filter { recommeneded.contains($0.id) }
                ),
                listSection(
                    title: Localized.Stake.active,
                    validators: validators
                ),
            ].filter { $0.values.isNotEmpty }
        case .unstake:
            return [
                listSection(
                    title: Localized.Stake.active,
                    validators: validators
                )
            ]
        }
    }
    
    public func contextMenu(for validator: DelegationValidator) -> [ContextMenuItemType] {
        guard let explorerLink = exploreService.validatorUrl(chain: validator.chain, address: validator.id) else {
            return []
        }
        return [
            .copy(value: validator.id),
            .url(
                title: Localized.Transaction.viewOn(explorerLink.name),
                onOpen: { [weak self] in
                    self?.isPresentingUrl = explorerLink.url
                }
            )
        ]
    }
    
    public func listSection(title: String, validators: [DelegationValidator]) -> ListItemValueSection<DelegationValidator> {
        ListItemValueSection(
            section: title,
            values: validators.map(listItem)
        )
    }
    
    public func listItem(validator: DelegationValidator) -> ListItemValue<DelegationValidator> {
        let model = ValidatorViewModel(validator: validator)
        return ListItemValue(
            title: model.name,
            subtitle: model.aprModel.text,
            value: validator
        )
    }
}
