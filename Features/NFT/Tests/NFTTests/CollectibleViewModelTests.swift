import Testing
import Foundation
import Primitives
import PrimitivesTestKit
import WalletServiceTestKit
import StoreTestKit
import NFTServiceTestKit
import ExplorerService
import PrimitivesComponents
import AvatarService
import Store

@testable import NFT

@MainActor
struct CollectibleViewModelTests {
    
    @Test
    func tokenIdValue() {
        #expect(CollectibleViewModel.mock(assetData: .mock(asset: .mock(tokenId: "12345"))).tokenIdValue == "12345")
    }
    
    @Test
    func tokenIdField() {
        let shortModel = CollectibleViewModel.mock(assetData: .mock(asset: .mock(tokenId: "123")))
        let longModel = CollectibleViewModel.mock(assetData: .mock(asset: .mock(tokenId: "1234567890123456789")))

        #expect(shortModel.tokenIdField.value.text == "#123")
        #expect(longModel.tokenIdField.value.text == "1234567890123456789")
    }

    @Test
    func contractField() {
        #expect(CollectibleViewModel.mock(assetData: .mock(
            collection: .mock(contractAddress: "0x123"),
            asset: .mock(tokenId: "456")
        )).contractField?.value.text == "0x123")
        #expect(CollectibleViewModel.mock(assetData: .mock(
            collection: .mock(contractAddress: "0x12345678910"),
            asset: .mock(tokenId: "")
        )).contractField?.value.text == "0x1234...78910")
        #expect(CollectibleViewModel.mock(assetData: .mock(
            collection: .mock(contractAddress: ""),
            asset: .mock(tokenId: "456")
        )).contractField == nil)
    }
    
    @Test
    func tokenExplorerUrl() {
        #expect(CollectibleViewModel.mock(assetData: .mock(asset: .mock(tokenId: "1234", chain: .ethereum))).tokenExplorerUrl != nil)
    }
    
    @Test
    func showAttributes() {
        #expect(CollectibleViewModel.mock(assetData: .mock(asset: .mock(attributes: []))).showAttributes == false)
        
        let withAttributesModel = CollectibleViewModel.mock(assetData: .mock(asset: .mock(attributes: [
            NFTAttribute(name: "Color", value: "Blue", percentage: nil)
        ])))
        #expect(withAttributesModel.showAttributes == true)
    }
    
    @Test
    func showLinks() {
        #expect(CollectibleViewModel.mock(assetData: .mock(collection: .mock(links: []))).showLinks == false)
        #expect(CollectibleViewModel.mock(assetData: .mock(collection: .mock(links: [
            AssetLink(name: "Website", url: "https://example.com")
        ]))).showLinks == true)
    }
    
    @Test
    func onSelectViewTokenInExplorer() {
        let model = CollectibleViewModel.mock(assetData: .mock(asset: .mock(tokenId: "1234", chain: .ethereum)))

        #expect(model.isPresentingTokenExplorerUrl == nil)
        model.onSelectViewTokenInExplorer()
        #expect(model.isPresentingTokenExplorerUrl != nil)
    }

    @Test
    func isSendEnabled() {
        let enabledModel = CollectibleViewModel.mock(
            wallet: .mock(type: .multicoin),
            assetData: .mock(asset: .mock(chain: .ethereum))
        )
        #expect(enabledModel.isSendEnabled == true)

        let viewOnlyModel = CollectibleViewModel.mock(
            wallet: .mock(type: .view),
            assetData: .mock(asset: .mock(chain: .ethereum))
        )
        #expect(viewOnlyModel.isSendEnabled == false)

        let bitcoinModel = CollectibleViewModel.mock(
            wallet: .mock(type: .multicoin),
            assetData: .mock(asset: .mock(chain: .bitcoin))
        )
        #expect(bitcoinModel.isSendEnabled == false)
    }
}

// MARK: - Mock Extensions

extension CollectibleViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        assetData: NFTAssetData = .mock(),
        explorerService: ExplorerService = ExplorerService.standard
    ) -> CollectibleViewModel {
        CollectibleViewModel(
            wallet: wallet,
            assetData: assetData,
            avatarService: AvatarService(store: WalletStore.mock()),
            nftService: .mock(),
            explorerService: explorerService,
            isPresentingSelectedAssetInput: .constant(.none)
        )
    }
}
