// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Foundation
import Localization
import Primitives
import PrimitivesTestKit
import PrimitivesComponents
import Formatters

struct SimulationPayloadFieldViewModelTests {

    @Test
    func addressSubtitleWithName() {
        let field = SimulationPayloadField.standard(
            kind: .contract,
            value: "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7",
            fieldType: .address,
            display: .primary
        )
        let viewModel = SimulationPayloadFieldViewModel(
            field: field,
            chain: .ethereum,
            addressName: .mock(address: field.value, name: "Hyperliquid")
        )

        #expect(viewModel.subtitle == "Hyperliquid (\(AddressFormatter(address: field.value, chain: .ethereum).value()))")
    }

    @Test
    func timestampSubtitle() {
        let formatter = RelativeDateFormatter(
            locale: Locale(identifier: "en_US_POSIX"),
            timeZone: TimeZone(secondsFromGMT: 0)!
        )
        let field = SimulationPayloadField.custom(
            label: "issuedAt",
            value: "2026-03-09T19:36:00Z",
            fieldType: .timestamp,
            display: .secondary
        )
        let viewModel = SimulationPayloadFieldViewModel(
            field: field,
            chain: .ethereum,
            relativeDateFormatter: formatter
        )

        #expect(viewModel.subtitle == formatter.string(fromTimestampValue: field.value))
    }

    @Test
    func addressContextMenuItems() {
        let field = SimulationPayloadField.standard(
            kind: .spender,
            value: "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7",
            fieldType: .address,
            display: .primary
        )
        let viewModel = SimulationPayloadFieldViewModel(field: field, chain: .ethereum)

        #expect(viewModel.contextMenuItems.count == 1)

        guard case let .copy(_, value, _, _) = viewModel.contextMenuItems[0] else {
            Issue.record("Expected copy context menu item")
            return
        }

        #expect(value == field.value)
    }

    @Test
    func methodSubtitleUsesPayloadValue() {
        let field = SimulationPayloadField.standard(
            kind: .method,
            value: "Set Approval For All",
            fieldType: .text,
            display: .primary
        )
        let viewModel = SimulationPayloadFieldViewModel(field: field, chain: .ethereum)

        #expect(viewModel.subtitle == "Set Approval For All")
    }

    @Test
    func standardTitlesUseLocalizedValues() {
        #expect(SimulationPayloadFieldViewModel(
            field: .standard(kind: .contract, value: "0x1", fieldType: .address, display: .primary),
            chain: .ethereum
        ).title == Localized.Asset.contract)

        #expect(SimulationPayloadFieldViewModel(
            field: .standard(kind: .method, value: "approve", fieldType: .text, display: .primary),
            chain: .ethereum
        ).title == Localized.Common.method)

        #expect(SimulationPayloadFieldViewModel(
            field: .standard(kind: .token, value: "0x1", fieldType: .address, display: .primary),
            chain: .ethereum
        ).title == Localized.Common.token)

        #expect(SimulationPayloadFieldViewModel(
            field: .standard(kind: .spender, value: "0x1", fieldType: .address, display: .primary),
            chain: .ethereum
        ).title == Localized.Transfer.to)
    }

    @Test
    func customTitleUsesRawLabel() {
        let field = SimulationPayloadField.custom(
            label: "issuedAt",
            value: "2026-03-09T19:36:00Z",
            fieldType: .timestamp,
            display: .secondary
        )

        #expect(SimulationPayloadFieldViewModel(field: field, chain: .ethereum).title == "issuedAt")
    }
}
