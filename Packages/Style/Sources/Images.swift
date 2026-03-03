// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public enum Images {
    public enum Logo {
        public static let logo = Image(.logo)
        public static let logoDark = Image(.logoDark)
        public static let icon = Image(.gemIcon)
    }

    public enum Chains {
        public static let aptos = Image(.aptos)
        public static let arbitrum = Image(.arbitrum)
        public static let avalanchec = Image(.avalanchec)
        public static let base = Image(.base)
        public static let bitcoin = Image(.bitcoin)
        public static let bitcoincash = Image(.bitcoincash)
        public static let blast = Image(.blast)
        public static let celestia = Image(.celestia)
        public static let celo = Image(.celo)
        public static let cosmos = Image(.cosmos)
        public static let doge = Image(.doge)
        public static let ethereum = Image(.ethereum)
        public static let fantom = Image(.fantom)
        public static let gnosis = Image(.gnosis)
        public static let injective = Image(.injective)
        public static let linea = Image(.linea)
        public static let litecoin = Image(.litecoin)
        public static let manta = Image(.manta)
        public static let mantle = Image(.mantle)
        public static let near = Image(.near)
        public static let world = Image(.world)
        public static let noble = Image(.noble)
        public static let opbnb = Image(.opbnb)
        public static let optimism = Image(.optimism)
        public static let osmosis = Image(.osmosis)
        public static let polygon = Image(.polygon)
        public static let sei = Image(.sei)
        public static let smartchain = Image(.smartchain)
        public static let solana = Image(.solana)
        public static let sui = Image(.sui)
        public static let thorchain = Image(.thorchain)
        public static let ton = Image(.ton)
        public static let tron = Image(.tron)
        public static let xrp = Image(.xrp)
        public static let zksync = Image(.zksync)
        public static let stellar = Image(.stellar)
        public static let sonic = Image(.sonic)
        public static let algorand = Image(.algorand)
        public static let polkadot = Image(.polkadot)
        public static let cardano = Image(.cardano)
        public static let abstract = Image(.abstract)
        public static let berachain = Image(.berachain)
        public static let ink = Image(.ink)
        public static let unichain = Image(.unichain)
        public static let hyperliquid = Image(.hyperliquid)
        public static let monad = Image(.monad)
        public static let plasma = Image(.plasma)
        public static let zcash = Image(.zcash)
        public static let xlayer = Image(.xlayer)
        public static let stable = Image(.stable)
    }

    public enum SwapProviders {
        public static let uniswap = Image(.uniswap)
        public static let pancakeswap = Image(.pancakeswap)
        public static let across = Image(.across)
        public static let cetus = Image(.cetus)
        public static let jupiter = Image(.jupiter)
        public static let mayan = Image(.mayan)
        public static let stonfi = Image(.stonfi)
        public static let thorchain = Image(.thorchain)
        public static let orca = Image(.orca)
        public static let stargate = Image(.stargate)
        public static let raydium = Image(.raydium)
        public static let oku = Image(.oku)
        public static let wagmi = Image(.wagmi)
        public static let chainflip = Image(.chainflip)
        public static let relay = Image(.relay)
        public static let aerodrome = Image(.aerodrome)
        public static let hyperliquid = Image(.hypercore)
        public static let panora = Image(.panora)
        public static let okx = Image(.okx)
        public static let nearIntents = Image(.nearIntents)
    }

    public enum EarnProviders {
        public static let yo = Image(.yo)
    }

    public enum Fiat {
        public static let moonpay = Image(.moonpay)
        public static let transak = Image(.transak)
        public static let banxa = Image(.banxa)
        public static let mercuryo = Image(.mercuryo)
        public static let ramp = Image(.ramp)
        public static let paybis = Image(.paybis)
    }

    public enum Actions {
        public static let send = Image(.send)
        public static let swap = Image(.swap)
        public static let receive = Image(.receive)
        public static let buy = Image(.buy)
        public static let manage = Image(.manage)
        public static let sell = Image(.sell)
        public static let more = Image(.ellipsis)
        public static let avatar = Image(.avatar)
    }

    public enum Settings {
        public static let priceAlerts = Image(.settingsPriceAlerts)
        public static let currency = Image(.settingsCurrency)
        public static let rate = Image(.settingsRate)
        public static let developer = Image(.settingsDeveloper)
        public static let security = Image(.settingsSecurity)
        public static let gem = Image(.settingsGem)
        public static let support = Image(.settingsSupport)
        public static let helpCenter = Image(.settingsHelpCenter)
        public static let version = Image(.settingsVersion)
        public static let language = Image(.settingsLanguage)
        public static let wallets = Image(.settingsWallets)
        public static let networks = Image(.settingsNetworks)
        public static let walletConnect = Image(.settingsWalletConnect)
        public static let notifications = Image(.settingsNotifications)
        public static let preferences = Image(.settingsPreferences)
        public static let perpetuals = Image(.settingsPerps)
        public static let contacts = Image(.settingsContact)
    }

    public enum Perpetuals {
        public static let perpetuals = Image(.roundedPerps)
    }

    public enum Social {
        public static let github = Image(.github)
        public static let telegram = Image(.telegram)
        public static let coingecko = Image(.coingecko)
        public static let instagram = Image(.instagram)
        public static let x = Image(.x)
        public static let discord = Image(.discord)
        public static let reddit = Image(.reddit)
        public static let youtube = Image(.youtube)
        public static let website = Image(.website)
        public static let facebook = Image("") // TODO:
        public static let coinmarketcap = Image(.coinmarketcap)
        public static let opensea = Image(.opensea)
        public static let magiceden = Image(.magiceden)
        public static let tiktok = Image(.tiktok)
    }

    public enum Tabs {
        public static let settings = Image(.tabSettings)
        public static let collections = Image(.tabCollections)
        public static let activity = Image(.tabActivity)
        public static let wallet = Image(.tabWallet)
        public static let markets = Image(.tabDiscover)
    }

    public enum Transaction {
        public static let outgoing = Image(.transferOutgoing)
        public static let incoming = Image(.transferIncoming)

        public enum State {
            public static let pending = Image(.transactionStatePending)
            public static let error = Image(.transactionStateError)
            public static let success = Image(.transactionStateSuccess)
        }
    }

    public enum Wallets {
        public static let edit = Image(.edit)
        public static let create = Image(.create)
        public static let `import` = Image(.import)
        public static let watch = Image(.watch)
        public static let selected = Image(.selected)
        public static let editFilled = Image(.editFilled)
    }

    public enum NameResolve {
        public static let success = Image(.nameResolveSuccess)
        public static let error = Image(.nameResolveError)
    }

    public enum Info {
        public static let networkFee = Image(.networkFee)
    }

    public enum PriceAlert {
        public static let up = Image(.up)
        public static let down = Image(.down)
    }

    public enum EmptyContent {
        public static let activity = Image(.emptyActivity)
        public static let priceAlerts = Image(.emptyNotification)
        public static let nft = Image(.emptyIcons)
        public static let stake = Image(.emptyStake)
        public static let walletConnect = Image(.emptyDapps)
        public static let search = Image(.emptySearch)
        public static let contacts = Images.System.personBadgePlus
    }

    public enum ErrorConent {
        public static let error = Image(.error)
    }

    public enum Filters {
        public static let balance = Image(.filtersBalance)
    }

    public enum TokenStatus {
        public static let warning = Image(.verificationOrange)
        public static let risk = Image(.verificationRed)
    }

    public enum AppIcons {
        public static let primary = Image(.appIcon)
        public static let mono = Image(.appIconMono)
        public static let lava = Image(.appIconLava)
    }
}

