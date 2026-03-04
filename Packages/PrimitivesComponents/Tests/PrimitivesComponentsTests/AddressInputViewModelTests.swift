// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing
import Primitives
import PrimitivesTestKit
import GemstonePrimitives
import Validators

@testable import PrimitivesComponents

@MainActor
struct AddressInputViewModelTests {

    @Test
    func validate() {
        let model = AddressInputViewModel.mock(validators: [
            .required(requireName: "Address"),
            .address(Asset(.ethereum)),
        ])

        model.inputModel.text = "gemcoder"
        #expect(model.validate() == false)

        model.inputModel.text = "test.eth"
        model.nameRecordViewModel.state = .loading
        #expect(model.validate() == false)

        model.nameRecordViewModel.state = .error
        #expect(model.validate() == false)

        model.nameRecordViewModel.state = .complete(.mock())
        #expect(model.validate())
    }

    @Test
    func chainChangeResetsState() {
        let model = AddressInputViewModel.mock()

        model.inputModel.text = "sometext"
        model.nameRecordViewModel.state = .complete(.mock())
        model.chain = .bitcoin

        #expect(model.nameResolveState == .none)
        #expect(model.text == "sometext")
    }
}

private struct NameServiceMock: NameServiceable {
    func getName(name: String, chain: String) async throws -> NameRecord? {
        .mock()
    }
}

extension AddressInputViewModel {
    static func mock(
        chain: Chain = .ethereum,
        validators: [any TextValidator] = []
    ) -> AddressInputViewModel {
        AddressInputViewModel(
            chain: chain,
            nameService: NameServiceMock(),
            placeholder: "Address",
            validators: validators
        )
    }
}
