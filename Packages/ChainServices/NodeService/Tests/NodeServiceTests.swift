// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Primitives
import Store
import StoreTestKit
import NodeServiceTestKit

@testable import NodeService

struct NodeServiceTests {

    @Test
    func getNodeSelectedReturnsDefaultWhenNotSet() {
        #expect(NodeService.mock().getNodeSelected(chain: .ethereum).node.url == Chain.ethereum.defaultChainNode.node.url)
    }

    @Test
    func setNodeSelectedPersistsNode() throws {
        let service = NodeService.mock(nodeStore: .mock(db: .mockWithChains([.ethereum])))

        try service.setNodeSelected(chain: .ethereum, node: Chain.ethereum.asiaChainNode.node)

        #expect(service.getNodeSelected(chain: .ethereum).node.url == Chain.ethereum.asiaChainNode.node.url)
    }

    @Test
    func switchNode() throws {
        let service = NodeService.mock(nodeStore: .mock(db: .mockWithChains([.ethereum])))

        try service.setNodeSelected(chain: .ethereum, node: Chain.ethereum.asiaChainNode.node)
        #expect(service.getNodeSelected(chain: .ethereum).node.url == Chain.ethereum.asiaChainNode.node.url)

        try service.setNodeSelected(chain: .ethereum, node: Chain.ethereum.europeChainNode.node)
        #expect(service.getNodeSelected(chain: .ethereum).node.url == Chain.ethereum.europeChainNode.node.url)
    }

    @Test
    func nodeURLFetchableReturnsSelectedUrl() throws {
        let service = NodeService.mock(nodeStore: .mock(db: .mockWithChains([.ethereum])))

        try service.setNodeSelected(chain: .ethereum, node: Chain.ethereum.asiaChainNode.node)

        #expect(service.node(for: .ethereum) == Chain.ethereum.asiaChainNode.node.url.asURL)
    }

    @Test
    func nodeURLFetchableReturnsDefaultWhenNotSet() {
        let service = NodeService.mock()

        #expect(service.node(for: .ethereum) == Chain.ethereum.defaultBaseUrl)
    }
}
