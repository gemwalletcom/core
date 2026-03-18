// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import XCTest
import Primitives
import SwiftHTTPClient
@testable import GemAPI

final class GemAPITests: XCTestCase {

    override func tearDown() {
        MockURLProtocol.handler = nil
        MockURLProtocol.observer = nil
        super.tearDown()
    }

    func testWalletScopedRequestWaitsForPreflight() async throws {
        let events = RequestEvents()
        let service = makeService(
            observer: { _ in
                events.record("request")
            },
            walletRequestPreflight: {
                events.record("preflight-start")
                try await Task.sleep(for: .milliseconds(20))
                events.record("preflight-end")
            }
        )

        let assetIds = try await service.getDeviceAssets(walletId: "wallet", fromTimestamp: 0)

        XCTAssertTrue(assetIds.isEmpty)
        XCTAssertEqual(events.snapshot(), ["preflight-start", "preflight-end", "request"])
    }

    func testNonWalletScopedRequestSkipsPreflight() async throws {
        let events = RequestEvents()
        let service = makeService(
            responseBody: Data("[]".utf8),
            observer: { _ in
                events.record("request")
            },
            walletRequestPreflight: {
                events.record("preflight")
            }
        )

        let names = try await service.getAddressNames(requests: [])

        XCTAssertTrue(names.isEmpty)
        XCTAssertEqual(events.snapshot(), ["request"])
    }

    func testWalletScopedPreflightFailurePreventsDispatch() async {
        let events = RequestEvents()
        let service = makeService(
            observer: { _ in
                events.record("request")
            },
            walletRequestPreflight: {
                throw TestError.failed
            }
        )

        do {
            _ = try await service.getDeviceAssets(walletId: "wallet", fromTimestamp: 0)
            XCTFail("Expected preflight to throw")
        } catch {
            XCTAssertTrue(events.snapshot().isEmpty)
        }
    }

    func testGemDeviceAPIWalletIdClassifiesWalletScopedTargets() {
        XCTAssertNil(GemDeviceAPI.getSubscriptions.walletId)
        XCTAssertEqual(GemDeviceAPI.getAssetsList(walletId: "wallet", fromTimestamp: 0).walletId, "wallet")
        XCTAssertEqual(GemDeviceAPI.getTransactions(walletId: "wallet", assetId: nil, fromTimestamp: 0).walletId, "wallet")
        XCTAssertEqual(GemDeviceAPI.getFiatQuoteUrl(walletId: "wallet", quoteId: "quote").walletId, "wallet")
    }
}

private extension GemAPITests {
    func makeService(
        responseBody: Data = Data("[]".utf8),
        observer: @escaping @Sendable (URLRequest) -> Void = { _ in },
        walletRequestPreflight: (@Sendable () async throws -> Void)? = nil
    ) -> GemAPIService {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [MockURLProtocol.self]

        MockURLProtocol.observer = observer
        MockURLProtocol.handler = { request in
            let response = HTTPURLResponse(
                url: try XCTUnwrap(request.url),
                statusCode: 200,
                httpVersion: nil,
                headerFields: [:]
            )!
            return (response, responseBody)
        }

        let session = URLSession(configuration: configuration)
        return GemAPIService(
            provider: Provider<GemAPI>(session: session),
            deviceProvider: Provider<GemDeviceAPI>(session: session),
            walletRequestPreflight: walletRequestPreflight
        )
    }
}

private final class MockURLProtocol: URLProtocol, @unchecked Sendable {
    nonisolated(unsafe) static var handler: (@Sendable (URLRequest) throws -> (HTTPURLResponse, Data))?
    nonisolated(unsafe) static var observer: (@Sendable (URLRequest) -> Void)?

    override class func canInit(with request: URLRequest) -> Bool {
        true
    }

    override class func canonicalRequest(for request: URLRequest) -> URLRequest {
        request
    }

    override func startLoading() {
        guard let handler = Self.handler else {
            client?.urlProtocol(self, didFailWithError: URLError(.badServerResponse))
            return
        }

        do {
            Self.observer?(request)
            let (response, data) = try handler(request)
            client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
            client?.urlProtocol(self, didLoad: data)
            client?.urlProtocolDidFinishLoading(self)
        } catch {
            client?.urlProtocol(self, didFailWithError: error)
        }
    }

    override func stopLoading() {}
}

private final class RequestEvents: @unchecked Sendable {
    private let lock = NSLock()
    private var events: [String] = []

    func record(_ event: String) {
        lock.lock()
        defer { lock.unlock() }
        events.append(event)
    }

    func snapshot() -> [String] {
        lock.lock()
        defer { lock.unlock() }
        return events
    }
}

private enum TestError: Error {
    case failed
}
