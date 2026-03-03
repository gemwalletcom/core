// swiftlint:disable all
// Generated using SwiftGen — https://github.com/SwiftGen/SwiftGen

import Foundation

// swiftlint:disable superfluous_disable_command file_length implicit_return prefer_self_in_static_references

// MARK: - Strings

// swiftlint:disable explicit_type_interface function_parameter_count identifier_name line_length
// swiftlint:disable nesting type_body_length type_name vertical_whitespace_opening_braces
public enum Localized {
  /// Gem Wallet
  public static let brandName = Localized.tr("Localizable", "brand_name", fallback: "Gem Wallet")
  public enum Activity {
    /// Activity
    public static let title = Localized.tr("Localizable", "activity.title", fallback: "Activity")
    public enum State {
      public enum Empty {
        /// Make your first transaction
        public static let description = Localized.tr("Localizable", "activity.state.empty.description", fallback: "Make your first transaction")
        /// Clear filters to refresh your activities
        public static let searchDescription = Localized.tr("Localizable", "activity.state.empty.search_description", fallback: "Clear filters to refresh your activities")
        /// No activities found
        public static let searchTitle = Localized.tr("Localizable", "activity.state.empty.search_title", fallback: "No activities found")
        /// Your activity will appear here
        public static let title = Localized.tr("Localizable", "activity.state.empty.title", fallback: "Your activity will appear here")
      }
    }
  }
  public enum Asset {
    /// Add to wallet
    public static let addToWallet = Localized.tr("Localizable", "asset.add_to_wallet", fallback: "Add to wallet")
    /// All Time High
    public static let allTimeHigh = Localized.tr("Localizable", "asset.all_time_high", fallback: "All Time High")
    /// All Time Low
    public static let allTimeLow = Localized.tr("Localizable", "asset.all_time_low", fallback: "All Time Low")
    /// Balances
    public static let balances = Localized.tr("Localizable", "asset.balances", fallback: "Balances")
    /// Buy %@
    public static func buyAsset(_ p1: Any) -> String {
      return Localized.tr("Localizable", "asset.buy_asset", String(describing: p1), fallback: "Buy %@")
    }
    /// Circulating Supply
    public static let circulatingSupply = Localized.tr("Localizable", "asset.circulating_supply", fallback: "Circulating Supply")
    /// Contract
    public static let contract = Localized.tr("Localizable", "asset.contract", fallback: "Contract")
    /// Decimals
    public static let decimals = Localized.tr("Localizable", "asset.decimals", fallback: "Decimals")
    /// Hide from wallet
    public static let hideFromWallet = Localized.tr("Localizable", "asset.hide_from_wallet", fallback: "Hide from wallet")
    /// Market Cap
    public static let marketCap = Localized.tr("Localizable", "asset.market_cap", fallback: "Market Cap")
    /// Market Cap Rank
    public static let marketCapRank = Localized.tr("Localizable", "asset.market_cap_rank", fallback: "Market Cap Rank")
    /// Name
    public static let name = Localized.tr("Localizable", "asset.name", fallback: "Name")
    /// Price
    public static let price = Localized.tr("Localizable", "asset.price", fallback: "Price")
    /// Resources
    public static let resources = Localized.tr("Localizable", "asset.resources", fallback: "Resources")
    /// Symbol
    public static let symbol = Localized.tr("Localizable", "asset.symbol", fallback: "Symbol")
    /// Token ID
    public static let tokenId = Localized.tr("Localizable", "asset.token_id", fallback: "Token ID")
    /// Total Supply
    public static let totalSupply = Localized.tr("Localizable", "asset.total_supply", fallback: "Total Supply")
    /// Trading Volume (24h)
    public static let tradingVolume = Localized.tr("Localizable", "asset.trading_volume", fallback: "Trading Volume (24h)")
    /// View address on %@
    public static func viewAddressOn(_ p1: Any) -> String {
      return Localized.tr("Localizable", "asset.view_address_on", String(describing: p1), fallback: "View address on %@")
    }
    /// View token on %@
    public static func viewTokenOn(_ p1: Any) -> String {
      return Localized.tr("Localizable", "asset.view_token_on", String(describing: p1), fallback: "View token on %@")
    }
    public enum Balances {
      /// Available
      public static let available = Localized.tr("Localizable", "asset.balances.available", fallback: "Available")
      /// Reserved
      public static let reserved = Localized.tr("Localizable", "asset.balances.reserved", fallback: "Reserved")
    }
    public enum State {
      public enum Empty {
        /// Receive, swap or buy %@
        public static func description(_ p1: Any) -> String {
          return Localized.tr("Localizable", "asset.state.empty.description", String(describing: p1), fallback: "Receive, swap or buy %@")
        }
        /// Your transactions will appear here️
        public static let title = Localized.tr("Localizable", "asset.state.empty.title", fallback: "Your transactions will appear here️")
      }
    }
    public enum Verification {
      /// Suspicious
      public static let suspicious = Localized.tr("Localizable", "asset.verification.suspicious", fallback: "Suspicious")
      /// Unverified
      public static let unverified = Localized.tr("Localizable", "asset.verification.unverified", fallback: "Unverified")
      /// Verified
      public static let verified = Localized.tr("Localizable", "asset.verification.verified", fallback: "Verified")
      /// Anyone can create one - including fake or malicious tokens.
      public static let warningMessage = Localized.tr("Localizable", "asset.verification.warning_message", fallback: "Anyone can create one - including fake or malicious tokens.")
      /// Know What You’re Adding
      public static let warningTitle = Localized.tr("Localizable", "asset.verification.warning_title", fallback: "Know What You’re Adding")
    }
  }
  public enum Assets {
    /// Add Custom Token
    public static let addCustomToken = Localized.tr("Localizable", "assets.add_custom_token", fallback: "Add Custom Token")
    /// No assets found
    public static let noAssetsFound = Localized.tr("Localizable", "assets.no_assets_found", fallback: "No assets found")
    /// Select Asset
    public static let selectAsset = Localized.tr("Localizable", "assets.select_asset", fallback: "Select Asset")
    /// Assets
    public static let title = Localized.tr("Localizable", "assets.title", fallback: "Assets")
    public enum State {
      public enum Empty {
        /// You can try to add it manually
        public static let searchDescription = Localized.tr("Localizable", "assets.state.empty.search_description", fallback: "You can try to add it manually")
      }
    }
    public enum Tags {
      /// Gainers
      public static let gainers = Localized.tr("Localizable", "assets.tags.gainers", fallback: "Gainers")
      /// Losers
      public static let losers = Localized.tr("Localizable", "assets.tags.losers", fallback: "Losers")
      /// New
      public static let new = Localized.tr("Localizable", "assets.tags.new", fallback: "New")
      /// Stablecoins
      public static let stablecoins = Localized.tr("Localizable", "assets.tags.stablecoins", fallback: "Stablecoins")
      /// Trending
      public static let trending = Localized.tr("Localizable", "assets.tags.trending", fallback: "Trending")
    }
  }
  public enum Banner {
    public enum AccountActivation {
      /// The %@ network requires a one time fee of %@.
      public static func description(_ p1: Any, _ p2: Any) -> String {
        return Localized.tr("Localizable", "banner.account_activation.description", String(describing: p1), String(describing: p2), fallback: "The %@ network requires a one time fee of %@.")
      }
      /// Account Activation Fee
      public static let title = Localized.tr("Localizable", "banner.account_activation.title", fallback: "Account Activation Fee")
    }
    public enum ActivateAsset {
      /// To use the %@ asset, you must first enable it on the %@ network by fulfilling the network’s specific requirements.
      public static func description(_ p1: Any, _ p2: Any) -> String {
        return Localized.tr("Localizable", "banner.activate_asset.description", String(describing: p1), String(describing: p2), fallback: "To use the %@ asset, you must first enable it on the %@ network by fulfilling the network’s specific requirements.")
      }
    }
    public enum AssetStatus {
      /// Token may be unsafe or misleading. Proceed only if you fully trust it.
      public static let description = Localized.tr("Localizable", "banner.asset_status.description", fallback: "Token may be unsafe or misleading. Proceed only if you fully trust it.")
      /// Suspicious Asset
      public static let title = Localized.tr("Localizable", "banner.asset_status.title", fallback: "Suspicious Asset")
    }
    public enum EnableNotifications {
      /// Stay on top of your wallet activity.
      public static let description = Localized.tr("Localizable", "banner.enable_notifications.description", fallback: "Stay on top of your wallet activity.")
      /// Enable Notifications
      public static let title = Localized.tr("Localizable", "banner.enable_notifications.title", fallback: "Enable Notifications")
    }
    public enum Onboarding {
      /// Buy or Receive crypto to get started
      public static let description = Localized.tr("Localizable", "banner.onboarding.description", fallback: "Buy or Receive crypto to get started")
      /// Your wallet is ready
      public static let title = Localized.tr("Localizable", "banner.onboarding.title", fallback: "Your wallet is ready")
    }
    public enum Perpetuals {
      /// Deposit, trade, and earn with Hyperliquid perpetuals
      public static let description = Localized.tr("Localizable", "banner.perpetuals.description", fallback: "Deposit, trade, and earn with Hyperliquid perpetuals")
      /// Trade Perpetuals on Hyperliquid
      public static let title = Localized.tr("Localizable", "banner.perpetuals.title", fallback: "Trade Perpetuals on Hyperliquid")
    }
    public enum Stake {
      /// Earn %@ rewards on your stake while you sleep.
      public static func description(_ p1: Any) -> String {
        return Localized.tr("Localizable", "banner.stake.description", String(describing: p1), fallback: "Earn %@ rewards on your stake while you sleep.")
      }
      /// Start staking %@
      public static func title(_ p1: Any) -> String {
        return Localized.tr("Localizable", "banner.stake.title", String(describing: p1), fallback: "Start staking %@")
      }
    }
  }
  public enum Buy {
    /// No quotes available
    public static let noResults = Localized.tr("Localizable", "buy.no_results", fallback: "No quotes available")
    /// Rate
    public static let rate = Localized.tr("Localizable", "buy.rate", fallback: "Rate")
    /// Buy %@
    public static func title(_ p1: Any) -> String {
      return Localized.tr("Localizable", "buy.title", String(describing: p1), fallback: "Buy %@")
    }
    public enum Providers {
      /// Providers
      public static let title = Localized.tr("Localizable", "buy.providers.title", fallback: "Providers")
    }
  }
  public enum Charts {
    /// All
    public static let all = Localized.tr("Localizable", "charts.all", fallback: "All")
    /// 1D
    public static let day = Localized.tr("Localizable", "charts.day", fallback: "1D")
    /// Entry
    public static let entry = Localized.tr("Localizable", "charts.entry", fallback: "Entry")
    /// 1H
    public static let hour = Localized.tr("Localizable", "charts.hour", fallback: "1H")
    /// Liq
    public static let liquidation = Localized.tr("Localizable", "charts.liquidation", fallback: "Liq")
    /// 1M
    public static let month = Localized.tr("Localizable", "charts.month", fallback: "1M")
    /// SL
    public static let stopLoss = Localized.tr("Localizable", "charts.stop_loss", fallback: "SL")
    /// TP
    public static let takeProfit = Localized.tr("Localizable", "charts.take_profit", fallback: "TP")
    /// 1W
    public static let week = Localized.tr("Localizable", "charts.week", fallback: "1W")
    /// 1Y
    public static let year = Localized.tr("Localizable", "charts.year", fallback: "1Y")
    public enum Price {
      /// Change
      public static let change = Localized.tr("Localizable", "charts.price.change", fallback: "Change")
      /// Close
      public static let close = Localized.tr("Localizable", "charts.price.close", fallback: "Close")
      /// High
      public static let high = Localized.tr("Localizable", "charts.price.high", fallback: "High")
      /// Low
      public static let low = Localized.tr("Localizable", "charts.price.low", fallback: "Low")
      /// Open
      public static let `open` = Localized.tr("Localizable", "charts.price.open", fallback: "Open")
    }
  }
  public enum Common {
    /// Address
    public static let address = Localized.tr("Localizable", "common.address", fallback: "Address")
    /// All
    public static let all = Localized.tr("Localizable", "common.all", fallback: "All")
    /// Avatar
    public static let avatar = Localized.tr("Localizable", "common.avatar", fallback: "Avatar")
    /// Back
    public static let back = Localized.tr("Localizable", "common.back", fallback: "Back")
    /// Cancel
    public static let cancel = Localized.tr("Localizable", "common.cancel", fallback: "Cancel")
    /// Continue
    public static let `continue` = Localized.tr("Localizable", "common.continue", fallback: "Continue")
    /// Copied: %@
    public static func copied(_ p1: Any) -> String {
      return Localized.tr("Localizable", "common.copied", String(describing: p1), fallback: "Copied: %@")
    }
    /// Copy
    public static let copy = Localized.tr("Localizable", "common.copy", fallback: "Copy")
    /// Delete
    public static let delete = Localized.tr("Localizable", "common.delete", fallback: "Delete")
    /// Are sure you want to delete %@?
    public static func deleteConfirmation(_ p1: Any) -> String {
      return Localized.tr("Localizable", "common.delete_confirmation", String(describing: p1), fallback: "Are sure you want to delete %@?")
    }
    /// Description
    public static let description = Localized.tr("Localizable", "common.description", fallback: "Description")
    /// Details
    public static let details = Localized.tr("Localizable", "common.details", fallback: "Details")
    /// Done
    public static let done = Localized.tr("Localizable", "common.done", fallback: "Done")
    /// Earn
    public static let earn = Localized.tr("Localizable", "common.earn", fallback: "Earn")
    /// Edit
    public static let edit = Localized.tr("Localizable", "common.edit", fallback: "Edit")
    /// Emoji
    public static let emoji = Localized.tr("Localizable", "common.emoji", fallback: "Emoji")
    /// Get Started
    public static let getStarted = Localized.tr("Localizable", "common.get_started", fallback: "Get Started")
    /// Hide
    public static let hide = Localized.tr("Localizable", "common.hide", fallback: "Hide")
    /// Info
    public static let info = Localized.tr("Localizable", "common.info", fallback: "Info")
    /// %d ms
    public static func latencyInMs(_ p1: Int) -> String {
      return Localized.tr("Localizable", "common.latency_in_ms", p1, fallback: "%d ms")
    }
    /// Learn More
    public static let learnMore = Localized.tr("Localizable", "common.learn_more", fallback: "Learn More")
    /// Loading
    public static let loading = Localized.tr("Localizable", "common.loading", fallback: "Loading")
    /// Manage
    public static let manage = Localized.tr("Localizable", "common.manage", fallback: "Manage")
    /// Next
    public static let next = Localized.tr("Localizable", "common.next", fallback: "Next")
    /// No
    public static let no = Localized.tr("Localizable", "common.no", fallback: "No")
    /// No Results Found
    public static let noResultsFound = Localized.tr("Localizable", "common.no_results_found", fallback: "No Results Found")
    /// Not Available
    public static let notAvailable = Localized.tr("Localizable", "common.not_available", fallback: "Not Available")
    /// Open settings
    public static let openSettings = Localized.tr("Localizable", "common.open_settings", fallback: "Open settings")
    /// Paste
    public static let paste = Localized.tr("Localizable", "common.paste", fallback: "Paste")
    /// Percentage
    public static let percentage = Localized.tr("Localizable", "common.percentage", fallback: "Percentage")
    /// Photo
    public static let photo = Localized.tr("Localizable", "common.photo", fallback: "Photo")
    /// Phrase
    public static let phrase = Localized.tr("Localizable", "common.phrase", fallback: "Phrase")
    /// Pin
    public static let pin = Localized.tr("Localizable", "common.pin", fallback: "Pin")
    /// Pinned
    public static let pinned = Localized.tr("Localizable", "common.pinned", fallback: "Pinned")
    /// Popular
    public static let popular = Localized.tr("Localizable", "common.popular", fallback: "Popular")
    /// Private Key
    public static let privateKey = Localized.tr("Localizable", "common.private_key", fallback: "Private Key")
    /// Provider
    public static let provider = Localized.tr("Localizable", "common.provider", fallback: "Provider")
    /// Recommended
    public static let recommended = Localized.tr("Localizable", "common.recommended", fallback: "Recommended")
    /// Save
    public static let save = Localized.tr("Localizable", "common.save", fallback: "Save")
    /// Secret Phrase
    public static let secretPhrase = Localized.tr("Localizable", "common.secret_phrase", fallback: "Secret Phrase")
    /// Share
    public static let share = Localized.tr("Localizable", "common.share", fallback: "Share")
    /// Gem
    public static let shortName = Localized.tr("Localizable", "common.short_name", fallback: "Gem")
    /// Show %@
    public static func show(_ p1: Any) -> String {
      return Localized.tr("Localizable", "common.show", String(describing: p1), fallback: "Show %@")
    }
    /// Skip
    public static let skip = Localized.tr("Localizable", "common.skip", fallback: "Skip")
    /// Style
    public static let style = Localized.tr("Localizable", "common.style", fallback: "Style")
    /// Try Again
    public static let tryAgain = Localized.tr("Localizable", "common.try_again", fallback: "Try Again")
    /// Type
    public static let type = Localized.tr("Localizable", "common.type", fallback: "Type")
    /// Unpin
    public static let unpin = Localized.tr("Localizable", "common.unpin", fallback: "Unpin")
    /// URL
    public static let url = Localized.tr("Localizable", "common.url", fallback: "URL")
    /// Wallet
    public static let wallet = Localized.tr("Localizable", "common.wallet", fallback: "Wallet")
    /// Warning
    public static let warning = Localized.tr("Localizable", "common.warning", fallback: "Warning")
    /// Yes
    public static let yes = Localized.tr("Localizable", "common.yes", fallback: "Yes")
  }
  public enum Contacts {
    /// Add to Contacts
    public static let addToContacts = Localized.tr("Localizable", "contacts.add_to_contacts", fallback: "Add to Contacts")
    /// Addresses
    public static let addresses = Localized.tr("Localizable", "contacts.addresses", fallback: "Addresses")
    /// Contact
    public static let contact = Localized.tr("Localizable", "contacts.contact", fallback: "Contact")
    /// Contacts
    public static let title = Localized.tr("Localizable", "contacts.title", fallback: "Contacts")
    public enum State {
      public enum Empty {
        /// Save your frequently used addresses
        public static let description = Localized.tr("Localizable", "contacts.state.empty.description", fallback: "Save your frequently used addresses")
        /// No Contacts
        public static let title = Localized.tr("Localizable", "contacts.state.empty.title", fallback: "No Contacts")
      }
    }
  }
  public enum Date {
    /// Today
    public static let today = Localized.tr("Localizable", "date.today", fallback: "Today")
    /// Yesterday
    public static let yesterday = Localized.tr("Localizable", "date.yesterday", fallback: "Yesterday")
  }
  public enum Earn {
    public enum State {
      public enum Empty {
        /// Deposit your first %@
        public static func description(_ p1: Any) -> String {
          return Localized.tr("Localizable", "earn.state.empty.description", String(describing: p1), fallback: "Deposit your first %@")
        }
        /// Your positions will appear here
        public static let title = Localized.tr("Localizable", "earn.state.empty.title", fallback: "Your positions will appear here")
      }
    }
  }
  public enum Errors {
    /// Camera permission not granted. Please enable camera access in settings to scan QR code.
    public static let cameraPermissionsNotGranted = Localized.tr("Localizable", "errors.camera_permissions_not_granted", fallback: "Camera permission not granted. Please enable camera access in settings to scan QR code.")
    /// Create Wallet Error: %@
    public static func createWallet(_ p1: Any) -> String {
      return Localized.tr("Localizable", "errors.create_wallet", String(describing: p1), fallback: "Create Wallet Error: %@")
    }
    /// Decoding Error
    public static let decoding = Localized.tr("Localizable", "errors.decoding", fallback: "Decoding Error")
    /// Failed to decode the QR code. Please try again with a different QR code.
    public static let decodingQr = Localized.tr("Localizable", "errors.decoding_qr", fallback: "Failed to decode the QR code. Please try again with a different QR code.")
    /// The transaction failed because the amount is too small to meet the %@ network’s minimum requirement (dust threshold). This limit ensures the transaction value covers the fees and processing costs.
    public static func dustThreshold(_ p1: Any) -> String {
      return Localized.tr("Localizable", "errors.dust_threshold", String(describing: p1), fallback: "The transaction failed because the amount is too small to meet the %@ network’s minimum requirement (dust threshold). This limit ensures the transaction value covers the fees and processing costs.")
    }
    /// The network considers this amount dust — the fee is higher than the amount itself.
    public static let dustThresholdShort = Localized.tr("Localizable", "errors.dust_threshold_short", fallback: "The network considers this amount dust — the fee is higher than the amount itself.")
    /// Error
    public static let error = Localized.tr("Localizable", "errors.error", fallback: "Error")
    /// An error occurred!
    public static let errorOccured = Localized.tr("Localizable", "errors.error_occured", fallback: "An error occurred!")
    /// Invalid address or name
    public static let invalidAddressName = Localized.tr("Localizable", "errors.invalid_address_name", fallback: "Invalid address or name")
    /// Invalid amount
    public static let invalidAmount = Localized.tr("Localizable", "errors.invalid_amount", fallback: "Invalid amount")
    /// Invalid %@ address
    public static func invalidAssetAddress(_ p1: Any) -> String {
      return Localized.tr("Localizable", "errors.invalid_asset_address", String(describing: p1), fallback: "Invalid %@ address")
    }
    /// Invalid Network ID
    public static let invalidNetworkId = Localized.tr("Localizable", "errors.invalid_network_id", fallback: "Invalid Network ID")
    /// Invalid URL
    public static let invalidUrl = Localized.tr("Localizable", "errors.invalid_url", fallback: "Invalid URL")
    /// No data available
    public static let noDataAvailable = Localized.tr("Localizable", "errors.no_data_available", fallback: "No data available")
    /// Not Supported
    public static let notSupported = Localized.tr("Localizable", "errors.not_supported", fallback: "Not Supported")
    /// This device does not support QR code scanning. You can only select QR code image from library.
    public static let notSupportedQr = Localized.tr("Localizable", "errors.not_supported_qr", fallback: "This device does not support QR code scanning. You can only select QR code image from library.")
    /// Permissions Not Granted
    public static let permissionsNotGranted = Localized.tr("Localizable", "errors.permissions_not_granted", fallback: "Permissions Not Granted")
    /// %@ is required
    public static func `required`(_ p1: Any) -> String {
      return Localized.tr("Localizable", "errors.required", String(describing: p1), fallback: "%@ is required")
    }
    /// Transfer Error: %@
    public static func transfer(_ p1: Any) -> String {
      return Localized.tr("Localizable", "errors.transfer", String(describing: p1), fallback: "Transfer Error: %@")
    }
    /// Transfer Error
    public static let transferError = Localized.tr("Localizable", "errors.transfer_error", fallback: "Transfer Error")
    /// We are currently unable to calculate the network fee.
    public static let unableEstimateNetworkFee = Localized.tr("Localizable", "errors.unable_estimate_network_fee", fallback: "We are currently unable to calculate the network fee.")
    /// Unknown
    public static let unknown = Localized.tr("Localizable", "errors.unknown", fallback: "Unknown")
    /// Validation Error: %@
    public static func validation(_ p1: Any) -> String {
      return Localized.tr("Localizable", "errors.validation", String(describing: p1), fallback: "Validation Error: %@")
    }
    public enum Connections {
      /// Invalid parameters provided for sending a transaction.
      public static let invalidSendParameters = Localized.tr("Localizable", "errors.connections.invalid_send_parameters", fallback: "Invalid parameters provided for sending a transaction.")
      /// Invalid parameters provided for signing.
      public static let invalidSignParameters = Localized.tr("Localizable", "errors.connections.invalid_sign_parameters", fallback: "Invalid parameters provided for signing.")
      /// This connection comes from an untrusted source.
      public static let maliciousOrigin = Localized.tr("Localizable", "errors.connections.malicious_origin", fallback: "This connection comes from an untrusted source.")
      /// No supported wallets are available.
      public static let noSupportedWallets = Localized.tr("Localizable", "errors.connections.no_supported_wallets", fallback: "No supported wallets are available.")
      /// The provided chain is not supported.
      public static let unsupportedChain = Localized.tr("Localizable", "errors.connections.unsupported_chain", fallback: "The provided chain is not supported.")
      /// The requested method is not supported.
      public static let unsupportedMethod = Localized.tr("Localizable", "errors.connections.unsupported_method", fallback: "The requested method is not supported.")
      /// User cancelled
      public static let userCancelled = Localized.tr("Localizable", "errors.connections.user_cancelled", fallback: "User cancelled")
    }
    public enum Import {
      /// Invalid Secret Phrase
      public static let invalidSecretPhrase = Localized.tr("Localizable", "errors.import.invalid_secret_phrase", fallback: "Invalid Secret Phrase")
      /// Invalid Secret Phrase word: %@
      public static func invalidSecretPhraseWord(_ p1: Any) -> String {
        return Localized.tr("Localizable", "errors.import.invalid_secret_phrase_word", String(describing: p1), fallback: "Invalid Secret Phrase word: %@")
      }
    }
    public enum ScanTransaction {
      /// %@ destination wallet address requires a destination tag / memo
      public static func memoRequired(_ p1: Any) -> String {
        return Localized.tr("Localizable", "errors.scan_transaction.memo_required", String(describing: p1), fallback: "%@ destination wallet address requires a destination tag / memo")
      }
      public enum Malicious {
        /// This transaction cannot be completed — the destination wallet address is linked to suspicious or harmful activity.
        public static let description = Localized.tr("Localizable", "errors.scan_transaction.malicious.description", fallback: "This transaction cannot be completed — the destination wallet address is linked to suspicious or harmful activity.")
        /// Suspicious Activity
        public static let title = Localized.tr("Localizable", "errors.scan_transaction.malicious.title", fallback: "Suspicious Activity")
      }
    }
    public enum Swap {
      /// Amount too small
      public static let amountTooSmall = Localized.tr("Localizable", "errors.swap.amount_too_small", fallback: "Amount too small")
      /// Minimum trade amount is %@. Please enter a higher amount.
      public static func minimumAmount(_ p1: Any) -> String {
        return Localized.tr("Localizable", "errors.swap.minimum_amount", String(describing: p1), fallback: "Minimum trade amount is %@. Please enter a higher amount.")
      }
      /// No quote available.
      public static let noQuoteAvailable = Localized.tr("Localizable", "errors.swap.no_quote_available", fallback: "No quote available.")
      /// Not supported asset.
      public static let notSupportedAsset = Localized.tr("Localizable", "errors.swap.not_supported_asset", fallback: "Not supported asset.")
      /// Not supported chain.
      public static let notSupportedChain = Localized.tr("Localizable", "errors.swap.not_supported_chain", fallback: "Not supported chain.")
      /// Not supported pair.
      public static let notSupportedPair = Localized.tr("Localizable", "errors.swap.not_supported_pair", fallback: "Not supported pair.")
    }
    public enum Token {
      /// Invalid Token ID
      public static let invalidId = Localized.tr("Localizable", "errors.token.invalid_id", fallback: "Invalid Token ID")
    }
    public enum Wallets {
      public enum Limit {
        /// You’ve reached the maximum number of wallets allowed (%d). Please remove an existing wallet to add or create a new one.
        public static func description(_ p1: Int) -> String {
          return Localized.tr("Localizable", "errors.wallets.limit.description", p1, fallback: "You’ve reached the maximum number of wallets allowed (%d). Please remove an existing wallet to add or create a new one.")
        }
        /// Wallets Limit Reached
        public static let title = Localized.tr("Localizable", "errors.wallets.limit.title", fallback: "Wallets Limit Reached")
      }
    }
  }
  public enum FeeRate {
    /// %@ gwei
    public static func gwei(_ p1: Any) -> String {
      return Localized.tr("Localizable", "fee_rate.gwei", String(describing: p1), fallback: "%@ gwei")
    }
    /// %@ sat/B
    public static func satB(_ p1: Any) -> String {
      return Localized.tr("Localizable", "fee_rate.satB", String(describing: p1), fallback: "%@ sat/B")
    }
    /// %@ sat/vB
    public static func satvB(_ p1: Any) -> String {
      return Localized.tr("Localizable", "fee_rate.satvB", String(describing: p1), fallback: "%@ sat/vB")
    }
  }
  public enum FeeRates {
    /// Fast
    public static let fast = Localized.tr("Localizable", "fee_rates.fast", fallback: "Fast")
    /// Speed of transaction is determined by network fee paid to the network miners.
    public static let info = Localized.tr("Localizable", "fee_rates.info", fallback: "Speed of transaction is determined by network fee paid to the network miners.")
    /// Normal
    public static let normal = Localized.tr("Localizable", "fee_rates.normal", fallback: "Normal")
    /// Slow
    public static let slow = Localized.tr("Localizable", "fee_rates.slow", fallback: "Slow")
  }
  public enum Filter {
    /// Clear
    public static let clear = Localized.tr("Localizable", "filter.clear", fallback: "Clear")
    /// Has balance
    public static let hasBalance = Localized.tr("Localizable", "filter.has_balance", fallback: "Has balance")
    /// Filters
    public static let title = Localized.tr("Localizable", "filter.title", fallback: "Filters")
    /// Types
    public static let types = Localized.tr("Localizable", "filter.types", fallback: "Types")
  }
  public enum Info {
    public enum AccountMinimumBalance {
      /// Minimum balance
      public static let title = Localized.tr("Localizable", "info.account_minimum_balance.title", fallback: "Minimum balance")
    }
    public enum AssetStatus {
      public enum Suspicious {
        /// Suspicious or spam tokens are identified as potential scams or harmful assets. They may appear in your wallet due to airdrops, transfers, or manual imports.
        public static let description = Localized.tr("Localizable", "info.asset_status.suspicious.description", fallback: "Suspicious or spam tokens are identified as potential scams or harmful assets. They may appear in your wallet due to airdrops, transfers, or manual imports.")
      }
      public enum Unverified {
        /// Unverified tokens have not been sufficiently verified by trusted third-party services. They may appear in your wallet due to airdrops, transfers, or manual imports.
        public static let description = Localized.tr("Localizable", "info.asset_status.unverified.description", fallback: "Unverified tokens have not been sufficiently verified by trusted third-party services. They may appear in your wallet due to airdrops, transfers, or manual imports.")
      }
    }
    public enum CirculatingSupply {
      /// The number of coins currently available and trading in the market."
      public static let description = Localized.tr("Localizable", "info.circulating_supply.description", fallback: "The number of coins currently available and trading in the market.\"")
    }
    public enum FullyDilutedValuation {
      /// The theoretical market value if all coins were in circulation. Calculated as price multiplied by max supply.
      public static let description = Localized.tr("Localizable", "info.fully_diluted_valuation.description", fallback: "The theoretical market value if all coins were in circulation. Calculated as price multiplied by max supply.")
      /// Fully Diluted Valuation
      public static let title = Localized.tr("Localizable", "info.fully_diluted_valuation.title", fallback: "Fully Diluted Valuation")
    }
    public enum FundingPayments {
      /// Funding payments are periodic payments between traders to keep the perpetual contract price close to the underlying asset's spot price. Positive funding means long positions pay short positions, while negative funding means short positions pay long positions.
      public static let description = Localized.tr("Localizable", "info.funding_payments.description", fallback: "Funding payments are periodic payments between traders to keep the perpetual contract price close to the underlying asset's spot price. Positive funding means long positions pay short positions, while negative funding means short positions pay long positions.")
      /// Funding Payments
      public static let title = Localized.tr("Localizable", "info.funding_payments.title", fallback: "Funding Payments")
    }
    public enum FundingRate {
      /// The funding rate determines the cost of holding a perpetual position. It is calculated hourly and helps maintain price equilibrium between the perpetual contract and the underlying asset's spot price.
      public static let description = Localized.tr("Localizable", "info.funding_rate.description", fallback: "The funding rate determines the cost of holding a perpetual position. It is calculated hourly and helps maintain price equilibrium between the perpetual contract and the underlying asset's spot price.")
      /// Funding
      public static let title = Localized.tr("Localizable", "info.funding_rate.title", fallback: "Funding")
    }
    public enum InsufficientBalance {
      /// You don’t have enough %@ to complete this transaction. Please top up, receive, or swap in your wallet and try again.
      public static func description(_ p1: Any) -> String {
        return Localized.tr("Localizable", "info.insufficient_balance.description", String(describing: p1), fallback: "You don’t have enough %@ to complete this transaction. Please top up, receive, or swap in your wallet and try again.")
      }
      /// Insufficient Balance
      public static let title = Localized.tr("Localizable", "info.insufficient_balance.title", fallback: "Insufficient Balance")
    }
    public enum InsufficientNetworkFeeBalance {
      /// This transaction requires %@ to cover the network fee paid to %@ miners, not Gem Wallet. Ensure you have enough %@.
      public static func description(_ p1: Any, _ p2: Any, _ p3: Any) -> String {
        return Localized.tr("Localizable", "info.insufficient_network_fee_balance.description", String(describing: p1), String(describing: p2), String(describing: p3), fallback: "This transaction requires %@ to cover the network fee paid to %@ miners, not Gem Wallet. Ensure you have enough %@.")
      }
      /// %@ required
      public static func title(_ p1: Any) -> String {
        return Localized.tr("Localizable", "info.insufficient_network_fee_balance.title", String(describing: p1), fallback: "%@ required")
      }
    }
    public enum LiquidationPrice {
      /// The liquidation price is the price level at which your position will be automatically closed to prevent further losses. When the market price reaches this level, your position is liquidated and you lose your margin.
      public static let description = Localized.tr("Localizable", "info.liquidation_price.description", fallback: "The liquidation price is the price level at which your position will be automatically closed to prevent further losses. When the market price reaches this level, your position is liquidated and you lose your margin.")
      /// Liquidation Price
      public static let title = Localized.tr("Localizable", "info.liquidation_price.title", fallback: "Liquidation Price")
    }
    public enum LockTime {
      /// Lock time, also known as the unbonding or unfreezing period, is the duration during which staked assets are inaccessible after you decide to unstake them.
      public static let description = Localized.tr("Localizable", "info.lock_time.description", fallback: "Lock time, also known as the unbonding or unfreezing period, is the duration during which staked assets are inaccessible after you decide to unstake them.")
    }
    public enum MaxSupply {
      /// The maximum number of coins that will ever exist.
      public static let description = Localized.tr("Localizable", "info.max_supply.description", fallback: "The maximum number of coins that will ever exist.")
      /// Max Supply
      public static let title = Localized.tr("Localizable", "info.max_supply.title", fallback: "Max Supply")
    }
    public enum NetworkFee {
      /// Every transaction on the %@ network requires a fee in %@ paid to miners to process your transaction, not Gem Wallet. Network fees varies based on network usage.
      public static func description(_ p1: Any, _ p2: Any) -> String {
        return Localized.tr("Localizable", "info.network_fee.description", String(describing: p1), String(describing: p2), fallback: "Every transaction on the %@ network requires a fee in %@ paid to miners to process your transaction, not Gem Wallet. Network fees varies based on network usage.")
      }
      /// Network Fee
      public static let title = Localized.tr("Localizable", "info.network_fee.title", fallback: "Network Fee")
    }
    public enum NoQuote {
      /// Unable to return a quote for the selected token pair, possibly due to low amount, lack of liquidity, or technical limitations.
      public static let description = Localized.tr("Localizable", "info.no_quote.description", fallback: "Unable to return a quote for the selected token pair, possibly due to low amount, lack of liquidity, or technical limitations.")
    }
    public enum OpenInterest {
      /// Open interest represents the total value of all outstanding perpetual contracts that have not been settled. It provides insight into market activity and liquidity.
      public static let description = Localized.tr("Localizable", "info.open_interest.description", fallback: "Open interest represents the total value of all outstanding perpetual contracts that have not been settled. It provides insight into market activity and liquidity.")
      /// Open Interest
      public static let title = Localized.tr("Localizable", "info.open_interest.title", fallback: "Open Interest")
    }
    public enum Perpetual {
      public enum AutoClose {
        /// Automatically close your position at set price levels. Take Profit locks in gains, Stop Loss limits losses.
        public static let description = Localized.tr("Localizable", "info.perpetual.auto_close.description", fallback: "Automatically close your position at set price levels. Take Profit locks in gains, Stop Loss limits losses.")
      }
    }
    public enum PriceImpact {
      /// Price impact is the change in token price caused by your trade size. Higher price impact means you receive fewer tokens due to low liquidity or a large order size.
      public static let description = Localized.tr("Localizable", "info.price_impact.description", fallback: "Price impact is the change in token price caused by your trade size. Higher price impact means you receive fewer tokens due to low liquidity or a large order size.")
    }
    public enum Slippage {
      /// Slippage refers to the difference between the expected price of a trade and the actual price at which it is executed.
      public static let description = Localized.tr("Localizable", "info.slippage.description", fallback: "Slippage refers to the difference between the expected price of a trade and the actual price at which it is executed.")
    }
    public enum Stake {
      public enum Apr {
        /// Annual Percentage Rate (APR) is the yearly reward rate for staking your cryptocurrency.
        public static let description = Localized.tr("Localizable", "info.stake.apr.description", fallback: "Annual Percentage Rate (APR) is the yearly reward rate for staking your cryptocurrency.")
      }
      public enum Reserved {
        /// A small amount stays in your wallet to cover fees for operations like unstaking or claiming rewards.
        public static let description = Localized.tr("Localizable", "info.stake.reserved.description", fallback: "A small amount stays in your wallet to cover fees for operations like unstaking or claiming rewards.")
        /// Reserved for Network Fee
        public static let title = Localized.tr("Localizable", "info.stake.reserved.title", fallback: "Reserved for Network Fee")
      }
    }
    public enum StakeMinimumAmount {
      /// On the %@ network, the minimum staking requirement is %@.
      public static func description(_ p1: Any, _ p2: Any) -> String {
        return Localized.tr("Localizable", "info.stake_minimum_amount.description", String(describing: p1), String(describing: p2), fallback: "On the %@ network, the minimum staking requirement is %@.")
      }
      /// Minimum Amount
      public static let title = Localized.tr("Localizable", "info.stake_minimum_amount.title", fallback: "Minimum Amount")
    }
    public enum TotalSupply {
      /// The total number of coins that exist, including locked or reserved coins.
      public static let description = Localized.tr("Localizable", "info.total_supply.description", fallback: "The total number of coins that exist, including locked or reserved coins.")
    }
    public enum Transaction {
      public enum Error {
        /// The transaction could not be completed due to an error, such as insufficient funds, invalid input, or rejection by the network. Please review the details and try again.
        public static let description = Localized.tr("Localizable", "info.transaction.error.description", fallback: "The transaction could not be completed due to an error, such as insufficient funds, invalid input, or rejection by the network. Please review the details and try again.")
      }
      public enum Pending {
        /// The transaction has been submitted and is awaiting confirmation on the network. Processing times may vary. Please check back for updates.
        public static let description = Localized.tr("Localizable", "info.transaction.pending.description", fallback: "The transaction has been submitted and is awaiting confirmation on the network. Processing times may vary. Please check back for updates.")
      }
      public enum Success {
        /// The transaction has been completed and confirmed on the network. You can review the details to verify its status.
        public static let description = Localized.tr("Localizable", "info.transaction.success.description", fallback: "The transaction has been completed and confirmed on the network. You can review the details to verify its status.")
      }
    }
    public enum WatchWallet {
      /// A wallet that you do not have access to, but you can watch its transactions and movements.
      public static let description = Localized.tr("Localizable", "info.watch_wallet.description", fallback: "A wallet that you do not have access to, but you can watch its transactions and movements.")
      /// Watch Wallet
      public static let title = Localized.tr("Localizable", "info.watch_wallet.title", fallback: "Watch Wallet")
    }
  }
  public enum Input {
    /// Please enter amount to %@
    public static func enterAmountTo(_ p1: Any) -> String {
      return Localized.tr("Localizable", "input.enter_amount_to", String(describing: p1), fallback: "Please enter amount to %@")
    }
  }
  public enum Library {
    /// Select from Photo Library
    public static let selectFromPhotoLibrary = Localized.tr("Localizable", "library.select_from_photo_library", fallback: "Select from Photo Library")
  }
  public enum Lock {
    /// 15 minutes
    public static let fifteenMinutes = Localized.tr("Localizable", "lock.fifteen_minutes", fallback: "15 minutes")
    /// 5 minutes
    public static let fiveMinutes = Localized.tr("Localizable", "lock.five_minutes", fallback: "5 minutes")
    /// Protect access to this app on your device
    public static let footer = Localized.tr("Localizable", "lock.footer", fallback: "Protect access to this app on your device")
    /// Immediately
    public static let immediately = Localized.tr("Localizable", "lock.immediately", fallback: "Immediately")
    /// 1 hour
    public static let oneHour = Localized.tr("Localizable", "lock.one_hour", fallback: "1 hour")
    /// 1 minute
    public static let oneMinute = Localized.tr("Localizable", "lock.one_minute", fallback: "1 minute")
    /// Privacy Lock
    public static let privacyLock = Localized.tr("Localizable", "lock.privacy_lock", fallback: "Privacy Lock")
    /// Require authentication
    public static let requireAuthentication = Localized.tr("Localizable", "lock.require_authentication", fallback: "Require authentication")
    /// 6 hours
    public static let sixHours = Localized.tr("Localizable", "lock.six_hours", fallback: "6 hours")
    /// Unlock
    public static let unlock = Localized.tr("Localizable", "lock.unlock", fallback: "Unlock")
  }
  public enum Markets {
    /// 24h Volume
    public static let dailyVolume = Localized.tr("Localizable", "markets.daily_volume", fallback: "24h Volume")
    /// Markets
    public static let title = Localized.tr("Localizable", "markets.title", fallback: "Markets")
    public enum State {
      public enum Empty {
        /// Your markets data will appear here
        public static let title = Localized.tr("Localizable", "markets.state.empty.title", fallback: "Your markets data will appear here")
      }
    }
  }
  public enum Networks {
    public enum State {
      public enum Empty {
        /// No networks found
        public static let searchTitle = Localized.tr("Localizable", "networks.state.empty.search_title", fallback: "No networks found")
      }
    }
  }
  public enum Nft {
    /// Collection
    public static let collection = Localized.tr("Localizable", "nft.collection", fallback: "Collection")
    /// Collections
    public static let collections = Localized.tr("Localizable", "nft.collections", fallback: "Collections")
    /// Properties
    public static let properties = Localized.tr("Localizable", "nft.properties", fallback: "Properties")
    /// Save to Photos
    public static let saveToPhotos = Localized.tr("Localizable", "nft.save_to_photos", fallback: "Save to Photos")
    /// Set as Avatar
    public static let setAsAvatar = Localized.tr("Localizable", "nft.set_as_avatar", fallback: "Set as Avatar")
    public enum Report {
      /// Report
      public static let reportButtonTitle = Localized.tr("Localizable", "nft.report.report_button_title", fallback: "Report")
      public enum Reason {
        /// Copyright
        public static let copyright = Localized.tr("Localizable", "nft.report.reason.copyright", fallback: "Copyright")
        /// Inappropriate Content
        public static let inappropriate = Localized.tr("Localizable", "nft.report.reason.inappropriate", fallback: "Inappropriate Content")
        /// Malicious
        public static let malicious = Localized.tr("Localizable", "nft.report.reason.malicious", fallback: "Malicious")
        /// Spam
        public static let spam = Localized.tr("Localizable", "nft.report.reason.spam", fallback: "Spam")
      }
    }
    public enum State {
      public enum Empty {
        /// Receive your first NFT
        public static let description = Localized.tr("Localizable", "nft.state.empty.description", fallback: "Receive your first NFT")
        /// Your NFTs will appear here️
        public static let title = Localized.tr("Localizable", "nft.state.empty.title", fallback: "Your NFTs will appear here️")
      }
    }
  }
  public enum Nodes {
    /// Gem Wallet Node
    public static let gemWalletNode = Localized.tr("Localizable", "nodes.gem_wallet_node", fallback: "Gem Wallet Node")
    public enum ImportNode {
      /// Chain ID
      public static let chainId = Localized.tr("Localizable", "nodes.import_node.chain_id", fallback: "Chain ID")
      /// In Sync
      public static let inSync = Localized.tr("Localizable", "nodes.import_node.in_sync", fallback: "In Sync")
      /// Latency
      public static let latency = Localized.tr("Localizable", "nodes.import_node.latency", fallback: "Latency")
      /// Latest Block
      public static let latestBlock = Localized.tr("Localizable", "nodes.import_node.latest_block", fallback: "Latest Block")
      /// Add node
      public static let title = Localized.tr("Localizable", "nodes.import_node.title", fallback: "Add node")
      /// Custom nodes can be malicious and may expose your transaction data or provide false information.
      public static let warningMessage = Localized.tr("Localizable", "nodes.import_node.warning_message", fallback: "Custom nodes can be malicious and may expose your transaction data or provide false information.")
    }
  }
  public enum Notifications {
    public enum Inapp {
      public enum Rewards {
        public enum Invite {
          /// Invite friends and earn rewards together
          public static let subtitle = Localized.tr("Localizable", "notifications.inapp.rewards.invite.subtitle", fallback: "Invite friends and earn rewards together")
        }
      }
      public enum State {
        public enum Empty {
          /// You'll see updates about your notifications here
          public static let description = Localized.tr("Localizable", "notifications.inapp.state.empty.description", fallback: "You'll see updates about your notifications here")
          /// No notifications yet
          public static let title = Localized.tr("Localizable", "notifications.inapp.state.empty.title", fallback: "No notifications yet")
        }
      }
    }
  }
  public enum Onboarding {
    public enum AcceptTerms {
      /// Agree and Continue
      public static let `continue` = Localized.tr("Localizable", "onboarding.accept_terms.continue", fallback: "Agree and Continue")
      /// Please read and agree to the following terms before you continue.
      public static let message = Localized.tr("Localizable", "onboarding.accept_terms.message", fallback: "Please read and agree to the following terms before you continue.")
      /// Accept Terms
      public static let title = Localized.tr("Localizable", "onboarding.accept_terms.title", fallback: "Accept Terms")
      public enum Item1 {
        /// I understand that I am solely responsible for the security and backup of my wallets, not Gem.
        public static let message = Localized.tr("Localizable", "onboarding.accept_terms.item1.message", fallback: "I understand that I am solely responsible for the security and backup of my wallets, not Gem.")
      }
      public enum Item2 {
        /// I understand that Gem is not a bank or exchange, and using it for illegal purposes is strictly prohibited.
        public static let message = Localized.tr("Localizable", "onboarding.accept_terms.item2.message", fallback: "I understand that Gem is not a bank or exchange, and using it for illegal purposes is strictly prohibited.")
      }
      public enum Item3 {
        /// I understand that if I ever lose access to my wallets, Gem is not liable and cannot help in any way.
        public static let message = Localized.tr("Localizable", "onboarding.accept_terms.item3.message", fallback: "I understand that if I ever lose access to my wallets, Gem is not liable and cannot help in any way.")
      }
    }
    public enum Security {
      public enum CreateWallet {
        public enum Confirm {
          /// I understand and want to continue
          public static let title = Localized.tr("Localizable", "onboarding.security.create_wallet.confirm.title", fallback: "I understand and want to continue")
        }
        public enum DoNotShare {
          /// Anyone who gets your secret phrase can take full control of your wallet.
          public static let subtitle = Localized.tr("Localizable", "onboarding.security.create_wallet.do_not_share.subtitle", fallback: "Anyone who gets your secret phrase can take full control of your wallet.")
          /// Do Not Share It With Anyone
          public static let title = Localized.tr("Localizable", "onboarding.security.create_wallet.do_not_share.title", fallback: "Do Not Share It With Anyone")
        }
        public enum Intro {
          /// You will get a Secret Phrase — it’s the only way to access your wallet.
          public static let title = Localized.tr("Localizable", "onboarding.security.create_wallet.intro.title", fallback: "You will get a Secret Phrase — it’s the only way to access your wallet.")
        }
        public enum KeepSafe {
          /// The secret phrase is only way to access your wallet.
          public static let subtitle = Localized.tr("Localizable", "onboarding.security.create_wallet.keep_safe.subtitle", fallback: "The secret phrase is only way to access your wallet.")
          /// Store It Somewhere Safe
          public static let title = Localized.tr("Localizable", "onboarding.security.create_wallet.keep_safe.title", fallback: "Store It Somewhere Safe")
        }
        public enum NoRecovery {
          /// If you lose your secret phrase, you lose access to your wallet.
          public static let subtitle = Localized.tr("Localizable", "onboarding.security.create_wallet.no_recovery.subtitle", fallback: "If you lose your secret phrase, you lose access to your wallet.")
          /// We Can’t Help You Recover It
          public static let title = Localized.tr("Localizable", "onboarding.security.create_wallet.no_recovery.title", fallback: "We Can’t Help You Recover It")
        }
      }
    }
  }
  public enum Permissions {
    /// Access Denied
    public static let accessDenied = Localized.tr("Localizable", "permissions.access_denied", fallback: "Access Denied")
    public enum Image {
      public enum PhotoAccess {
        public enum Denied {
          /// This app does not have permission to access your photo library. Please enable access in your device settings.
          public static let description = Localized.tr("Localizable", "permissions.image.photo_access.denied.description", fallback: "This app does not have permission to access your photo library. Please enable access in your device settings.")
        }
      }
    }
  }
  public enum Perpetual {
    /// Account Leverage
    public static let accountLeverage = Localized.tr("Localizable", "perpetual.account_leverage", fallback: "Account Leverage")
    /// All Time PnL
    public static let allTimePnl = Localized.tr("Localizable", "perpetual.all_time_pnl", fallback: "All Time PnL")
    /// Auto Close
    public static let autoClose = Localized.tr("Localizable", "perpetual.auto_close", fallback: "Auto Close")
    /// Close %@
    public static func closeDirection(_ p1: Any) -> String {
      return Localized.tr("Localizable", "perpetual.close_direction", String(describing: p1), fallback: "Close %@")
    }
    /// Close position
    public static let closePosition = Localized.tr("Localizable", "perpetual.close_position", fallback: "Close position")
    /// Direction
    public static let direction = Localized.tr("Localizable", "perpetual.direction", fallback: "Direction")
    /// Entry Price
    public static let entryPrice = Localized.tr("Localizable", "perpetual.entry_price", fallback: "Entry Price")
    /// Increase %@
    public static func increaseDirection(_ p1: Any) -> String {
      return Localized.tr("Localizable", "perpetual.increase_direction", String(describing: p1), fallback: "Increase %@")
    }
    /// Increase Position
    public static let increasePosition = Localized.tr("Localizable", "perpetual.increase_position", fallback: "Increase Position")
    /// Leverage
    public static let leverage = Localized.tr("Localizable", "perpetual.leverage", fallback: "Leverage")
    /// Long
    public static let long = Localized.tr("Localizable", "perpetual.long", fallback: "Long")
    /// Margin
    public static let margin = Localized.tr("Localizable", "perpetual.margin", fallback: "Margin")
    /// Margin Usage
    public static let marginUsage = Localized.tr("Localizable", "perpetual.margin_usage", fallback: "Margin Usage")
    /// Market Price
    public static let marketPrice = Localized.tr("Localizable", "perpetual.market_price", fallback: "Market Price")
    /// Modify
    public static let modify = Localized.tr("Localizable", "perpetual.modify", fallback: "Modify")
    /// Modify Position
    public static let modifyPosition = Localized.tr("Localizable", "perpetual.modify_position", fallback: "Modify Position")
    /// Open %@
    public static func openDirection(_ p1: Any) -> String {
      return Localized.tr("Localizable", "perpetual.open_direction", String(describing: p1), fallback: "Open %@")
    }
    /// PnL
    public static let pnl = Localized.tr("Localizable", "perpetual.pnl", fallback: "PnL")
    /// Position
    public static let position = Localized.tr("Localizable", "perpetual.position", fallback: "Position")
    /// Positions
    public static let positions = Localized.tr("Localizable", "perpetual.positions", fallback: "Positions")
    /// Reduce %@
    public static func reduceDirection(_ p1: Any) -> String {
      return Localized.tr("Localizable", "perpetual.reduce_direction", String(describing: p1), fallback: "Reduce %@")
    }
    /// Reduce Position
    public static let reducePosition = Localized.tr("Localizable", "perpetual.reduce_position", fallback: "Reduce Position")
    /// Short
    public static let short = Localized.tr("Localizable", "perpetual.short", fallback: "Short")
    /// Size
    public static let size = Localized.tr("Localizable", "perpetual.size", fallback: "Size")
    /// Unrealized PnL
    public static let unrealizedPnl = Localized.tr("Localizable", "perpetual.unrealized_pnl", fallback: "Unrealized PnL")
    /// Value
    public static let value = Localized.tr("Localizable", "perpetual.value", fallback: "Value")
    /// Volume
    public static let volume = Localized.tr("Localizable", "perpetual.volume", fallback: "Volume")
    public enum AutoClose {
      /// Expected loss
      public static let expectedLoss = Localized.tr("Localizable", "perpetual.auto_close.expected_loss", fallback: "Expected loss")
      /// Expected profit
      public static let expectedProfit = Localized.tr("Localizable", "perpetual.auto_close.expected_profit", fallback: "Expected profit")
      /// Stop loss
      public static let stopLoss = Localized.tr("Localizable", "perpetual.auto_close.stop_loss", fallback: "Stop loss")
      /// Take profit
      public static let takeProfit = Localized.tr("Localizable", "perpetual.auto_close.take_profit", fallback: "Take profit")
    }
  }
  public enum Perpetuals {
    /// Markets
    public static let markets = Localized.tr("Localizable", "perpetuals.markets", fallback: "Markets")
    /// Perpetuals
    public static let title = Localized.tr("Localizable", "perpetuals.title", fallback: "Perpetuals")
    public enum EmptyState {
      /// No markets
      public static let noMarkets = Localized.tr("Localizable", "perpetuals.empty_state.no_markets", fallback: "No markets")
      /// No markets found
      public static let noMarketsFound = Localized.tr("Localizable", "perpetuals.empty_state.no_markets_found", fallback: "No markets found")
    }
  }
  public enum PriceAlerts {
    /// Set price alert %@
    public static func addedFor(_ p1: Any) -> String {
      return Localized.tr("Localizable", "price_alerts.added_for", String(describing: p1), fallback: "Set price alert %@")
    }
    /// Alerts trigger on significant price moves.
    public static let autoFooter = Localized.tr("Localizable", "price_alerts.auto_footer", fallback: "Alerts trigger on significant price moves.")
    /// Price alert disabled for %@
    public static func disabledFor(_ p1: Any) -> String {
      return Localized.tr("Localizable", "price_alerts.disabled_for", String(describing: p1), fallback: "Price alert disabled for %@")
    }
    /// Price alert enabled for %@
    public static func enabledFor(_ p1: Any) -> String {
      return Localized.tr("Localizable", "price_alerts.enabled_for", String(describing: p1), fallback: "Price alert enabled for %@")
    }
    /// Get notified when there’s a significant price change in your favorite crypto assets.
    public static let getNotifiedExplainMessage = Localized.tr("Localizable", "price_alerts.get_notified_explain_message", fallback: "Get notified when there’s a significant price change in your favorite crypto assets.")
    public enum Direction {
      /// Decreases by
      public static let decreasesBy = Localized.tr("Localizable", "price_alerts.direction.decreases_by", fallback: "Decreases by")
      /// Increases by
      public static let increasesBy = Localized.tr("Localizable", "price_alerts.direction.increases_by", fallback: "Increases by")
      /// Over
      public static let over = Localized.tr("Localizable", "price_alerts.direction.over", fallback: "Over")
      /// Under
      public static let under = Localized.tr("Localizable", "price_alerts.direction.under", fallback: "Under")
    }
    public enum SetAlert {
      /// Current price
      public static let currentPrice = Localized.tr("Localizable", "price_alerts.set_alert.current_price", fallback: "Current price")
      /// When price decreases by
      public static let priceDecreasesBy = Localized.tr("Localizable", "price_alerts.set_alert.price_decreases_by", fallback: "When price decreases by")
      /// When price increases by
      public static let priceIncreasesBy = Localized.tr("Localizable", "price_alerts.set_alert.price_increases_by", fallback: "When price increases by")
      /// When price is over
      public static let priceOver = Localized.tr("Localizable", "price_alerts.set_alert.price_over", fallback: "When price is over")
      /// When price is under
      public static let priceUnder = Localized.tr("Localizable", "price_alerts.set_alert.price_under", fallback: "When price is under")
      /// Set target price
      public static let setTargetPrice = Localized.tr("Localizable", "price_alerts.set_alert.set_target_price", fallback: "Set target price")
      /// Set Price Alert
      public static let title = Localized.tr("Localizable", "price_alerts.set_alert.title", fallback: "Set Price Alert")
    }
    public enum State {
      public enum Empty {
        /// Enable them by adding coins to track
        public static let description = Localized.tr("Localizable", "price_alerts.state.empty.description", fallback: "Enable them by adding coins to track")
        /// Your price alerts will appear here️
        public static let title = Localized.tr("Localizable", "price_alerts.state.empty.title", fallback: "Your price alerts will appear here️")
      }
    }
  }
  public enum Receive {
    /// Receive %@
    public static func title(_ p1: Any) -> String {
      return Localized.tr("Localizable", "receive.title", String(describing: p1), fallback: "Receive %@")
    }
    /// This is your address — send only %@ on the %@ network.
    public static func warning(_ p1: Any, _ p2: Any) -> String {
      return Localized.tr("Localizable", "receive.warning", String(describing: p1), String(describing: p2), fallback: "This is your address — send only %@ on the %@ network.")
    }
  }
  public enum RecentActivity {
    /// Recents
    public static let title = Localized.tr("Localizable", "recent_activity.title", fallback: "Recents")
    public enum State {
      public enum Empty {
        /// Assets you’ve recently used will appear here
        public static let description = Localized.tr("Localizable", "recent_activity.state.empty.description", fallback: "Assets you’ve recently used will appear here")
        /// Your recents will appear here
        public static let title = Localized.tr("Localizable", "recent_activity.state.empty.title", fallback: "Your recents will appear here")
      }
    }
  }
  public enum Rewards {
    /// Get %@ for %@!
    public static func confirmRedeem(_ p1: Any, _ p2: Any) -> String {
      return Localized.tr("Localizable", "rewards.confirm_redeem", String(describing: p1), String(describing: p2), fallback: "Get %@ for %@!")
    }
    /// Insufficient points
    public static let insufficientPoints = Localized.tr("Localizable", "rewards.insufficient_points", fallback: "Insufficient points")
    /// Invited By
    public static let invitedBy = Localized.tr("Localizable", "rewards.invited_by", fallback: "Invited By")
    /// My referral code
    public static let myReferralCode = Localized.tr("Localizable", "rewards.my_referral_code", fallback: "My referral code")
    /// Nickname
    public static let nickname = Localized.tr("Localizable", "rewards.nickname", fallback: "Nickname")
    /// Points
    public static let points = Localized.tr("Localizable", "rewards.points", fallback: "Points")
    /// Referral Code
    public static let referralCode = Localized.tr("Localizable", "rewards.referral_code", fallback: "Referral Code")
    /// Referrals
    public static let referrals = Localized.tr("Localizable", "rewards.referrals", fallback: "Referrals")
    /// Join Gem Wallet with my referral link and earn rewards: %@
    public static func shareText(_ p1: Any) -> String {
      return Localized.tr("Localizable", "rewards.share_text", String(describing: p1), fallback: "Join Gem Wallet with my referral link and earn rewards: %@")
    }
    /// Rewards
    public static let title = Localized.tr("Localizable", "rewards.title", fallback: "Rewards")
    /// Username
    public static let username = Localized.tr("Localizable", "rewards.username", fallback: "Username")
    public enum ActivateReferralCode {
      /// Have a referral code from a friend?
      public static let description = Localized.tr("Localizable", "rewards.activate_referral_code.description", fallback: "Have a referral code from a friend?")
      /// Redeem code
      public static let title = Localized.tr("Localizable", "rewards.activate_referral_code.title", fallback: "Redeem code")
    }
    public enum CreateReferralCode {
      /// This will be your personal nickname for the current wallet.
      public static let info = Localized.tr("Localizable", "rewards.create_referral_code.info", fallback: "This will be your personal nickname for the current wallet.")
      /// Create Username
      public static let title = Localized.tr("Localizable", "rewards.create_referral_code.title", fallback: "Create Username")
    }
    public enum EarnPoints {
      /// Earn Points
      public static let title = Localized.tr("Localizable", "rewards.earn_points.title", fallback: "Earn Points")
    }
    public enum GetRewards {
      /// Get Rewards
      public static let title = Localized.tr("Localizable", "rewards.get_rewards.title", fallback: "Get Rewards")
    }
    public enum InviteFriends {
      /// Earn %@ points for each friend who joins.
      public static func description(_ p1: Any) -> String {
        return Localized.tr("Localizable", "rewards.invite_friends.description", String(describing: p1), fallback: "Earn %@ points for each friend who joins.")
      }
      /// Invite Friends
      public static let title = Localized.tr("Localizable", "rewards.invite_friends.title", fallback: "Invite Friends")
    }
    public enum Pending {
      /// Available in %@
      public static func description(_ p1: Any) -> String {
        return Localized.tr("Localizable", "rewards.pending.description", String(describing: p1), fallback: "Available in %@")
      }
      /// Your bonus is ready!
      public static let descriptionReady = Localized.tr("Localizable", "rewards.pending.description_ready", fallback: "Your bonus is ready!")
      /// Bonus Pending
      public static let title = Localized.tr("Localizable", "rewards.pending.title", fallback: "Bonus Pending")
    }
    public enum WaysSpend {
      /// Ways to Spend
      public static let title = Localized.tr("Localizable", "rewards.ways_spend.title", fallback: "Ways to Spend")
      public enum Asset {
        /// Get %@
        public static func title(_ p1: Any) -> String {
          return Localized.tr("Localizable", "rewards.ways_spend.asset.title", String(describing: p1), fallback: "Get %@")
        }
      }
    }
  }
  public enum Search {
    public enum State {
      public enum Empty {
        /// Check the spelling or try a new search.
        public static let description = Localized.tr("Localizable", "search.state.empty.description", fallback: "Check the spelling or try a new search.")
      }
    }
  }
  public enum SecretPhrase {
    /// Save your Secret Phrase in a secure place 
    /// that only you control.
    public static let savePhraseSafely = Localized.tr("Localizable", "secret_phrase.save_phrase_safely", fallback: "Save your Secret Phrase in a secure place \nthat only you control.")
    public enum Confirm {
      public enum QuickTest {
        /// Complete this quick test to confirm you've saved everything correctly.
        public static let title = Localized.tr("Localizable", "secret_phrase.confirm.quick_test.title", fallback: "Complete this quick test to confirm you've saved everything correctly.")
      }
    }
    public enum ContentHidden {
      /// Content hidden during screen recording
      public static let description = Localized.tr("Localizable", "secret_phrase.content_hidden.description", fallback: "Content hidden during screen recording")
    }
    public enum DoNotShare {
      /// If someone has your secret phrase they will have full control of your wallet!
      public static let description = Localized.tr("Localizable", "secret_phrase.do_not_share.description", fallback: "If someone has your secret phrase they will have full control of your wallet!")
      /// Do not share your Secret Phrase!
      public static let title = Localized.tr("Localizable", "secret_phrase.do_not_share.title", fallback: "Do not share your Secret Phrase!")
    }
    public enum ScreenshotDetected {
      /// Screenshots may be accessible to other apps, they can put your secret phrase at risk if saved this way.
      public static let description = Localized.tr("Localizable", "secret_phrase.screenshot_detected.description", fallback: "Screenshots may be accessible to other apps, they can put your secret phrase at risk if saved this way.")
      /// Screenshot Detected
      public static let title = Localized.tr("Localizable", "secret_phrase.screenshot_detected.title", fallback: "Screenshot Detected")
    }
  }
  public enum Sell {
    /// Sell %@
    public static func title(_ p1: Any) -> String {
      return Localized.tr("Localizable", "sell.title", String(describing: p1), fallback: "Sell %@")
    }
  }
  public enum Settings {
    /// About Us
    public static let aboutus = Localized.tr("Localizable", "settings.aboutus", fallback: "About Us")
    /// Community
    public static let community = Localized.tr("Localizable", "settings.community", fallback: "Community")
    /// Currency
    public static let currency = Localized.tr("Localizable", "settings.currency", fallback: "Currency")
    /// Developer
    public static let developer = Localized.tr("Localizable", "settings.developer", fallback: "Developer")
    /// Disable %@
    public static func disableValue(_ p1: Any) -> String {
      return Localized.tr("Localizable", "settings.disable_value", String(describing: p1), fallback: "Disable %@")
    }
    /// Enable Passcode
    public static let enablePasscode = Localized.tr("Localizable", "settings.enable_passcode", fallback: "Enable Passcode")
    /// Enable %@
    public static func enableValue(_ p1: Any) -> String {
      return Localized.tr("Localizable", "settings.enable_value", String(describing: p1), fallback: "Enable %@")
    }
    /// Help Center
    public static let helpCenter = Localized.tr("Localizable", "settings.help_center", fallback: "Help Center")
    /// Hide Balance
    public static let hideBalance = Localized.tr("Localizable", "settings.hide_balance", fallback: "Hide Balance")
    /// Language
    public static let language = Localized.tr("Localizable", "settings.language", fallback: "Language")
    /// Privacy Policy
    public static let privacyPolicy = Localized.tr("Localizable", "settings.privacy_policy", fallback: "Privacy Policy")
    /// Rate App
    public static let rateApp = Localized.tr("Localizable", "settings.rate_app", fallback: "Rate App")
    /// Security
    public static let security = Localized.tr("Localizable", "settings.security", fallback: "Security")
    /// Support
    public static let support = Localized.tr("Localizable", "settings.support", fallback: "Support")
    /// Terms of Services
    public static let termsOfServices = Localized.tr("Localizable", "settings.terms_of_services", fallback: "Terms of Services")
    /// Settings
    public static let title = Localized.tr("Localizable", "settings.title", fallback: "Settings")
    /// Version
    public static let version = Localized.tr("Localizable", "settings.version", fallback: "Version")
    /// Visit Website
    public static let website = Localized.tr("Localizable", "settings.website", fallback: "Visit Website")
    public enum Networks {
      /// Explorer
      public static let explorer = Localized.tr("Localizable", "settings.networks.explorer", fallback: "Explorer")
      /// Source
      public static let source = Localized.tr("Localizable", "settings.networks.source", fallback: "Source")
      /// Networks
      public static let title = Localized.tr("Localizable", "settings.networks.title", fallback: "Networks")
    }
    public enum Notifications {
      /// Notifications
      public static let title = Localized.tr("Localizable", "settings.notifications.title", fallback: "Notifications")
    }
    public enum Preferences {
      /// App Icon
      public static let appIcon = Localized.tr("Localizable", "settings.preferences.app_icon", fallback: "App Icon")
      /// Default Leverage
      public static let defaultLeverage = Localized.tr("Localizable", "settings.preferences.default_leverage", fallback: "Default Leverage")
      /// Preferences
      public static let title = Localized.tr("Localizable", "settings.preferences.title", fallback: "Preferences")
    }
    public enum PriceAlerts {
      /// Price Alerts
      public static let title = Localized.tr("Localizable", "settings.price_alerts.title", fallback: "Price Alerts")
    }
    public enum Security {
      /// Authentication
      public static let authentication = Localized.tr("Localizable", "settings.security.authentication", fallback: "Authentication")
    }
  }
  public enum SignMessage {
    /// Message
    public static let message = Localized.tr("Localizable", "sign_message.message", fallback: "Message")
    /// Sign Message
    public static let title = Localized.tr("Localizable", "sign_message.title", fallback: "Sign Message")
    /// View Full Message
    public static let viewFullMessage = Localized.tr("Localizable", "sign_message.view_full_message", fallback: "View Full Message")
  }
  public enum Social {
    /// CoinGecko
    public static let coingecko = Localized.tr("Localizable", "social.coingecko", fallback: "CoinGecko")
    /// CoinMarketCap
    public static let coinmarketcap = Localized.tr("Localizable", "social.coinmarketcap", fallback: "CoinMarketCap")
    /// Discord
    public static let discord = Localized.tr("Localizable", "social.discord", fallback: "Discord")
    /// Facebook
    public static let facebook = Localized.tr("Localizable", "social.facebook", fallback: "Facebook")
    /// GitHub
    public static let github = Localized.tr("Localizable", "social.github", fallback: "GitHub")
    /// Instagram
    public static let instagram = Localized.tr("Localizable", "social.instagram", fallback: "Instagram")
    /// Links
    public static let links = Localized.tr("Localizable", "social.links", fallback: "Links")
    /// MagicEden
    public static let magiceden = Localized.tr("Localizable", "social.magiceden", fallback: "MagicEden")
    /// OpenSea
    public static let opensea = Localized.tr("Localizable", "social.opensea", fallback: "OpenSea")
    /// Reddit
    public static let reddit = Localized.tr("Localizable", "social.reddit", fallback: "Reddit")
    /// Telegram
    public static let telegram = Localized.tr("Localizable", "social.telegram", fallback: "Telegram")
    /// TikTok
    public static let tiktok = Localized.tr("Localizable", "social.tiktok", fallback: "TikTok")
    /// Website
    public static let website = Localized.tr("Localizable", "social.website", fallback: "Website")
    /// X (formerly Twitter)
    public static let x = Localized.tr("Localizable", "social.x", fallback: "X (formerly Twitter)")
    /// YouTube
    public static let youtube = Localized.tr("Localizable", "social.youtube", fallback: "YouTube")
  }
  public enum Stake {
    /// Activating
    public static let activating = Localized.tr("Localizable", "stake.activating", fallback: "Activating")
    /// Active
    public static let active = Localized.tr("Localizable", "stake.active", fallback: "Active")
    /// Active In
    public static let activeIn = Localized.tr("Localizable", "stake.active_in", fallback: "Active In")
    /// APR %@
    public static func apr(_ p1: Any) -> String {
      return Localized.tr("Localizable", "stake.apr", String(describing: p1), fallback: "APR %@")
    }
    /// Available In
    public static let availableIn = Localized.tr("Localizable", "stake.available_in", fallback: "Available In")
    /// Awaiting Withdrawal
    public static let awaitingWithdrawal = Localized.tr("Localizable", "stake.awaiting_withdrawal", fallback: "Awaiting Withdrawal")
    /// Deactivating
    public static let deactivating = Localized.tr("Localizable", "stake.deactivating", fallback: "Deactivating")
    /// Delegations
    public static let delegations = Localized.tr("Localizable", "stake.delegations", fallback: "Delegations")
    /// Inactive
    public static let inactive = Localized.tr("Localizable", "stake.inactive", fallback: "Inactive")
    /// Lock Time
    public static let lockTime = Localized.tr("Localizable", "stake.lock_time", fallback: "Lock Time")
    /// Minimum amount
    public static let minimumAmount = Localized.tr("Localizable", "stake.minimum_amount", fallback: "Minimum amount")
    /// No active staking yet.
    public static let noActiveStaking = Localized.tr("Localizable", "stake.no_active_staking", fallback: "No active staking yet.")
    /// Pending
    public static let pending = Localized.tr("Localizable", "stake.pending", fallback: "Pending")
    /// Resource
    public static let resource = Localized.tr("Localizable", "stake.resource", fallback: "Resource")
    /// Rewards
    public static let rewards = Localized.tr("Localizable", "stake.rewards", fallback: "Rewards")
    /// Validator
    public static let validator = Localized.tr("Localizable", "stake.validator", fallback: "Validator")
    /// Validators
    public static let validators = Localized.tr("Localizable", "stake.validators", fallback: "Validators")
    /// Stake via Gem Wallet
    public static let viagem = Localized.tr("Localizable", "stake.viagem", fallback: "Stake via Gem Wallet")
    public enum Resource {
      /// Bandwidth
      public static let bandwidth = Localized.tr("Localizable", "stake.resource.bandwidth", fallback: "Bandwidth")
      /// Energy
      public static let energy = Localized.tr("Localizable", "stake.resource.energy", fallback: "Energy")
    }
    public enum State {
      public enum Empty {
        /// Stake your first %@
        public static func description(_ p1: Any) -> String {
          return Localized.tr("Localizable", "stake.state.empty.description", String(describing: p1), fallback: "Stake your first %@")
        }
        /// Your stakes will appear here
        public static let title = Localized.tr("Localizable", "stake.state.empty.title", fallback: "Your stakes will appear here")
      }
    }
  }
  public enum Swap {
    /// Minimum Receive
    public static let minReceive = Localized.tr("Localizable", "swap.min_receive", fallback: "Minimum Receive")
    /// Price Impact
    public static let priceImpact = Localized.tr("Localizable", "swap.price_impact", fallback: "Price Impact")
    /// Slippage
    public static let slippage = Localized.tr("Localizable", "swap.slippage", fallback: "Slippage")
    /// Use Minimum Amount
    public static let useMinimumAmount = Localized.tr("Localizable", "swap.use_minimum_amount", fallback: "Use Minimum Amount")
    /// You Pay
    public static let youPay = Localized.tr("Localizable", "swap.you_pay", fallback: "You Pay")
    /// You Receive
    public static let youReceive = Localized.tr("Localizable", "swap.you_receive", fallback: "You Receive")
    public enum EstimatedTime {
      /// Estimated Time
      public static let title = Localized.tr("Localizable", "swap.estimated_time.title", fallback: "Estimated Time")
    }
    public enum PriceImpactWarning {
      /// You will lose %@ of your %@ in this trade. Are you sure you want to proceed?
      public static func description(_ p1: Any, _ p2: Any) -> String {
        return Localized.tr("Localizable", "swap.price_impact_warning.description", String(describing: p1), String(describing: p2), fallback: "You will lose %@ of your %@ in this trade. Are you sure you want to proceed?")
      }
      /// High Price Impact
      public static let title = Localized.tr("Localizable", "swap.price_impact_warning.title", fallback: "High Price Impact")
    }
  }
  public enum Transaction {
    /// Date
    public static let date = Localized.tr("Localizable", "transaction.date", fallback: "Date")
    /// Recipient
    public static let recipient = Localized.tr("Localizable", "transaction.recipient", fallback: "Recipient")
    /// Sender
    public static let sender = Localized.tr("Localizable", "transaction.sender", fallback: "Sender")
    /// Status
    public static let status = Localized.tr("Localizable", "transaction.status", fallback: "Status")
    /// Swap Again
    public static let swapAgain = Localized.tr("Localizable", "transaction.swap_again", fallback: "Swap Again")
    /// View on %@
    public static func viewOn(_ p1: Any) -> String {
      return Localized.tr("Localizable", "transaction.view_on", String(describing: p1), fallback: "View on %@")
    }
    public enum Status {
      /// Successful
      public static let confirmed = Localized.tr("Localizable", "transaction.status.confirmed", fallback: "Successful")
      /// Failed
      public static let failed = Localized.tr("Localizable", "transaction.status.failed", fallback: "Failed")
      /// Pending
      public static let pending = Localized.tr("Localizable", "transaction.status.pending", fallback: "Pending")
      /// Reverted
      public static let reverted = Localized.tr("Localizable", "transaction.status.reverted", fallback: "Reverted")
    }
    public enum Title {
      /// Received
      public static let received = Localized.tr("Localizable", "transaction.title.received", fallback: "Received")
      /// Sent
      public static let sent = Localized.tr("Localizable", "transaction.title.sent", fallback: "Sent")
    }
  }
  public enum Transfer {
    /// Balance: %@
    public static func balance(_ p1: Any) -> String {
      return Localized.tr("Localizable", "transfer.balance", String(describing: p1), fallback: "Balance: %@")
    }
    /// Confirm
    public static let confirm = Localized.tr("Localizable", "transfer.confirm", fallback: "Confirm")
    /// From
    public static let from = Localized.tr("Localizable", "transfer.from", fallback: "From")
    /// Insufficient %@ balance.
    public static func insufficientBalance(_ p1: Any) -> String {
      return Localized.tr("Localizable", "transfer.insufficient_balance", String(describing: p1), fallback: "Insufficient %@ balance.")
    }
    /// Insufficient %@ balance to cover network fees.
    public static func insufficientNetworkFeeBalance(_ p1: Any) -> String {
      return Localized.tr("Localizable", "transfer.insufficient_network_fee_balance", String(describing: p1), fallback: "Insufficient %@ balance to cover network fees.")
    }
    /// Max
    public static let max = Localized.tr("Localizable", "transfer.max", fallback: "Max")
    /// Maximum Amount is %@
    public static func maximumAmount(_ p1: Any) -> String {
      return Localized.tr("Localizable", "transfer.maximum_amount", String(describing: p1), fallback: "Maximum Amount is %@")
    }
    /// Memo
    public static let memo = Localized.tr("Localizable", "transfer.memo", fallback: "Memo")
    /// A minimum %@ balance must remain after this, unless you’re using your full balance.
    public static func minimumAccountBalance(_ p1: Any) -> String {
      return Localized.tr("Localizable", "transfer.minimum_account_balance", String(describing: p1), fallback: "A minimum %@ balance must remain after this, unless you’re using your full balance.")
    }
    /// Minimum Amount is %@
    public static func minimumAmount(_ p1: Any) -> String {
      return Localized.tr("Localizable", "transfer.minimum_amount", String(describing: p1), fallback: "Minimum Amount is %@")
    }
    /// Network
    public static let network = Localized.tr("Localizable", "transfer.network", fallback: "Network")
    /// Network Fee
    public static let networkFee = Localized.tr("Localizable", "transfer.network_fee", fallback: "Network Fee")
    /// We’ve left %@ in your balance to cover future network fees.
    public static func reservedFees(_ p1: Any) -> String {
      return Localized.tr("Localizable", "transfer.reserved_fees", String(describing: p1), fallback: "We’ve left %@ in your balance to cover future network fees.")
    }
    /// Transfer
    public static let title = Localized.tr("Localizable", "transfer.title", fallback: "Transfer")
    /// To
    public static let to = Localized.tr("Localizable", "transfer.to", fallback: "To")
    public enum ActivateAsset {
      /// Activate Asset
      public static let title = Localized.tr("Localizable", "transfer.activate_asset.title", fallback: "Activate Asset")
    }
    public enum Approve {
      /// Approve
      public static let title = Localized.tr("Localizable", "transfer.approve.title", fallback: "Approve")
    }
    public enum ClaimRewards {
      /// Claim Rewards
      public static let title = Localized.tr("Localizable", "transfer.claim_rewards.title", fallback: "Claim Rewards")
    }
    public enum Freeze {
      /// Freeze
      public static let title = Localized.tr("Localizable", "transfer.freeze.title", fallback: "Freeze")
    }
    public enum Other {
      /// Other
      public static let title = Localized.tr("Localizable", "transfer.other.title", fallback: "Other")
    }
    public enum Recipient {
      /// Address or Name
      public static let addressField = Localized.tr("Localizable", "transfer.recipient.address_field", fallback: "Address or Name")
      /// My Wallets
      public static let myWallets = Localized.tr("Localizable", "transfer.recipient.my_wallets", fallback: "My Wallets")
      /// Recipient
      public static let title = Localized.tr("Localizable", "transfer.recipient.title", fallback: "Recipient")
      /// View Wallets
      public static let viewWallets = Localized.tr("Localizable", "transfer.recipient.view_wallets", fallback: "View Wallets")
    }
    public enum Redelegate {
      /// Redelegate
      public static let title = Localized.tr("Localizable", "transfer.redelegate.title", fallback: "Redelegate")
    }
    public enum Rewards {
      /// Rewards
      public static let title = Localized.tr("Localizable", "transfer.rewards.title", fallback: "Rewards")
    }
    public enum Send {
      /// Send
      public static let title = Localized.tr("Localizable", "transfer.send.title", fallback: "Send")
    }
    public enum SignTransaction {
      /// Sign Transaction
      public static let title = Localized.tr("Localizable", "transfer.sign_transaction.title", fallback: "Sign Transaction")
    }
    public enum SmartContract {
      /// Smart Contract
      public static let title = Localized.tr("Localizable", "transfer.smart_contract.title", fallback: "Smart Contract")
    }
    public enum Stake {
      /// Stake
      public static let title = Localized.tr("Localizable", "transfer.stake.title", fallback: "Stake")
    }
    public enum Unfreeze {
      /// Unfreeze
      public static let title = Localized.tr("Localizable", "transfer.unfreeze.title", fallback: "Unfreeze")
    }
    public enum Unstake {
      /// Unstake
      public static let title = Localized.tr("Localizable", "transfer.unstake.title", fallback: "Unstake")
    }
    public enum Withdraw {
      /// Withdraw
      public static let title = Localized.tr("Localizable", "transfer.withdraw.title", fallback: "Withdraw")
    }
  }
  public enum UpdateApp {
    /// Update
    public static let action = Localized.tr("Localizable", "update_app.action", fallback: "Update")
    /// Version %@ of the app is now available. Update and enjoy the latest features and improvements.
    public static func description(_ p1: Any) -> String {
      return Localized.tr("Localizable", "update_app.description", String(describing: p1), fallback: "Version %@ of the app is now available. Update and enjoy the latest features and improvements.")
    }
    /// New update available!
    public static let title = Localized.tr("Localizable", "update_app.title", fallback: "New update available!")
  }
  public enum VerifyPhrase {
    /// Confirm
    public static let title = Localized.tr("Localizable", "verify_phrase.title", fallback: "Confirm")
  }
  public enum Wallet {
    /// Available: %@
    public static func availableBalance(_ p1: Any) -> String {
      return Localized.tr("Localizable", "wallet.available_balance", String(describing: p1), fallback: "Available: %@")
    }
    /// Buy
    public static let buy = Localized.tr("Localizable", "wallet.buy", fallback: "Buy")
    /// Copy Address
    public static let copyAddress = Localized.tr("Localizable", "wallet.copy_address", fallback: "Copy Address")
    /// Create a New Wallet
    public static let createNewWallet = Localized.tr("Localizable", "wallet.create_new_wallet", fallback: "Create a New Wallet")
    /// Wallet #%d
    public static func defaultName(_ p1: Int) -> String {
      return Localized.tr("Localizable", "wallet.default_name", p1, fallback: "Wallet #%d")
    }
    /// %@ Wallet #%d
    public static func defaultNameChain(_ p1: Any, _ p2: Int) -> String {
      return Localized.tr("Localizable", "wallet.default_name_chain", String(describing: p1), p2, fallback: "%@ Wallet #%d")
    }
    /// Deposit
    public static let deposit = Localized.tr("Localizable", "wallet.deposit", fallback: "Deposit")
    /// You can view balances and transactions for this address, but **cannot send or sell funds**.
    public static let importAddressWarning = Localized.tr("Localizable", "wallet.import_address_warning", fallback: "You can view balances and transactions for this address, but **cannot send or sell funds**.")
    /// Import an Existing Wallet
    public static let importExistingWallet = Localized.tr("Localizable", "wallet.import_existing_wallet", fallback: "Import an Existing Wallet")
    /// Manage Tokens
    public static let manageTokenList = Localized.tr("Localizable", "wallet.manage_token_list", fallback: "Manage Tokens")
    /// More
    public static let more = Localized.tr("Localizable", "wallet.more", fallback: "More")
    /// Multi-Coin
    public static let multicoin = Localized.tr("Localizable", "wallet.multicoin", fallback: "Multi-Coin")
    /// Name
    public static let name = Localized.tr("Localizable", "wallet.name", fallback: "Name")
    /// Receive
    public static let receive = Localized.tr("Localizable", "wallet.receive", fallback: "Receive")
    /// Receive Collection
    public static let receiveCollection = Localized.tr("Localizable", "wallet.receive_collection", fallback: "Receive Collection")
    /// Scan
    public static let scan = Localized.tr("Localizable", "wallet.scan", fallback: "Scan")
    /// Scan QR Code
    public static let scanQrCode = Localized.tr("Localizable", "wallet.scan_qr_code", fallback: "Scan QR Code")
    /// Sell
    public static let sell = Localized.tr("Localizable", "wallet.sell", fallback: "Sell")
    /// Send
    public static let send = Localized.tr("Localizable", "wallet.send", fallback: "Send")
    /// Stake
    public static let stake = Localized.tr("Localizable", "wallet.stake", fallback: "Stake")
    /// Swap
    public static let swap = Localized.tr("Localizable", "wallet.swap", fallback: "Swap")
    /// Wallet
    public static let title = Localized.tr("Localizable", "wallet.title", fallback: "Wallet")
    /// Withdraw
    public static let withdraw = Localized.tr("Localizable", "wallet.withdraw", fallback: "Withdraw")
    public enum AddToken {
      /// Add Token
      public static let title = Localized.tr("Localizable", "wallet.add_token.title", fallback: "Add Token")
    }
    public enum Import {
      /// Import
      public static let action = Localized.tr("Localizable", "wallet.import.action", fallback: "Import")
      /// Address or Name
      public static let addressField = Localized.tr("Localizable", "wallet.import.address_field", fallback: "Address or Name")
      /// Contract or Token ID
      public static let contractAddressField = Localized.tr("Localizable", "wallet.import.contract_address_field", fallback: "Contract or Token ID")
      /// Import Wallet
      public static let title = Localized.tr("Localizable", "wallet.import.title", fallback: "Import Wallet")
    }
    public enum New {
      /// New Wallet
      public static let title = Localized.tr("Localizable", "wallet.new.title", fallback: "New Wallet")
    }
    public enum Receive {
      /// No destination tag required
      public static let noDestinationTagRequired = Localized.tr("Localizable", "wallet.receive.no_destination_tag_required", fallback: "No destination tag required")
      /// No memo required
      public static let noMemoRequired = Localized.tr("Localizable", "wallet.receive.no_memo_required", fallback: "No memo required")
    }
    public enum Watch {
      public enum Tooltip {
        /// You are watching this wallet.
        public static let title = Localized.tr("Localizable", "wallet.watch.tooltip.title", fallback: "You are watching this wallet.")
      }
    }
  }
  public enum WalletConnect {
    /// App
    public static let app = Localized.tr("Localizable", "wallet_connect.app", fallback: "App")
    /// WalletConnect
    public static let brandName = Localized.tr("Localizable", "wallet_connect.brand_name", fallback: "WalletConnect")
    /// Disconnect
    public static let disconnect = Localized.tr("Localizable", "wallet_connect.disconnect", fallback: "Disconnect")
    /// Domain
    public static let domain = Localized.tr("Localizable", "wallet_connect.domain", fallback: "Domain")
    /// No active connections
    public static let noActiveConnections = Localized.tr("Localizable", "wallet_connect.no_active_connections", fallback: "No active connections")
    /// WalletConnect
    public static let title = Localized.tr("Localizable", "wallet_connect.title", fallback: "WalletConnect")
    /// Website
    public static let website = Localized.tr("Localizable", "wallet_connect.website", fallback: "Website")
    public enum Connect {
      /// Connect
      public static let title = Localized.tr("Localizable", "wallet_connect.connect.title", fallback: "Connect")
    }
    public enum Connection {
      /// Connection
      public static let title = Localized.tr("Localizable", "wallet_connect.connection.title", fallback: "Connection")
    }
    public enum State {
      public enum Empty {
        /// Scan or paste code to connect to the DApp
        public static let description = Localized.tr("Localizable", "wallet_connect.state.empty.description", fallback: "Scan or paste code to connect to the DApp")
      }
    }
  }
  public enum Wallets {
    /// Wallets
    public static let title = Localized.tr("Localizable", "wallets.title", fallback: "Wallets")
    /// Watch
    public static let watch = Localized.tr("Localizable", "wallets.watch", fallback: "Watch")
  }
  public enum Warnings {
    /// Do not transfer funds to this %@ Multi-Signature wallet unless you are certain you control the private keys. Failure to do so could expose you to scams, and you may permanently lose your assets.
    public static func multiSignatureBlocked(_ p1: Any) -> String {
      return Localized.tr("Localizable", "warnings.multi_signature_blocked", String(describing: p1), fallback: "Do not transfer funds to this %@ Multi-Signature wallet unless you are certain you control the private keys. Failure to do so could expose you to scams, and you may permanently lose your assets.")
    }
  }
  public enum Welcome {
    /// Welcome to Gem Family
    public static let title = Localized.tr("Localizable", "welcome.title", fallback: "Welcome to Gem Family")
  }
  public enum Widget {
    public enum Medium {
      /// Track prices of top cryptocurrencies
      public static let description = Localized.tr("Localizable", "widget.medium.description", fallback: "Track prices of top cryptocurrencies")
      /// Top Crypto Price
      public static let name = Localized.tr("Localizable", "widget.medium.name", fallback: "Top Crypto Price")
    }
    public enum Small {
      /// Track Bitcoin price
      public static let description = Localized.tr("Localizable", "widget.small.description", fallback: "Track Bitcoin price")
      /// Bitcoin Price
      public static let name = Localized.tr("Localizable", "widget.small.name", fallback: "Bitcoin Price")
    }
  }
}
// swiftlint:enable explicit_type_interface function_parameter_count identifier_name line_length
// swiftlint:enable nesting type_body_length type_name vertical_whitespace_opening_braces

// MARK: - Implementation Details

extension Localized {
  private static func tr(_ table: String, _ key: String, _ args: CVarArg..., fallback value: String) -> String {
    let format = BundleToken.bundle.localizedString(forKey: key, value: value, table: table)
    return String(format: format, locale: Locale.current, arguments: args)
  }
}

// swiftlint:disable convenience_type
private final class BundleToken {
  static let bundle: Bundle = {
    #if SWIFT_PACKAGE
    return Bundle.module
    #else
    return Bundle(for: BundleToken.self)
    #endif
  }()
}
// swiftlint:enable convenience_type
