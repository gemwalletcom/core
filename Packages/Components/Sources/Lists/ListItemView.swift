// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Style

public struct ListItemView: View {
    private let model: ListItemModel

    public init(model: ListItemModel) {
        self.model = model
    }

    public init(
        title: String? = nil,
        titleStyle: TextStyle = ListItemModel.StyleDefaults.titleStyle,
        titleTag: String? = nil,
        titleTagStyle: TextStyle = ListItemModel.StyleDefaults.titleTagStyle,
        titleTagType: TitleTagType = .none,
        titleExtra: String? = nil,
        titleStyleExtra: TextStyle = ListItemModel.StyleDefaults.titleExtraStyle,
        subtitle: String? = nil,
        subtitleStyle: TextStyle = ListItemModel.StyleDefaults.subtitleStyle,
        subtitleExtra: String? = nil,
        subtitleStyleExtra: TextStyle = ListItemModel.StyleDefaults.subtitleExtraStyle,
        imageStyle: ListItemImageStyle? = nil,
        placeholders: [ListItemViewPlaceholderType] = [],
        infoAction: (() -> Void)? = nil
    ) {
        self.init(model: ListItemModel(
            title: title,
            titleStyle: titleStyle,
            titleTag: titleTag,
            titleTagStyle: titleTagStyle,
            titleTagType: titleTagType,
            titleExtra: titleExtra,
            titleStyleExtra: titleStyleExtra,
            subtitle: subtitle,
            subtitleStyle: subtitleStyle,
            subtitleExtra: subtitleExtra,
            subtitleStyleExtra: subtitleStyleExtra,
            imageStyle: imageStyle,
            placeholders: placeholders,
            infoAction: infoAction
        ))
    }
    
    public init(field: ListItemField, infoAction: (() -> Void)? = nil) {
        self.init(title: field.title, subtitle: field.value, infoAction: infoAction)
    }

    public init(
        title: TextValue? = nil,
        titleExtra: TextValue? = nil,
        titleTag: TextValue? = nil,
        titleTagType: TitleTagType = .none,
        subtitle: TextValue? = nil,
        subtitleExtra: TextValue? = nil,
        imageStyle: ListItemImageStyle? = nil,
        placeholders: [ListItemViewPlaceholderType] = [],
        infoAction: (() -> Void)? = nil
    ) {
        self.init(model: ListItemModel(
            title: title?.text,
            titleStyle: title?.style ?? ListItemModel.StyleDefaults.titleStyle,
            titleLineLimit: title?.lineLimit,
            titleTag: titleTag?.text,
            titleTagStyle: titleTag?.style ?? ListItemModel.StyleDefaults.titleTagStyle,
            titleTagLineLimit: titleTag?.lineLimit,
            titleTagType: titleTagType,
            titleExtra: titleExtra?.text,
            titleStyleExtra: titleExtra?.style ?? ListItemModel.StyleDefaults.titleExtraStyle,
            titleExtraLineLimit: titleExtra?.lineLimit,
            subtitle: subtitle?.text,
            subtitleStyle: subtitle?.style ?? ListItemModel.StyleDefaults.subtitleStyle,
            subtitleLineLimit: subtitle?.lineLimit,
            subtitleExtra: subtitleExtra?.text,
            subtitleStyleExtra: subtitleExtra?.style ?? ListItemModel.StyleDefaults.subtitleExtraStyle,
            subtitleExtraLineLimit: subtitleExtra?.lineLimit,
            imageStyle: imageStyle,
            placeholders: placeholders,
            infoAction: infoAction
        ))
    }

    public var body: some View {
        HStack(alignment: model.imageAlignment, spacing: .space12) {
            if let imageStyle = model.imageStyle {
                AssetImageView(
                    assetImage: imageStyle.assetImage,
                    size: imageStyle.imageSize,
                    style: .init(cornerRadius: imageStyle.cornerRadius)
                )
            }
            HStack {
                if let titleConfig = model.titleConfiguration {
                    TitleView(configuration: titleConfig)
                        .listRowInsets(.zero)
                }

                if model.hasSubtitlePlaceholder {
                    Spacer()
                    LoadingView(tint: model.loadingTintColor)
                } else if let subtitle = model.subtitleView {
                    Spacer(minLength: .extraSmall)
                    SubtitleView(subtitle: subtitle, subtitleExtra: model.subtitleExtraTextValue)
                }
            }
        }
    }
}

