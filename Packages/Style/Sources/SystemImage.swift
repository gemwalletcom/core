// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public struct SystemImage {
    public static let settings = "gearshape"
    public static let qrCodeViewfinder = "qrcode.viewfinder"
    public static let qrCode = "qrcode"
    public static let paste = "doc.on.clipboard"
    public static let copy = "doc.on.doc"
    public static let arrowSwap = "arrow.trianglehead.2.clockwise"
    public static let paperplane = "paperplane"
    public static let chevronDown = "chevron.down"
    public static let chevronRight = "chevron.right"
    public static let clear = "multiply.circle.fill"
    public static let hide = "eye.slash.fill"
    public static let faceid = "faceid"
    public static let network = "network"
    public static let globe = "globe"
    public static let share = "square.and.arrow.up"
    public static let lock = "lock"
    public static let delete = "trash"
    public static let checkmark = "checkmark"
    public static let ellipsis = "ellipsis"
    public static let info = "info.circle"
    public static let eyeglasses = "eyeglasses"
    public static let plus = "plus"
    public static let plusCircle = "plus.circle"
    public static let minusCircle = "minus.circle"
    public static let eye = "eye.fill"
    public static let search = "magnifyingglass"
    public static let exclamationmarkTriangleFill = "exclamationmark.triangle.fill"
    public static let exclamationmarkTriangle = "exclamationmark.triangle"
    public static let gallery = "photo.on.rectangle.angled"
    public static let photo = "photo"
    public static let xmarkCircle = "xmark.circle.fill"
    public static let xmark = "xmark"
    public static let bell = "bell"
    public static let bellFill = "bell.fill"
    public static let pin = "pin"
    public static let unpin = "pin.slash"
    public static let filter = "line.horizontal.3.decrease.circle"
    public static let filterFill = "line.horizontal.3.decrease.circle.fill"
    public static let book = "book"
    public static let starFill = "star.fill"
    public static let textPageFill = "text.page.fill"
    public static let emoji = "face.smiling"
    public static let checkmarkCircle = "checkmark.circle"
    public static let circle = "circle"
    public static let dollarsign = "dollarsign"
    public static let wallet = "wallet.pass"
    public static let bitcoin = "bitcoinsign.arrow.trianglehead.counterclockwise.rotate.90"
    public static let arrowTriangleUp = "arrowtriangle.up.fill"
    public static let arrowTriangleDown = "arrowtriangle.down.fill"
    public static let person = "person"
    public static let personBadgePlus = "person.crop.circle.badge.plus"
    public static let chartLineUptrendXyaxis = "chart.line.uptrend.xyaxis"
    public static let checkmarkSealFill = "checkmark.seal.fill"

    // specific to Gem style
    public static let errorOccurred = exclamationmarkTriangleFill
}

// MARK: - Previews

#Preview {
    let symbols = [
        (SystemImage.settings, "Settings"),
        (SystemImage.qrCode, "QR Code"),
        (SystemImage.paste, "Paste"),
        (SystemImage.copy, "Copy"),
        (SystemImage.chevronDown, "Chevron Down"),
        (SystemImage.clear, "Clear"),
        (SystemImage.hide, "Hide"),
        (SystemImage.faceid, "Face ID"),
        (SystemImage.network, "Network"),
        (SystemImage.globe, "Globe"),
        (SystemImage.share, "Share"),
        (SystemImage.lock, "Lock"),
        (SystemImage.delete, "Delete"),
        (SystemImage.checkmark, "Checkmark"),
        (SystemImage.ellipsis, "Ellipsis"),
        (SystemImage.info, "Info"),
        (SystemImage.eyeglasses, "Eyeglasses"),
        (SystemImage.plus, "Plus"),
        (SystemImage.eye, "Eye"),
        (SystemImage.errorOccurred, "Error Ocurred"),
        (SystemImage.gallery, "Gallery"),
        (SystemImage.xmarkCircle, "X MarkCircle"),
        (SystemImage.xmark, "X Mark"),
        (SystemImage.bell, "Bell"),
        (SystemImage.pin, "Pin"),
        (SystemImage.unpin, "Unpin"),
        (SystemImage.filter, "Filter"),
        (SystemImage.filterFill, "Filter Fill"),
        (SystemImage.book, "book"),
    ]

    return List {
        ForEach(symbols, id: \.1) { symbol in
            Section(header: Text(symbol.1)) {
                Image(systemName: symbol.0)
                    .resizable()
                    .aspectRatio(contentMode: .fit)
                    .frame(width: .list.image, height: .list.image)
                    .padding(.extraSmall)
            }
        }
    }
    .listStyle(InsetGroupedListStyle())
    .padding()
}
