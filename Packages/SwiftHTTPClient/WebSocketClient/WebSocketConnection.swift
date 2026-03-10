// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public actor WebSocketConnection: WebSocketConnectable {

    public private(set) var state: WebSocketState = .disconnected

    private let configuration: WebSocketConfiguration

    private var session: URLSession?
    private var task: URLSessionWebSocketTask?
    private var reconnectTask: Task<Void, Never>?
    private var continuation: AsyncStream<WebSocketEvent>.Continuation?
    private var reconnectAttempt: Int = 0
    private var pendingMessages: [URLSessionWebSocketTask.Message] = []

    public init(configuration: WebSocketConfiguration) {
        self.configuration = configuration
    }

    public init(url: URL) {
        self.init(configuration: WebSocketConfiguration(url: url))
    }

    deinit {
        task?.cancel(with: .goingAway, reason: nil)
        reconnectTask?.cancel()
        continuation?.finish()
    }

    // MARK: - Public

    public func connect() -> AsyncStream<WebSocketEvent> {
        AsyncStream { [weak self] continuation in
            guard let self else {
                continuation.finish()
                return
            }

            Task {
                await self.setupStream(continuation)
            }
        }
    }

    public func disconnect() async {
        state = .disconnected

        cancelReconnect()
        cancelTask()
        cancelPendingMessages()
        invalidateSession()

        continuation?.yield(.disconnected(nil))
        continuation?.finish()
        continuation = nil
    }

    public func send(_ data: Data) async throws {
        try await send(message: .data(data))
    }

    public func send(_ text: String) async throws {
        try await send(message: .string(text))
    }

    // MARK: - Private

    private func send(message: URLSessionWebSocketTask.Message) async throws {
        switch state {
        case .connected:
            guard let task else { throw WebSocketError.notConnected }
            try await task.send(message)
        case .connecting, .reconnecting:
            pendingMessages.append(message)
        case .disconnected:
            throw WebSocketError.notConnected
        }
    }

    private func sendPendingMessages() async {
        let messages = pendingMessages
        pendingMessages = []

        for message in messages {
            do {
                try await task?.send(message)
            } catch {
                #if DEBUG
                NSLog("WebSocket send error: \(error)")
                #endif
            }
        }
    }

    private func cancelPendingMessages() {
        pendingMessages.removeAll()
    }

    private func cancelTask() {
        task?.cancel(with: .goingAway, reason: nil)
        task = nil
    }

    private func cancelReconnect() {
        reconnectTask?.cancel()
        reconnectTask = nil
    }

    private func invalidateSession() {
        session?.invalidateAndCancel()
        session = nil
    }

    private func setupStream(_ continuation: AsyncStream<WebSocketEvent>.Continuation) {
        self.continuation = continuation

        continuation.onTermination = { [weak self] _ in
            Task {
                await self?.handleStreamTermination()
            }
        }

        resetReconnectionAttempt()
        startConnection()
    }

    private func handleStreamTermination() {
        guard state != .disconnected else { return }

        cancelTask()
        cancelReconnect()
        cancelPendingMessages()
        invalidateSession()

        state = .disconnected
    }

    private func startConnection() {
        state = .connecting

        guard let request = try? configuration.requestProvider.makeRequest() else {
            scheduleReconnect(with: WebSocketError.notConnected)
            return
        }

        let delegate = WebSocketSessionDelegate(
            didOpen: { [weak self] in
                Task { await self?.didOpen() }
            },
            didClose: { [weak self] closeCode, reason in
                Task { await self?.didClose(closeCode: closeCode, reason: reason) }
            }
        )

        session = URLSession(
            configuration: configuration.sessionConfiguration,
            delegate: delegate,
            delegateQueue: nil
        )

        task = session?.webSocketTask(with: request)
        task?.resume()

        listen()
    }

    private func didOpen() {
        guard state == .connecting else { return }

        state = .connected
        resetReconnectionAttempt()
        continuation?.yield(.connected)

        Task {
            await sendPendingMessages()
        }
    }

    private func didClose(closeCode: URLSessionWebSocketTask.CloseCode, reason: Data?) {
        guard state != .disconnected else { return }
        scheduleReconnect(with: nil)
    }

    private func listen() {
        task?.receive { [weak self] result in
            Task {
                await self?.handleReceive(result)
            }
        }
    }

    private func handleReceive(_ result: Result<URLSessionWebSocketTask.Message, Error>) {
        switch result {
        case .success(let message):
            handleMessage(message)
            listen()

        case .failure(let error):
            handleError(error)
        }
    }

    private func handleMessage(_ message: URLSessionWebSocketTask.Message) {
        switch message {
        case .data(let data):
            continuation?.yield(.message(data))
        case .string(let text):
            if let data = text.data(using: .utf8) {
                continuation?.yield(.message(data))
            }
        @unknown default:
            break
        }
    }

    private func handleError(_ error: Error) {
        guard state != .disconnected else { return }

        if let urlError = error as? URLError, urlError.code == .cancelled {
            return
        }

        scheduleReconnect(with: error)
    }

    private func scheduleReconnect(with error: Error?) {
        guard reconnectTask == nil else { return }

        cancelTask()

        guard state != .disconnected else { return }

        state = .reconnecting
        continuation?.yield(.disconnected(error))

        let delay = configuration.reconnection.reconnectAfter(attempt: reconnectAttempt)
        reconnectAttempt += 1

        reconnectTask = Task { [weak self] in
            try? await Task.sleep(for: .seconds(delay))
            await self?.finishReconnect()
        }
    }

    private func finishReconnect() {
        reconnectTask = nil

        guard state == .reconnecting, task == nil else { return }
        startConnection()
    }

    private func resetReconnectionAttempt() {
        reconnectAttempt = 0
    }
}
