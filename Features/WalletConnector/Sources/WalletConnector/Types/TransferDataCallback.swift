// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public final class TransferDataCallback<T: Identifiable & Sendable>: Sendable, Identifiable {
    public typealias ConfirmTransferDelegate = @Sendable (Result<String, any Error>) -> Void

    public let payload: T
    public let delegate: ConfirmTransferDelegate

    public init(
        payload: T,
        delegate: @escaping ConfirmTransferDelegate
    ) {
        self.payload = payload
        self.delegate = delegate
    }

    public var id: any Hashable { payload.id }
}
