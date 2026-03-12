// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Charts
import Style
import Primitives
import PrimitivesComponents
import Formatters
import Components

private struct ChartKey {
    static let date = "Date"
    static let low = "Low"
    static let high = "High"
    static let open = "Open"
    static let close = "Close"
    static let price = "Price"
}

struct CandlestickChartView: View {
    private let data: [ChartCandleStick]
    private let period: ChartPeriod
    private let basePrice: Double?
    private let lineModels: [ChartLineViewModel]
    private let formatter: CurrencyFormatter

    @State private var selectedCandle: ChartCandleStick? {
        didSet {
            if let selectedCandle, selectedCandle.date != oldValue?.date {
                vibrate()
            }
        }
    }

    init(
        data: [ChartCandleStick],
        period: ChartPeriod = .day,
        basePrice: Double? = nil,
        lineModels: [ChartLineViewModel] = [],
        formatter: CurrencyFormatter = CurrencyFormatter(type: .currency, currencyCode: Currency.usd.rawValue)
    ) {
        self.data = data
        self.period = period
        self.basePrice = basePrice ?? data.first?.close
        self.lineModels = lineModels
        self.formatter = formatter
    }

    var body: some View {
        VStack {
            priceHeader
            chartView(bounds: ChartBounds(candles: data, lines: lineModels))
        }
    }

    private var priceHeader: some View {
        VStack {
            if let selectedPriceModel {
                ChartHeaderView(model: selectedPriceModel)
            } else if let currentPriceModel {
                ChartHeaderView(model: currentPriceModel)
            }
        }
        .padding(.top, Spacing.small)
        .padding(.bottom, Spacing.tiny)
    }

    private func chartView(bounds: ChartBounds) -> some View {
        let dateRange = (data.first?.date ?? Date())...(data.last?.date ?? Date())

        return Chart {
            candlestickMarks
            linesMarks(bounds)
            selectionMarks
        }
        .chartOverlay { proxy in
            GeometryReader { geometry in
                Rectangle()
                    .fill(.clear)
                    .contentShape(Rectangle())
                    .gesture(
                        DragGesture(minimumDistance: 0)
                            .onChanged { value in
                                if let candle = findCandle(location: value.location, proxy: proxy, geometry: geometry) {
                                    selectedCandle = candle
                                }
                            }
                            .onEnded { _ in
                                selectedCandle = nil
                            }
                    )

                if let selectedCandle {
                    tooltipOverlay(for: selectedCandle, proxy: proxy, geometry: geometry)
                }
            }
        }
        .chartXAxis {
            AxisMarks(position: .bottom, values: .automatic(desiredCount: 6)) { _ in
                AxisGridLine(stroke: ChartGridStyle.strokeStyle)
                    .foregroundStyle(ChartGridStyle.color)
            }
        }
        .chartYAxis {
            AxisMarks(position: .trailing, values: .automatic(desiredCount: ChartBounds.desiredTickCount)) { value in
                AxisGridLine(stroke: ChartGridStyle.strokeStyle)
                    .foregroundStyle(ChartGridStyle.color)
                AxisTick(stroke: StrokeStyle(lineWidth: ChartGridStyle.lineWidth))
                    .foregroundStyle(ChartGridStyle.color)
                AxisValueLabel {
                    if let price = value.as(Double.self) {
                        Text(price, format: bounds.axisFormat)
                            .font(.caption2)
                            .foregroundStyle(Colors.gray)
                            .padding(.horizontal, .extraSmall)
                    }
                }
            }
            if let currentPrice = data.last?.close {
                AxisMarks(position: .trailing, values: [currentPrice]) { value in
                    AxisValueLabel {
                        if let price = value.as(Double.self) {
                            Text(price, format: bounds.axisFormat)
                                .font(.caption2)
                                .foregroundStyle(Colors.whiteSolid)
                                .padding(.horizontal, .extraSmall)
                                .padding(.vertical, .space1)
                                .background(currentPriceColor)
                                .clipShape(RoundedRectangle(cornerRadius: Spacing.tiny))
                        }
                    }
                }
            }
        }
        .chartXScale(domain: dateRange)
        .chartYScale(domain: bounds.minPrice...bounds.maxPrice)
    }

    private var currentPriceColor: Color {
        guard let lastCandle = data.last else { return Colors.gray }
        return PriceChangeColor.color(for: lastCandle.close - lastCandle.open)
    }

    @ChartContentBuilder
    private var candlestickMarks: some ChartContent {
        ForEach(data, id: \.date) { candle in
            RuleMark(
                x: .value(ChartKey.date, candle.date),
                yStart: .value(ChartKey.low, candle.low),
                yEnd: .value(ChartKey.high, candle.high)
            )
            .lineStyle(StrokeStyle(lineWidth: .space1))
            .foregroundStyle(PriceChangeColor.color(for: candle.close - candle.open))

            RectangleMark(
                x: .value(ChartKey.date, candle.date),
                yStart: .value(ChartKey.open, candle.open),
                yEnd: .value(ChartKey.close, candle.close),
                width: .fixed(.space4)
            )
            .foregroundStyle(PriceChangeColor.color(for: candle.close - candle.open))
        }
    }

