// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keystore
import WalletConnectorService
import ExplorerService
import AddressNameService
import Primitives
import PrimitivesComponents
import Localization
import Components
import Style
import class Gemstone.MessageSigner
import GemstonePrimitives

@Observable
@MainActor
public final class SignMessageSceneViewModel {
    private let explorerService: ExplorerService = .standard
    private let keystore: any Keystore
    private let addressNameService: AddressNameService
    private let payload: SignMessagePayload
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate
    private let signer: MessageSigner
    private let plainMessage: String
    public let messageDisplayType: SignMessageDisplayType

    public var isPresentingUrl: URL? = nil
    public var isPresentingPayloadDetails: Bool = false
    private var payloadAddressNames: [ChainAddress: AddressName] = [:]

    public init(
        keystore: any Keystore,
        addressNameService: AddressNameService,
        payload: SignMessagePayload,
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate
    ) {
        self.keystore = keystore
        self.addressNameService = addressNameService
        self.payload = payload
        let signer = MessageSigner(message: payload.message)
        self.signer = signer
        let plainMessage = signer.plainPreview()
        self.plainMessage = plainMessage
        let messageDisplayType: SignMessageDisplayType = {
            guard let payloadPreview = try? signer.payloadPreview(simulationPayload: payload.simulation.payload.map { $0.map() }) else {
                return .text(plainMessage)
            }

            return .payload(
                primary: payloadPreview.primary.map { $0.map() },
                secondary: payloadPreview.secondary.map { $0.map() }
            )
        }()
        self.messageDisplayType = messageDisplayType
        self.confirmTransferDelegate = confirmTransferDelegate
    }

    public var networkText: String {
        payload.chain.asset.name
    }

    public var title: String {
        Localized.Transfer.reviewRequest
    }

    public var walletText: String {
        payload.wallet.name
    }

    public var buttonTitle: String {
        Localized.Transfer.confirm
    }

    public var connectionViewModel: WalletConnectionViewModel {
        WalletConnectionViewModel(connection: WalletConnection(session: payload.session, wallet: payload.wallet))
    }

    public var appName: String {
        payload.session.metadata.shortName
    }

    public var appUrl: URL? {
        payload.session.metadata.url.asURL
    }

    public var appAssetImage: AssetImage {
        AssetImage(imageURL: connectionViewModel.imageUrl)
    }

    public var walletAssetImage: AssetImage {
        WalletViewModel(wallet: payload.wallet).avatarImage
    }

    public var networkAssetImage: AssetImage {
        AssetIdViewModel(assetId: payload.chain.asset.id).networkAssetImage
    }

    public var appText: String {
        appName
    }

    var textMessageViewModel: TextMessageViewModel {
        TextMessageViewModel(message: plainMessage)
    }


    public var simulationWarnings: [SimulationWarning] {
        payload.simulation.warnings
    }

    public var primaryPayloadFields: [SimulationPayloadField] {
        switch messageDisplayType {
        case .payload(let primaryFields, _):
            primaryFields
        case .text:
            []
        }
    }

    public var secondaryPayloadFields: [SimulationPayloadField] {
        switch messageDisplayType {
        case .payload(_, let secondaryFields):
            secondaryFields
        case .text:
            []
        }
    }

    public var hasPayload: Bool {
        !payloadFields.isEmpty
    }

    public var hasWarnings: Bool {
        !simulationWarnings.isEmpty
    }

    public var isButtonDisabled: Bool {
        simulationWarnings.contains(where: { $0.severity == .critical })
    }

    public var buttonType: ButtonType {
        .primary(isButtonDisabled ? .disabled : .normal)
    }

    public func signMessage() async throws {
        var privateKey = try await keystore.getPrivateKey(wallet: payload.wallet, chain: payload.chain)
        defer { privateKey.zeroize() }

        let signature = try signer.sign(privateKey: privateKey)
        confirmTransferDelegate(.success(signature))
    }
}

// MARK: - Actions

extension SignMessageSceneViewModel {
    public func fetch() {
        Task {
            await loadPayloadAddressNamesIfNeeded()
        }
    }

    public func payloadFieldViewModel(for field: SimulationPayloadField) -> SimulationPayloadFieldViewModel {
        SimulationPayloadFieldViewModel(
            field: field,
            chain: payload.chain,
            addressName: payloadAddressNames[ChainAddress(chain: payload.chain, address: field.value)]
        )
    }

    public func contextMenuItems(for field: SimulationPayloadField) -> [ContextMenuItemType] {
        var items = payloadFieldViewModel(for: field).contextMenuItems
        guard field.fieldType == .address else { return items }

        let link = explorerService.addressUrl(chain: payload.chain, address: field.value)
        items.append(.url(title: Localized.Transaction.viewOn(link.name), onOpen: { [weak self] in
            self?.isPresentingUrl = URL(string: link.link)
        }))
        return items
    }

    public func onViewWebsite() {
        isPresentingUrl = appUrl
    }

    public func onViewPayloadDetails() {
        isPresentingPayloadDetails = true
    }
}

private extension SignMessageSceneViewModel {
    func loadPayloadAddressNamesIfNeeded() async {
        guard payloadAddressNames.isEmpty else { return }
        guard !payloadFields.isEmpty else { return }

        do {
            payloadAddressNames = try await addressNameService.getAddressNames(requests: payloadAddressRequests)
        } catch {
            debugLog("payload address name lookup error: \(error)")
        }
    }

    var payloadFields: [SimulationPayloadField] {
        primaryPayloadFields + secondaryPayloadFields
    }

    var payloadAddressRequests: [ChainAddress] {
        payloadFields.compactMap {
            guard $0.fieldType == .address else {
                return nil
            }

            return ChainAddress(chain: payload.chain, address: $0.value)
        }
    }
}
