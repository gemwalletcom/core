// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

protocol URLRequestConvertible {
    func asRequest() throws -> URLRequest
}

struct URLRequestSequence<T: URLRequestConvertible>: AsyncSequence {
    typealias Element = (Data, URLResponse)

    let requests: [T]
    let session: URLSession

    init(requests: [T], session: URLSession = .shared) {
        self.requests = requests
        self.session = session
    }

    struct AsyncRequestIterator: AsyncIteratorProtocol {
        var requests: [T].Iterator
        let session: URLSession

        mutating func next() async throws -> Element? {
            guard let request = requests.next() else {
                return nil
            }
            let req = try request.asRequest()
            #if DEBUG
            print("==> request: \(req)")
            if let body = req.httpBody {
                print("==> body: \(String(decoding: body, as: UTF8.self))")
            }
            #endif
            let tuple = try await session.data(for: request.asRequest())
            return tuple
        }
    }

    func makeAsyncIterator() -> AsyncRequestIterator {
        return AsyncRequestIterator(requests: requests.makeIterator(), session: session)
    }
}
