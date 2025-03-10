// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

class WrappedStruct<T: Sendable & Hashable>: NSObject {
    let value: T

    init(_ value: T) {
        self.value = value
    }

    override var hash: Int {
        value.hashValue
    }

    override func isEqual(_ object: Any?) -> Bool {
        guard let other = object as? WrappedStruct<T> else { return false }
        return value == other.value
    }
}

class Entry<Value: Sendable>: NSObject {
    let value: Value
    let expiration: Date?

    init(_ value: Value, expiration: Date? = nil) {
        self.value = value
        self.expiration = expiration
    }
}

actor Cache<Key: Sendable & Hashable, Value: Sendable> {
    private let cache = NSCache<WrappedStruct<Key>, AnyObject>()

    func set(value: Value, forKey key: Key, ttl: TimeInterval?) {
        let wrappedKey = WrappedStruct(key)
        let entry: Entry<Value>
        if let ttl {
            entry = Entry(value, expiration: Date().addingTimeInterval(ttl))
        } else {
            entry = Entry(value)
        }
        cache.setObject(entry as AnyObject, forKey: wrappedKey)
    }

    func get(key: Key) -> Value? {
        let wrappedKey = WrappedStruct(key)
        let entry = cache.object(forKey: wrappedKey) as? Entry<Value>
        // remove expired
        if let expiration = entry?.expiration, expiration < Date() {
            cache.removeObject(forKey: wrappedKey)
            return nil
        }
        return entry?.value
    }
}
