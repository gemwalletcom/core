// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Style
import SwiftUI

public struct ListItemImageStyle: Sendable {
    public let assetImage: AssetImage
    public let imageSize: CGFloat
    public let alignment: VerticalAlignment
    public let foregroundColor: Color?
    public let fontWeight: Font.Weight?
    private let cornerRadiusType: CornerRadiusType

    public var cornerRadius: CGFloat {
        switch cornerRadiusType {
        case .none: .zero
        case .rounded: imageSize / 2
        case .custom(let radius): radius
        }
    }

    public init?(
        assetImage: AssetImage?,
        imageSize: CGFloat,
        alignment: VerticalAlignment = .center,
        foregroundColor: Color? = nil,
        fontWeight: Font.Weight? = nil,
        cornerRadiusType: CornerRadiusType
    ) {
        guard let assetImage else { return nil }
        self.assetImage = assetImage
        self.imageSize = imageSize
        self.cornerRadiusType = cornerRadiusType
        self.alignment = alignment
        self.foregroundColor = foregroundColor
        self.fontWeight = fontWeight
    }
    
    public enum CornerRadiusType: Sendable {
        case none
        case rounded
        case custom(CGFloat)
    }
}

public extension ListItemImageStyle {
    static func asset(assetImage: AssetImage?) -> Self? {
        ListItemImageStyle(
            assetImage: assetImage,
            imageSize: .image.asset,
            cornerRadiusType: .rounded
        )
    }
    
    static func list(assetImage: AssetImage?, cornerRadiusType: CornerRadiusType = .none) -> Self? {
        ListItemImageStyle(
            assetImage: assetImage,
            imageSize: .list.image,
            cornerRadiusType: cornerRadiusType
        )
    }
    
    static func accessory(assetImage: AssetImage?, foregroundColor: Color? = nil, fontWeight: Font.Weight? = nil) -> Self? {
        ListItemImageStyle(
            assetImage: assetImage,
            imageSize: .space12,
            foregroundColor: foregroundColor,
            fontWeight: fontWeight,
            cornerRadiusType: .none
        )
    }

    static func settings(assetImage: AssetImage?) -> Self? {
        ListItemImageStyle(
            assetImage: assetImage,
            imageSize: .list.settings,
            cornerRadiusType: .none
        )
    }
}
