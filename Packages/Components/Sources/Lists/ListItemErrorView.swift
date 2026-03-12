// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Style

public struct ListItemErrorView: View {
    let errorTitle: String?
    let errorSystemNameImage: String
    let errorImageColor: Color
    let error: Error
    let infoAction: (() -> Void)?

    public init(
        errorTitle: String?,
        errorSystemNameImage: String = SystemImage.errorOccurred,
        errorImageColor: Color = Colors.red,
        error: Error,
        infoAction: (() -> Void)? = nil
    ) {
        self.errorTitle = errorTitle
        self.errorSystemNameImage = errorSystemNameImage
        self.errorImageColor = errorImageColor
        self.error = error
        self.infoAction = infoAction
    }

    public var body: some View {
        if let infoAction {
            Button(action: infoAction) {
                errorContent
            }
            .tint(Colors.black)
        } else {
            errorContent
        }
    }
    
    @ViewBuilder
    private var errorContent: some View {
        HStack {
            VStack(alignment: .leading, spacing: .small) {
                HStack(spacing: .small) {
                    Image(systemName: errorSystemNameImage)
                        .foregroundStyle(errorImageColor)
                        .frame(width: .list.image, height: .list.image)
                    Text(errorTitle ?? error.localizedDescription)
                        .textStyle(.headline)
                    Spacer()
                }
                if errorTitle != nil {
                    Text(.init(error.localizedDescription))
                        .textStyle(.subHeadline)
                }
            }
            .layoutPriority(1)
            
            if infoAction != nil {
                NavigationLink.empty
            }
        }
    }
}

// MARK: - Preview

#Preview("Error States") {
    List {
        Section(header: Text("General Error")) {
            ListItemErrorView(
                errorTitle: "Error Loading Data",
                error: NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "An unexpected error occurred. Please try again."])
            )
        }

        Section(header: Text("Network Error")) {
            ListItemErrorView(
                errorTitle: "Network Error",
                error: NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Failed to load data. Check your internet connection."])
            )

            ListItemErrorView(
                errorTitle: "Insufficient funds",
                error: NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Failed to load data. Check your internet connection."]),
                infoAction: {}
            )
        }

        Section(header: Text("Operation Error")) {
            ListItemErrorView(
                errorTitle: "Operation Error",
                error: NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Unable to complete the operation. Please try again later."])
            )
        }

        Section(header: Text("Missing Error Title")) {
            ListItemErrorView(
                errorTitle: nil,
                errorSystemNameImage: SystemImage.errorOccurred,
                error: NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "An error without a specific title."])
            )
        }

        Section(header: Text("Missing Error Title & Different image")) {
            ListItemErrorView(
                errorTitle: nil,
                errorSystemNameImage: SystemImage.ellipsis,
                error: NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "An error without a specific title."])
            )
        }
        
        Section(header: Text("Error with Chevron (has infoAction)")) {
            ListItemErrorView(
                errorTitle: "Transaction Error",
                error: NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Transaction failed due to insufficient funds."]),
                infoAction: {}
            )
            
            ListItemErrorView(
                errorTitle: "Network Error",
                error: NSError(domain: "", code: -1, userInfo: [NSLocalizedDescriptionKey: "Failed to connect to the network."]),
                infoAction: {}
            )
        }
    }
    .listStyle(InsetGroupedListStyle())
    .background(Colors.grayBackground)
}