    @ChartContentBuilder
    private func linesMarks(_ bounds: ChartBounds) -> some ChartContent {
        ForEach(bounds.visibleLines) { line in
            RuleMark(y: .value(ChartKey.price, line.price))
                .foregroundStyle(line.color.opacity(.semiStrong))
                .lineStyle(line.lineStyle)
        }

        ForEach(Array(bounds.visibleLines.enumerated()), id: \.element.id) { index, line in
            RuleMark(y: .value(ChartKey.price, line.price))
                .foregroundStyle(.clear)
                .annotation(position: .overlay, alignment: .leading, spacing: 0) {
                    Text(line.label)
                        .font(.app.caption)
                        .foregroundStyle(Colors.whiteSolid)
                        .padding(.tiny)
                        .background(line.color)
                        .clipShape(RoundedRectangle(cornerRadius: .tiny))
                        .offset(x: labelXOffset(for: index, in: bounds))
                }
        }
    }

    private func labelXOffset(for index: Int, in bounds: ChartBounds) -> CGFloat {
        guard index > 0 else { return 0 }
        let threshold = (bounds.maxPrice - bounds.minPrice) * 0.06
        let lines = bounds.visibleLines
        let space = 115.0
        return (1...index).reduce(0.0) { offset, idx in
            abs(lines[idx].price - lines[idx - 1].price) < threshold ? offset + space : offset
        }
    }

    @ChartContentBuilder
    private var selectionMarks: some ChartContent {
        if let selectedCandle {
            PointMark(
                x: .value(ChartKey.date, selectedCandle.date),
                y: .value(ChartKey.price, selectedCandle.close)
            )
            .symbol {
                Circle()
                    .strokeBorder(Colors.blue, lineWidth: .space2)
                    .background(Circle().foregroundStyle(Colors.white))
                    .frame(width: .space12)
            }

            RuleMark(x: .value(ChartKey.date, selectedCandle.date))
                .foregroundStyle(Colors.blue)
                .lineStyle(StrokeStyle(lineWidth: .space1, dash: [5]))
        }
    }

    private var currentPriceModel: ChartHeaderViewModel? {
        guard let lastCandle = data.last, let base = basePrice else { return nil }
        return priceModel(for: lastCandle, base: base)
    }

    private var selectedPriceModel: ChartHeaderViewModel? {
        guard let selectedCandle else { return nil }
        let base = basePrice ?? data.first?.close ?? selectedCandle.close
        return priceModel(for: selectedCandle, base: base, date: selectedCandle.date)
    }

    private func priceModel(for candle: ChartCandleStick, base: Double, date: Date? = nil) -> ChartHeaderViewModel {
        ChartHeaderViewModel(
            period: period,
            date: date,
            price: candle.close,
            priceChangePercentage: PriceChangeCalculator.calculate(.percentage(from: base, to: candle.close)),
            formatter: formatter
        )
    }

    @ViewBuilder
    private func tooltipOverlay(for candle: ChartCandleStick, proxy: ChartProxy, geometry: GeometryProxy) -> some View {
        let isRightHalf: Bool = {
            guard let plotFrame = proxy.plotFrame,
                  let xPosition = proxy.position(forX: candle.date) else { return false }
            return xPosition > geometry[plotFrame].size.width / 2
        }()

        CandleTooltipView(model: CandleTooltipViewModel(candle: candle, formatter: formatter))
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: isRightHalf ? .topLeading : .topTrailing)
            .padding(.leading, Spacing.small)
            .padding(.top, Spacing.small)
            .padding(.trailing, Spacing.extraLarge + Spacing.medium)
            .transition(.opacity)
            .animation(.easeInOut(duration: Interval.AnimationDuration.fast), value: isRightHalf)
            .allowsHitTesting(false)
    }

    private func findCandle(location: CGPoint, proxy: ChartProxy, geometry: GeometryProxy) -> ChartCandleStick? {
        guard let plotFrame = proxy.plotFrame else { return nil }

        let relativeXPosition = location.x - geometry[plotFrame].origin.x

        if let date = proxy.value(atX: relativeXPosition) as Date? {
            // Find the closest candle
            var minDistance: TimeInterval = .infinity
            var closestCandle: ChartCandleStick?

            for candle in data {
                let distance = abs(candle.date.timeIntervalSince(date))
                if distance < minDistance {
                    minDistance = distance
                    closestCandle = candle
                }
            }

            return closestCandle
        }

        return nil
    }

    private func vibrate() {
        UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
}
