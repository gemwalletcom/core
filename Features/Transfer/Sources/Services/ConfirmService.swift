// Copyright (c). Gem Wallet. All rights reserved.
import ExplorerService
import Keystore
import Primitives
import BigInt
import Blockchain
import ChainService
import AddressNameService
import ActivityService
import EventPresenterService
import PrimitivesComponents

public struct ConfirmService: Sendable {
    private let metadataProvider: any TransferMetadataProvidable
    private let transferTransactionProvider: any TransferTransactionProvidable
    private let transferExecutor: any TransferExecutable
    private let keystore: any Keystore
    private let chainService: any ChainServiceable
    private let explorerService: any ExplorerLinkFetchable
    private let addressNameService: AddressNameService
    private let activityService: ActivityService
    private let eventPresenterService: EventPresenterService

    public init(
        explorerService: any ExplorerLinkFetchable,
        metadataProvider: any TransferMetadataProvidable,
        transferTransactionProvider: any TransferTransactionProvidable,
        transferExecutor: any TransferExecutable,
        keystore: any Keystore,
        chainService: any ChainServiceable,
        addressNameService: AddressNameService,
        activityService: ActivityService,
        eventPresenterService: EventPresenterService
    ) {
        self.explorerService = explorerService
        self.metadataProvider = metadataProvider
        self.transferTransactionProvider = transferTransactionProvider
        self.transferExecutor = transferExecutor
        self.keystore = keystore
        self.chainService = chainService
        self.addressNameService = addressNameService
        self.activityService = activityService
        self.eventPresenterService = eventPresenterService
    }

    public func getMetadata(wallet: Wallet, data: TransferData) throws -> TransferDataMetadata {
        try metadataProvider.metadata(wallet: wallet, data: data)
    }

    public func getExplorerLink(chain: Chain, address: String) -> BlockExplorerLink {
        explorerService.addressUrl(chain: chain, address: address)
    }

    public func loadTransferTransactionData(
        wallet: Wallet,
        data: TransferData,
        priority: FeePriority,
        available: BigInt
    ) async throws -> TransferTransactionData {
        try await transferTransactionProvider.loadTransferTransactionData(
            wallet: wallet,
            data: data,
            priority: priority,
            available: available
        )
    }

    public func executeTransfer(input: TransferConfirmationInput) async throws {
        try await transferExecutor.execute(input: input)
        await eventPresenterService.present(.transfer(input.data))
    }

    public func updateRecent(data: RecentActivityData, walletId: WalletId) {
        do {
            try activityService.updateRecent(data: data, walletId: walletId)
        } catch {
            debugLog("Failed to update recent activity: \(error)")
        }
    }

    public func getPasswordAuthentication() throws -> KeystoreAuthentication {
        try keystore.getPasswordAuthentication()
    }

    public func defaultPriority(for type: TransferDataType) -> FeePriority {
        chainService.defaultPriority(for: type)
    }
    
    public func getAddressName(chain: Chain, address: String) throws -> AddressName? {
        try addressNameService.getAddressName(chain: chain, address: address)
    }
}
