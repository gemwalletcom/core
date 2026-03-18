// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

actor DeviceSyncCoordinator {
    private var syncTask: Task<Void, Error>?

    func waitForSyncIfNeeded() async throws {
        if let syncTask {
            try await syncTask.value
        }
    }

    func coordinate(_ operation: @escaping @Sendable () async throws -> Void) async throws {
        if let syncTask {
            return try await syncTask.value
        }

        let task = Task {
            try await operation()
        }
        syncTask = task
        defer { syncTask = nil }
        try await task.value
    }
}
