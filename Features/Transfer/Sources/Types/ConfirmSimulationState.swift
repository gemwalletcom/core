// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesComponents

struct ConfirmSimulationState {
    let warnings: [SimulationWarning]
    let primaryFields: [SimulationPayloadField]
    let secondaryFields: [SimulationPayloadField]
    let payloadAddressNames: [ChainAddress: AddressName]
    let headerData: AssetValueHeaderData?

    var hasDetails: Bool {
        !secondaryFields.isEmpty
    }

    func addressName(chain: Chain, for field: SimulationPayloadField) -> AddressName? {
        payloadAddressNames[ChainAddress(chain: chain, address: field.value)]
    }
}
