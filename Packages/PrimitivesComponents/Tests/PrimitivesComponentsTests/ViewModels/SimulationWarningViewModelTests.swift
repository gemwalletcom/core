// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import BigInt
import Localization
import Primitives
import PrimitivesTestKit
import Style
@testable import PrimitivesComponents

struct SimulationWarningViewModelTests {

    @Test
    func titleUsesWarningTitleWhenMessageExists() {
        let warning = SimulationWarning(
            severity: .warning,
            warning: .validationError,
            message: "Chain ID mismatch"
        )
        let model = SimulationWarningViewModel(warning: warning)

        #expect(model.id == warning)
        #expect(model.title == Localized.Common.warning)
        #expect(model.message == "Chain ID mismatch")
    }

    @Test
    func titleUsesWarningTitleWhenDefaultMessageExists() {
        let warning = SimulationWarning(
            severity: .warning,
            warning: .permitApproval(assetId: Asset.mockEthereumUSDT().id, value: nil),
            message: nil
        )
        let model = SimulationWarningViewModel(warning: warning)

        #expect(model.title == Localized.Simulation.Warning.UnlimitedTokenApproval.title)
        #expect(model.message == Localized.Simulation.Warning.UnlimitedTokenApproval.description)
    }

    @Test
    func colorMatchesSeverity() {
        #expect(SimulationWarningViewModel(
            warning: SimulationWarning(severity: .critical, warning: .validationError, message: nil)
        ).color == Colors.red)

        #expect(SimulationWarningViewModel(
            warning: SimulationWarning(severity: .warning, warning: .validationError, message: nil)
        ).color == Colors.orange)
    }

    @Test
    func finiteApprovalWarningsAreHidden() {
        #expect(SimulationWarningViewModel(
            warning: SimulationWarning(
                severity: .warning,
                warning: .tokenApproval(assetId: Asset.mockEthereumUSDT().id, value:  BigInt(1)),
                message: nil
            )
        ).isVisible == false)

        #expect(SimulationWarningViewModel(
            warning: SimulationWarning(
                severity: .warning,
                warning: .permitApproval(assetId: Asset.mockEthereumUSDT().id, value: BigInt(1)),
                message: nil
            )
        ).isVisible == false)
    }

    @Test
    func suspiciousAddressUsesErrorOccurredTitle() {
        let model = SimulationWarningViewModel(
            warning: SimulationWarning(severity: .critical, warning: .suspiciousSpender, message: nil)
        )

        #expect(model.title == Localized.Errors.errorOccured)
        #expect(model.message == Localized.Common.suspiciousAddress)
    }
}
