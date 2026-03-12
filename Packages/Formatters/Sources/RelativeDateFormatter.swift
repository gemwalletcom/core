// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct RelativeDateFormatter: Sendable {
    public let calendar: Calendar

    private let relativeDateFormatter: DateFormatter
    private let timeFormatter: DateFormatter
    private let dateTimeFormatter: DateFormatter

    public init(locale: Locale = .current, timeZone: TimeZone = .current) {
        var calendar = Calendar.current
        calendar.locale = locale
        calendar.timeZone = timeZone
        self.calendar = calendar

        self.relativeDateFormatter = Self.makeDateFormatter(
            locale: locale,
            timeZone: timeZone,
            dateStyle: .medium,
            timeStyle: .none,
            relative: true
        )
        self.timeFormatter = Self.makeDateFormatter(
            locale: locale,
            timeZone: timeZone,
            dateStyle: .none,
            timeStyle: .short
        )
        self.dateTimeFormatter = Self.makeDateFormatter(
            locale: locale,
            timeZone: timeZone,
            dateStyle: .long,
            timeStyle: .short
        )
    }

    public func string(from date: Date) -> String {
        guard calendar.isDateInToday(date) || calendar.isDateInYesterday(date) else {
            return dateTimeFormatter.string(from: date)
        }

        return "\(relativeDateFormatter.string(from: date)), \(timeFormatter.string(from: date))"
    }

    public func string(fromTimestampValue value: String) -> String {
        guard let date = date(fromTimestampValue: value) else {
            return value
        }
        return string(from: date)
    }
}

private extension RelativeDateFormatter {
    static let iso8601StrategyWithFractionalSeconds = Date.ISO8601FormatStyle(includingFractionalSeconds: true)
        .year()
        .month()
        .day()
        .time(includingFractionalSeconds: true)
        .timeZone(separator: .omitted)

    static let iso8601Strategy = Date.ISO8601FormatStyle()
        .year()
        .month()
        .day()
        .time(includingFractionalSeconds: false)
        .timeZone(separator: .omitted)

    static func makeDateFormatter(
        locale: Locale,
        timeZone: TimeZone,
        dateStyle: DateFormatter.Style,
        timeStyle: DateFormatter.Style,
        relative: Bool = false
    ) -> DateFormatter {
        let formatter = DateFormatter()
        formatter.locale = locale
        formatter.timeZone = timeZone
        formatter.dateStyle = dateStyle
        formatter.timeStyle = timeStyle
        formatter.doesRelativeDateFormatting = relative
        return formatter
    }

    func date(fromTimestampValue value: String) -> Date? {
        if let timestamp = TimeInterval(value) {
            return Date(timeIntervalSince1970: timestamp)
        }

        if let date = try? Self.iso8601StrategyWithFractionalSeconds.parse(value) {
            return date
        }

        return try? Self.iso8601Strategy.parse(value)
    }
}