// MARK: - Preview

#Preview {
    let imageCategories = [
        ("Logo", [
            (Images.Logo.logo, "Logo"),
            (Images.Logo.logoDark, "Logo Dark")
        ]),
        ("Chains", [
            (Images.Chains.aptos, "Aptos"),
            (Images.Chains.arbitrum, "Arbitrum"),
            (Images.Chains.avalanchec, "Avalanche C"),
            (Images.Chains.base, "Base"),
            (Images.Chains.bitcoin, "Bitcoin"),
            (Images.Chains.blast, "Blast"),
            (Images.Chains.celestia, "Celestia"),
            (Images.Chains.celo, "Celo"),
            (Images.Chains.cosmos, "Cosmos"),
            (Images.Chains.doge, "Doge"),
            (Images.Chains.ethereum, "Ethereum"),
            (Images.Chains.fantom, "Fantom"),
            (Images.Chains.gnosis, "Gnosis"),
            (Images.Chains.injective, "Injective"),
            (Images.Chains.linea, "Linea"),
            (Images.Chains.litecoin, "Litecoin"),
            (Images.Chains.manta, "Manta"),
            (Images.Chains.mantle, "Mantle"),
            (Images.Chains.near, "Near"),
            (Images.Chains.noble, "Noble"),
            (Images.Chains.opbnb, "OpBNB"),
            (Images.Chains.optimism, "Optimism"),
            (Images.Chains.osmosis, "Osmosis"),
            (Images.Chains.polygon, "Polygon"),
            (Images.Chains.sei, "Sei"),
            (Images.Chains.smartchain, "Smart Chain"),
            (Images.Chains.solana, "Solana"),
            (Images.Chains.sui, "Sui"),
            (Images.Chains.thorchain, "Thorchain"),
            (Images.Chains.ton, "Ton"),
            (Images.Chains.tron, "Tron"),
            (Images.Chains.xrp, "XRP"),
            (Images.Chains.zksync, "zkSync")
        ]),
        ("Fiat", [
            (Images.Fiat.moonpay, "Moonpay"),
            (Images.Fiat.transak, "Transak"),
            (Images.Fiat.banxa, "Banxa"),
            (Images.Fiat.mercuryo, "Mercuryo"),
            (Images.Fiat.ramp, "Ramp")
        ]),
        ("Actions", [
            (Images.Actions.send, "Send"),
            (Images.Actions.swap, "Swap"),
            (Images.Actions.receive, "Receive"),
            (Images.Actions.buy, "Buy"),
            (Images.Actions.manage, "Manage"),
            (Images.Actions.avatar, "Avatar")
        ]),
        ("Settings", [
            (Images.Settings.priceAlerts, "Price Alerts"),
            (Images.Settings.currency, "Currency"),
            (Images.Settings.rate, "Rate"),
            (Images.Settings.developer, "Developer"),
            (Images.Settings.security, "Security"),
            (Images.Settings.gem, "Gem"),
            (Images.Settings.support, "Support"),
            (Images.Settings.helpCenter, "Help Center"),
            (Images.Settings.version, "Version"),
            (Images.Settings.language, "Language"),
            (Images.Settings.wallets, "Wallets"),
            (Images.Settings.networks, "Networks"),
            (Images.Settings.walletConnect, "WalletConnect"),
            (Images.Settings.notifications, "Notifications"),
            (Images.Settings.preferences, "Preferences"),
            (Images.Settings.perpetuals, "Perpetuals")
        ]),
        ("Social", [
            (Images.Social.github, "GitHub"),
            (Images.Social.telegram, "Telegram"),
            (Images.Social.coingecko, "CoinGecko"),
            (Images.Social.instagram, "Instagram"),
            (Images.Social.x, "X (Twitter)"),
            (Images.Social.discord, "Discord"),
            (Images.Social.reddit, "Reddit"),
            (Images.Social.youtube, "YouTube")
        ]),
        ("Tags", [
            (Images.Tabs.settings, "Settings"),
            (Images.Tabs.activity, "Activity"),
            (Images.Tabs.wallet, "Wallet")
        ]),
        ("Transaction", [
            (Images.Transaction.outgoing, "Outgoing"),
            (Images.Transaction.incoming, "Incoming"),
            (Images.Transaction.State.pending, "Pending"),
            (Images.Transaction.State.error, "Error"),
            (Images.Transaction.State.success, "Success")
        ]),
        ("Wallets", [
            (Images.Wallets.edit, "Edit"),
            (Images.Wallets.create, "Create"),
            (Images.Wallets.import, "Import"),
            (Images.Wallets.watch, "Watch"),
            (Images.Wallets.selected, "Selected"),
            (Images.Wallets.editFilled, "Filled")
        ]),
        ("Name Resolve", [
            (Images.NameResolve.success, "Success"),
            (Images.NameResolve.error, "Error")
        ]),
        ("Price Alert", [
            (Images.PriceAlert.down, "down"),
            (Images.PriceAlert.up, "up")
        ])
    ]

    return List {
        ForEach(imageCategories, id: \.0) { category in
            Section(header: Text(category.0)) {
                ForEach(category.1.indices, id: \.self) { index in
                    HStack {
                        category.1[index].0
                            .resizable()
                            .aspectRatio(contentMode: .fit)
                            .frame(width: 40, height: 40)
                            .padding(.tiny)
                        Text(category.1[index].1)
                    }
                    .listRowBackground(Colors.greenLight)
                }
            }
        }
    }
    .padding()
}
