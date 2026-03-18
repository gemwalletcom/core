import Foundation
import SwiftUI
import Primitives
import Localization
import PrimitivesComponents
import BalanceService
import Components
import Formatters

@Observable
@MainActor
public final class ReceiveViewModel: Sendable {
    var qrSize: CGFloat {
        UIDevice.current.userInterfaceIdiom == .pad ? 180 : 260
    }

    let assetModel: AssetViewModel
    let wallet: Wallet
    let address: String
    let assetsEnabler: any AssetsEnabler
    let generator = QRCodeGenerator()

    public var isPresentingShareSheet: Bool = false
    public var isPresentingCopyToast: Bool = false
    public var renderedImage: UIImage? = nil

    public init(
        assetModel: AssetViewModel,
        wallet: Wallet,
        address: String,
        assetsEnabler: any AssetsEnabler
    ) {
        self.assetModel = assetModel
        self.wallet = wallet
        self.address = address
        self.assetsEnabler = assetsEnabler
    }

    var title: String {
        Localized.Receive.title("")
    }

    var addressShort: String {
        AddressFormatter(style: .short, address: address, chain: assetModel.asset.chain).value()
    }

    var shareTitle: String {
        Localized.Common.share
    }

    var copyTitle: String {
        Localized.Common.copy
    }


    var warningMessage: String {
        [Localized.Receive.warning(assetModel.symbol.boldMarkdown(), assetModel.networkFullName.boldMarkdown()), memoWarningText]
            .compactMap { $0 }
            .joined(separator: " ")
    }

    private var memoWarningText: String? {
        switch assetModel.asset.chain {
        case .xrp where assetModel.asset.chain.isMemoSupported: Localized.Wallet.Receive.noDestinationTagRequired
        case _ where assetModel.asset.chain.isMemoSupported: Localized.Wallet.Receive.noMemoRequired
        default: nil
        }
    }

    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(
            type: .address(assetModel.asset, address: addressShort),
            copyValue: address
        )
    }

    func activityItems(qrImage: UIImage?) -> [Any] {
        if let qrImage {
            return [qrImage, address]
        }
        return [address]
    }

    func enableAsset() async {
        do {
            try await assetsEnabler.enableAssets(wallet: wallet, assetIds: [assetModel.asset.id], enabled: true)
        } catch {
            debugLog("ReceiveViewModel enableAsset error: \(error)")
        }
    }

    func generateQRCode() async -> UIImage? {
        await generator.generate(
            from: address,
            size: CGSize(
                width: qrSize,
                height: qrSize
            ),
            logo: UIImage.name("logo-dark")
        )
    }
}

// MARK: - Actions

extension ReceiveViewModel {
    func onTaskOnce() {
        Task {
            await enableAsset()
        }
    }

    func onShareSheet() {
        isPresentingShareSheet = true
    }

    func onCopyAddress() {
        isPresentingCopyToast = true
    }

    func onLoadImage() async {
        renderedImage = await generateQRCode()
    }
}
