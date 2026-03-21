// Copyright (c). Gem Wallet. All rights reserved.

import XCTest

@MainActor
final class UpgradeVerificationTests: XCTestCase {

    override func setUpWithError() throws {
        try super.setUpWithError()
        continueAfterFailure = false
    }

    func testWalletSurvivedUpgrade() throws {
        let app = XCUIApplication()
        setupPermissionHandler()
        app.launch()

        XCTAssertFalse(app.isOnboarding, "App should not show onboarding after upgrade — wallet data was lost")

        // Navigate to wallet detail
        app.buttons["Wallet"].firstMatch.tap()
        app.tapWalletBar()

        // WalletsScene
        let gearButton = app.buttons["gearshape"].firstMatch
        XCTAssertTrue(gearButton.waitForExistence(timeout: 10), "No wallets found after upgrade")
        gearButton.tap()

        // WalletDetailScene
        let showPhraseButton = app.buttons["Show Secret Phrase"].firstMatch
        XCTAssertTrue(showPhraseButton.waitForExistence(timeout: 10), "Show Secret Phrase not found")
        showPhraseButton.tap()

        // SecurityReminderScene
        app.tapContinue()

        // ShowSecretDataScene
        let expectedWords = UITestKitConstants.words.components(separatedBy: " ")
        let displayedWords = app.getWords()
        XCTAssertEqual(displayedWords, expectedWords, "Secret phrase mismatch after upgrade — keys were corrupted")
    }
}
