// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import WebSocketClient

public actor StreamObserverService: Sendable {
    private let subscriptionService: StreamSubscriptionService
    private let eventService: StreamEventService
    private let webSocket: any WebSocketConnectable
    private let decoder = JSONDateDecoder.standard
    private var observeTask: Task<Void, Never>?

    public init(
        subscriptionService: StreamSubscriptionService,
        eventService: StreamEventService,
        webSocket: any WebSocketConnectable
    ) {
        self.subscriptionService = subscriptionService
        self.eventService = eventService
        self.webSocket = webSocket
    }

    deinit {
        observeTask?.cancel()
    }

    // MARK: - Public API

    public func connect() {
        guard observeTask == nil else { return }

        observeTask = Task { [weak self] in
            guard let self else { return }
            await self.observeConnection()
        }
    }

    public func disconnect() async {
        guard observeTask != nil else { return }

        observeTask?.cancel()
        observeTask = nil

        await webSocket.disconnect()
    }

    // MARK: - Private

    private func observeConnection() async {
        for await event in await webSocket.connect() {
            guard !Task.isCancelled else { break }

            switch event {
            case .connected: await subscriptionService.resubscribe()
            case .message(let data): await handleMessage(data)
            case .disconnected: break
            }
        }
    }

    private func handleMessage(_ data: Data) async {
        do {
            let event = try decoder.decode(StreamEvent.self, from: data)
            await eventService.handle(event)
        } catch {
            debugLog("stream observer: handleMessage error: \(error)")
        }
    }
}
