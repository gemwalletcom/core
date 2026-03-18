// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import Style
import Components
import QRScanner
import Localization
import PrimitivesComponents

struct ImportWalletScene: View {
    enum Field {
        case input
    }

    @FocusState private var focusedField: Field?
    @State private var model: ImportWalletSceneViewModel

    init(model: ImportWalletSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        Form {
            Section {
                VStack {
                    if model.showImportTypes {
                        Picker("", selection: $model.importType) {
                            ForEach(model.importTypes) { type in
                                Text(type.title).tag(type)
                            }
                        }
                        .pickerStyle(.segmented)
                    }
                    HStack {
                        TextField(
                            model.importType.description,
                            text: $model.input,
                            axis: .vertical
                        )
                        .accessibilityIdentifier("importInputField")
                        .autocorrectionDisabled(true)
                        .textInputAutocapitalization(.never)
                        .lineLimit(8)
                        .keyboardType(.asciiCapable)
                        .frame(minHeight: 80, alignment: .top)
                        .focused($focusedField, equals: .input)
                        .padding(.top, .space12)
                        
                        if let nameRecordViewModel = model.nameRecordViewModel, model.importType == .address {
                            NameRecordView(model: nameRecordViewModel)
                        }
                    }
                    
                    HStack(alignment: .center, spacing: .medium) {
                        ListButton(
                            title: model.pasteButtonTitle,
                            image: model.pasteButtonImage,
                            action: model.onPaste
                        )
                        if model.type != .multicoin {
                            ListButton(
                                title: model.qrButtonTitle,
                                image: model.qrButtonImage,
                                action: model.onSelectScanQR
                            )
                        }
                    }
                }
                .listRowBackground(Colors.white)
            } footer: {
                if let text = model.footerText {
                    Text(.init(text))
                }
            }
            
            Section {} header: {
                VStack(alignment: .center) {
                    StateButton(
                        text: Localized.Wallet.Import.action,
                        type: .primary(model.buttonState),
                        action: model.onSelectActionButton
                    )
                    .frame(height: .scene.button.height)
                    .frame(maxWidth: .scene.button.maxWidth)
                }
                .frame(maxWidth: .infinity)
            }
            .textCase(nil)
        }
        .listSectionSpacing(.compact)
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .safeAreaView {
            if model.importType.showToolbar, model.wordsSuggestion.isNotEmpty, focusedField == .input {
                WordSuggestionView(
                    words: model.wordsSuggestion,
                    selectWord: model.onSelectWord
                )
                .clipShape(Capsule())
                .padding(.small)
                .liquidGlass()
                .background(Colors.grayBackground)
                .padding(.small)
            }
        }
        .navigationBarTitle(model.title)
        .alertSheet($model.isPresentingAlertMessage)
        .sheet(isPresented: $model.isPresentingScanner) {
            ScanQRCodeNavigationStack(action: model.onHandleScan)
        }
        .onChange(of: model.input, model.onChangeInput)
        .onChange(of: model.importType, model.onChangeImportType)
        .taskOnce {
            focusedField = .input
        }
        .detectScreenshots(docsUrl: model.docsUrl)
        .protectFromScreenRecording()
    }
}
