// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Style
import Localization
import Formatters
import Components

struct AddNodeResultViewModel: Sendable {
    static let valueFormatter = ValueFormatter.full_US

    private let addNodeResult: AddNodeResult

    init(addNodeResult: AddNodeResult) {
        self.addNodeResult = addNodeResult
    }

    var url: URL { addNodeResult.url }

    var isInSync: Bool { addNodeResult.isInSync }

    var chainIdField: ListItemField {
        ListItemField(title: Localized.Nodes.ImportNode.chainId, value: addNodeResult.chainID)
    }

    var inSyncField: ListItemField {
        ListItemField(title: Localized.Nodes.ImportNode.inSync, value: isInSync ? Emoji.checkmark : Emoji.reject)
    }

    var latestBlockField: ListItemField {
        ListItemField(title: Localized.Nodes.ImportNode.latestBlock, value: Self.valueFormatter.string(addNodeResult.blockNumber, decimals: 0))
    }

    var latencyField: ListItemField {
        ListItemField(title: Localized.Nodes.ImportNode.latency, value: LatencyViewModel(latency: addNodeResult.latency).title)
    }
}
