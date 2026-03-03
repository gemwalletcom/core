// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import GemstonePrimitives
import Localization
import PrimitivesComponents
import WalletService
import Components
import Style
import NameService
import Keystore
import NodeService
import SwiftUI
import ScanService
import Formatters
import Store

public typealias RecipientDataAction = ((RecipientData) -> Void)?

@Observable
@MainActor
public final class RecipientSceneViewModel {
    public let wallet: Wallet
    public let asset: Asset
    let type: RecipientAssetType

    public let onTransferAction: TransferDataAction

    private let walletService: WalletService
    private let onRecipientDataAction: RecipientDataAction
    private let formatter = ValueFormatter(style: .full)

    public var isPresentingScanner: RecipientScene.Field?
    var addressInputModel: AddressInputViewModel
    var memo: String = ""
    var amount: String = ""

    public let contactsQuery: ObservableQuery<ContactsRequest>
    var contacts: [ContactData] { contactsQuery.value }

    public init(
        wallet: Wallet,
        asset: Asset,
        walletService: WalletService,
        nameService: any NameServiceable,
        type: RecipientAssetType,
        onRecipientDataAction: RecipientDataAction,
        onTransferAction: TransferDataAction
    ) {
        self.wallet = wallet
        self.asset = asset
        self.walletService = walletService
        self.type = type
        self.onRecipientDataAction = onRecipientDataAction
        self.onTransferAction = onTransferAction

        self.addressInputModel = AddressInputViewModel(
            chain: asset.chain,
            nameService: nameService,
            placeholder: recipientField,
            validators: [
                .required(requireName: recipientField),
                .address(asset)
            ]
        )

        self.contactsQuery = ObservableQuery(ContactsRequest(chain: asset.chain), initialValue: [])
    }

    var tittle: String { Localized.Transfer.Recipient.title }
    let recipientField = Localized.Transfer.Recipient.addressField
    var memoField: String { Localized.Transfer.memo }

    var assetModel: AssetViewModel { AssetViewModel(asset: asset) }

    var actionButtonTitle: String { Localized.Common.continue }
    var actionButtonState: ButtonState {
        addressInputModel.isValid ? .normal : .disabled
    }

    var showMemo: Bool { asset.chain.isMemoSupported }
    var chain: Chain { asset.chain }

    var recipientSections: [ListItemValueSection<RecipientAddress>] {
        RecipientAddressType.allCases
            .map {
                ListItemValueSection(
                    section: sectionTitle(for: $0),
                    image: sectionImage(for: $0),
                    values: sectionRecipients(for: $0)
                )
            }
            .filter({ $0.values.isNotEmpty })
    }
}

// MARK: - Actions

extension RecipientSceneViewModel {
    func onContinue() {
        guard addressInputModel.validate() else { return }

        handle(
            recipientData: makeRecipientData(
                name: addressInputModel.nameResolveState.result,
                address: addressInputModel.text,
                memo: memo,
                amount: amount.isEmpty ? .none : amount
            )
        )
    }

    func onSelectScan(field: RecipientScene.Field) {
        isPresentingScanner = field
    }

    public func onHandleScan(_ result: String, for field: RecipientScene.Field) {
        switch field {
        case .address:
            do {
                try handleAddressScan(result)
            } catch {
                addressInputModel.update(error: error)
            }

        case .memo:
            memo = result
        }
    }

    func onChangeAddressText(_: String, new: String) {
        if !amount.isEmpty {
            amount = .empty
        }
    }

    func onSelectRecipient(_ recipient: RecipientAddress) {
        handle(
            recipientData: makeRecipientData(recipient: recipient)
        )
    }
}

// MARK: - Private

extension RecipientSceneViewModel {
    private func makeRecipientData(name: NameRecord?, address: String, memo: String?, amount: String?) -> RecipientData {
        let recipient: Recipient = {
            if let result = name {
                return Recipient(name: result.name, address: result.address, memo: memo)
            }
            return Recipient(name: .none, address: address, memo: memo)
        }()

        return RecipientData(
            recipient: recipient,
            amount: amount
        )
    }

    private func makeRecipientData(recipient: RecipientAddress) -> RecipientData {
        RecipientData(
            recipient: Recipient(
                name: recipient.name,
                address: recipient.address,
                memo: recipient.memo
            ),
            amount: .none
        )
    }

    //TODO: Add unit tests, will be added once moved to package
    private func paymentScan(string: String) throws -> PaymentScanResult {
        let payment = try PaymentURLDecoder.decode(string)

        return PaymentScanResult(
            address: payment.address,
            amount: payment.amount,
            memo: payment.memo
        )
    }

     func getRecipientScanResult(payment: PaymentScanResult) throws -> RecipientScanResult {
        if let amount = payment.amount, (showMemo ? ((payment.memo?.isEmpty) == nil) : true),
           asset.chain.isValidAddress(payment.address)
        {
            let transferType: TransferDataType = switch type {
            case .asset(let asset): .transfer(asset)
            case .nft(let asset): .transferNft(asset)
            }

            let value = try formatter.inputNumber(from: amount, decimals: asset.decimals.asInt)
            let recipientData = RecipientData(
                recipient: Recipient(
                    name: .none,
                    address: payment.address,
                    memo: payment.memo
                ),
                amount: .none
            )
            return .transferData(
                TransferData(type: transferType, recipientData: recipientData, value: value, canChangeValue: false)
            )
        }

        return .recipient(address: payment.address, memo: payment.memo, amount: payment.amount)
    }

    private func sectionRecipients(for section: RecipientAddressType) -> [ListItemValue<RecipientAddress>] {
        switch section {
        case .contacts:
            ContactRecipientSectionViewModel(contacts: contacts).listItems
        case .pinned, .wallets, .view:
            WalletRecipientSectionViewModel(
                wallets: walletService.wallets.filter { $0.id != wallet.id },
                section: section,
                chain: asset.chain
            ).listItems
        }
    }

    private func sectionTitle(for type: RecipientAddressType) -> String {
        switch type {
        case .pinned: Localized.Common.pinned
        case .contacts: Localized.Contacts.title
        case .wallets: Localized.Transfer.Recipient.myWallets
        case .view: Localized.Transfer.Recipient.viewWallets
        }
    }

    private func sectionImage(for type: RecipientAddressType) -> Image {
        switch type {
        case .pinned: Images.System.pin
        case .contacts: Images.System.person
        case .wallets: Images.System.wallet
        case .view: Images.System.eye
        }
    }

    private func handleAddressScan(_ string: String) throws {
        let payment = try paymentScan(string: string)
        let scanResult = try getRecipientScanResult(payment: payment)
        switch scanResult {
        case .transferData(let data):
            handle(transferData: data)
        case .recipient(let address, let memo, let amount):
            // TODO: - open if all fields filled
            addressInputModel.update(text: address)

            if let memo = memo { self.memo = memo }
            if let amount = amount { self.amount = amount }
        }
    }

    private func handle(recipientData: RecipientData) {
        switch type {
        case .asset:
            onRecipientDataAction?(recipientData)
        case .nft(let asset):
            handle(transferData: TransferData(type: .transferNft(asset), recipientData: recipientData, value: .zero, canChangeValue: true))
        }
    }

    private func handle(transferData: TransferData) {
        onTransferAction?(transferData)
    }
}
