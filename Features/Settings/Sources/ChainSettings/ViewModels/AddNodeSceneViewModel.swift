// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import Components
import Localization
import Blockchain
import ChainService
import NodeService
import PrimitivesComponents
import Validators
import Style

@MainActor
@Observable
final class AddNodeSceneViewModel {
    private let nodeService: NodeService
    private let chainServiceFactory: ChainServiceFactory
    private let addNodeService: AddNodeService

    let chain: Chain

    var urlInputModel = InputValidationViewModel(mode: .onDemand, validators: [.url])
    var state: StateViewType<AddNodeResultViewModel> = .noData
    var isPresentingScanner: Bool = false
    var isPresentingAlertMessage: AlertMessage?
    var fetchTrigger: AddNodeFetchTrigger?

    init(chain: Chain, nodeService: NodeService, chainServiceFactory: ChainServiceFactory) {
        self.chain = chain
        self.nodeService = nodeService
        self.chainServiceFactory = chainServiceFactory
        self.addNodeService = AddNodeService(nodeStore: nodeService.nodeStore)
    }

    var title: String { Localized.Nodes.ImportNode.title }

    var actionButtonTitle: String { Localized.Wallet.Import.action }
    var inputFieldTitle: String { Localized.Common.url }

    var errorTitle: String { Localized.Errors.errorOccured }
    var chainModel: ChainViewModel { ChainViewModel(chain: chain) }

    var warningModel: ListItemModel {
        ListItemModel(
            title: Localized.Asset.Verification.warningTitle,
            titleStyle: .headline,
            titleExtra: Localized.Nodes.ImportNode.warningMessage,
            titleStyleExtra: .bodySecondary,
            imageStyle: ListItemImageStyle(
                assetImage: AssetImage(type: Emoji.WalletAvatar.warning.rawValue),
                imageSize: .image.semiMedium,
                alignment: .top,
                cornerRadiusType: .none
            )
        )
    }
}

// MARK: - Business Logic

extension AddNodeSceneViewModel {
    func onChangeInput() {
        guard fetchTrigger?.url != urlInputModel.text else { return }
        setFetchTrigger(isImmediate: false)
    }

    func setInput(_ text: String) {
        urlInputModel.text = text
        setFetchTrigger(isImmediate: true)
    }

    private func setFetchTrigger(isImmediate: Bool) {
        let text = urlInputModel.text
        guard text.isNotEmpty, urlInputModel.isValid else {
            state = .noData
            fetchTrigger = nil
            return
        }
        fetchTrigger = AddNodeFetchTrigger(url: text, isImmediate: isImmediate)
    }

    func importFoundNode() throws {
        guard case .data(let model) = state else {
            throw AnyError("Unknown result")
        }
        
        // TODO: - implement disable after user selects "import node button", we can't use state: StateViewType<ImportNodeResult> progress
        let node = Node(url: model.url.absoluteString, status: .active, priority: 5)
        try addNodeService.addNode(ChainNodes(chain: chain.rawValue, nodes: [node]))

        // TODO: - impement correct way of selection node
        /*
        try nodeService.setNodeSelected(chain: chain, node: node)
         */
    }
    
    func fetch() async {
        guard let url = try? URLDecoder().decode(urlInputModel.text) else {
            // safety check for onSubmitUrl
            state = .error(AnyError(AddNodeError.invalidURL.errorDescription ?? ""))
            return
        }

        state = .loading
        let nodeProvider = CustomNodeULRFetchable(url: url, requestInterceptor: chainServiceFactory.requestInterceptor)
        let service = ChainServiceFactory(nodeProvider: nodeProvider).service(for: chain)

        do {
            let nodeStatus = try await service.getNodeStatus(url: urlInputModel.text)
            guard NodeService.isValid(netoworkId: nodeStatus.chainId, for: chain) else {
                throw AddNodeError.invalidNetworkId
            }
            
            let result = AddNodeResult(
                url: url, 
                chainID: nodeStatus.chainId, 
                blockNumber: nodeStatus.latestBlockNumber, 
                isInSync: true, 
                latency: nodeStatus.latency
            )
            state = .data(AddNodeResultViewModel(addNodeResult: result))
        } catch {
            state.setError(error)
        }
    }
}