// MARK: - UI Components

// MARK: - TitleView

extension ListItemView {
    struct TitleView: View {
        private let configuration: ListItemModel.TitleConfiguration
        
        init(configuration: ListItemModel.TitleConfiguration) {
            self.configuration = configuration
        }

        var body: some View {
            VStack(alignment: .leading, spacing: .tiny) {
                HStack(spacing: .tiny) {
                    Text(configuration.title.text)
                        .textStyle(configuration.title.style)
                        .lineLimit(configuration.title.lineLimit)
                        .truncationMode(.tail)

                    if let infoAction = configuration.infoAction {
                        InfoButton(action: infoAction)
                    }

                    if let titleTag = configuration.titleTag {
                        TitleTagView(titleTag: titleTag, titleTagType: configuration.titleTagType)
                    }
                }

                if let extra = configuration.titleExtra {
                    Text(extra.text)
                        .textStyle(extra.style)
                        .lineLimit(extra.lineLimit)
                }
            }
            .padding(.trailing, .small)
        }
    }
}

// MARK: - TitleTagView

extension ListItemView {
    struct TitleTagView: View {
        let titleTag: TextValue
        let titleTagType: TitleTagType

        var body: some View {
            HStack(spacing: .tiny) {
                Text(titleTag.text)
                    .textStyle(titleTag.style)
                    .lineLimit(titleTag.lineLimit)
                    .minimumScaleFactor(0.8)

                switch titleTagType {
                case .none:
                    EmptyView()
                case let .progressView(scale):
                    LoadingView(size: .small, tint: titleTag.style.color)
                        .scaleEffect(scale)
                case .image(let image):
                    image
                }
            }
            .padding(.horizontal, .tiny)
            .padding(.vertical, .extraSmall)
            .background(titleTag.style.background)
            .cornerRadius(6)
        }
    }
}

// MARK: - ListItemView

extension ListItemView {
    struct SubtitleView: View {
        public let subtitle: TextValue
        public let subtitleExtra: TextValue?

        var body: some View {
            VStack(alignment: .trailing, spacing: .tiny) {
                Text(subtitle.text)
                    .textStyle(subtitle.style)
                    .multilineTextAlignment(.trailing)
                    .lineLimit(subtitle.lineLimit)
                    .truncationMode(.middle)

                if let extra = subtitleExtra {
                    Text(extra.text)
                        .textStyle(extra.style)
                        .multilineTextAlignment(.trailing)
                        .lineLimit(extra.lineLimit)
                        .truncationMode(.middle)
                }
            }
        }
    }
}

// MARK: - Previews

#Preview {
    List {
        Section("Simple Text Cases") {
            ListItemView(model: .text(title: "Simple Title"))
            ListItemView(model: .text(title: "Title with Subtitle", subtitle: "This is a subtitle"))
            ListItemView(model: .text(title: "Long Title Long Title Long Title", subtitle: "Long Subtitle Long Subtitle"))
            ListItemView(model: ListItemModel(
                title: "Custom with Tag",
                titleTag: "NEW",
                titleTagStyle: TextStyle(font: .footnote, color: .white, background: .blue),
                subtitle: "Custom configuration example"
            ))
            
            ListItemView(model: ListItemModel(
                title: "With Image",
                subtitle: "Custom with left image",
                imageStyle: .list(assetImage: AssetImage.image(Images.System.faceid))
            ))
            
            ListItemView(model: ListItemModel(
                title: "Loading State",
                placeholders: [.subtitle]
            ))
        }
        
        Section("Complex Custom Examples") {
            ListItemView(model: ListItemModel(
                title: "Full Featured",
                titleTag: "PRO",
                titleTagStyle: TextStyle(font: .footnote, color: .white, background: .purple),
                titleTagType: .image(Images.System.book),
                titleExtra: "Extra info",
                subtitle: "Main subtitle",
                subtitleExtra: "Extra subtitle",
                imageStyle: .list(assetImage: AssetImage.image(Images.System.eye))
            ))
        }
    }.listStyle(.insetGrouped)
}
