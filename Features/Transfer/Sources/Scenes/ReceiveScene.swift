// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Style
import PrimitivesComponents

public struct ReceiveScene: View {
    @State private var model: ReceiveViewModel

    public init(model: ReceiveViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        VStack(spacing: .large) {
            VStack {
                Spacer()
                VStack(spacing: .medium) {
                    AssetPreviewView(model: model.assetModel)

                    Button(action: model.onCopyAddress) {
                        VStack(spacing: .medium) {
                            VStack {
                                if let image = model.renderedImage {
                                    qrCodeView(image: image)
                                } else {
                                    LoadingView()
                                }
                            }
                            .frame(size: model.qrSize)
                            
                            Text(model.address.preventingHyphenation)
                                .multilineTextAlignment(.center)
                                .textStyle(TextStyle(font: .subheadline, color: Colors.secondaryText, fontWeight: .medium))
                                .fixedSize(horizontal: false, vertical: true)
                                .frame(maxWidth: model.qrSize)
                                .accessibilityIdentifier(model.address)
                        }
                    }
                    .buttonStyle(.scale)
                    .padding(.medium)
                    .background(
                        RoundedRectangle(cornerRadius: .medium)
                            .fill(Colors.listStyleColor)
                            .shadow(color: Color.black.opacity(.opacity25), radius: Sizing.shadow.radius, x: .zero, y: Sizing.shadow.yOffset)
                    )
                }
                Text(model.warningMessage)
                    .textStyle(.subHeadline)
                    .multilineTextAlignment(.center)
                    .padding(.top, .small)
                    .frame(maxWidth: model.qrSize + .extraLarge)
                Spacer()
            }
            .frame(maxWidth: .scene.button.maxWidth)

            StateButton(
                text: model.copyTitle,
                image: Images.System.copy,
                action: model.onCopyAddress
            )
            .frame(maxWidth: .scene.button.maxWidth)
        }
        .padding(.bottom, .scene.bottom)
        .frame(maxWidth: .infinity)
        .background(Colors.grayBackground)
        .navigationBarTitle(model.title)
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(action: model.onShareSheet) {
                    Images.System.share
                }
            }
        }
        .sheet(isPresented: $model.isPresentingShareSheet) {
            ShareSheet(activityItems: model.activityItems(qrImage: model.renderedImage))
        }
        .copyToast(
            model: model.copyModel,
            isPresenting: $model.isPresentingCopyToast
        )
        .task {
            await model.onLoadImage()
        }
        .taskOnce(model.onTaskOnce)
    }
}

// MARK: - UI Components

extension ReceiveScene {
    @ViewBuilder
    private func qrCodeView(image: UIImage) -> some View {
        Image(uiImage: image)
            .resizable()
            .scaledToFit()
            .padding(.extraSmall)
            .background(Color.white)
            .clipShape(RoundedRectangle(cornerRadius: .small))
    }
}
