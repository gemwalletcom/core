// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Preferences
import GemAPI

public actor ConfigService {
    private let configPreferences: ConfigPreferences
    private let apiService: any GemAPIConfigService
    private var updateTask: Task<ConfigResponse, Error>?

    public init(
        configPreferences: ConfigPreferences = .standard,
        apiService: any GemAPIConfigService
    ) {
        self.configPreferences = configPreferences
        self.apiService = apiService
    }

    public func updateConfig() async throws {
        if let task = updateTask {
            _ = try await task.value
            return
        }

        updateTask = Task {
            let config = try await apiService.getConfig()
            configPreferences.config = config
            return config
        }

        defer { updateTask = nil }
        _ = try await updateTask?.value
    }

    public func getConfig() async -> ConfigResponse? {
        if let updateTask {
            return try? await updateTask.value
        }
        return configPreferences.config
    }
}
