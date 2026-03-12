// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Style
import SwiftUI

internal import Charts

public struct ChartView: View {
    private enum ChartKey {
        static let date = "Date"
        static let value = "Value"
    }

    private enum Metrics {
        static let lineWidth: CGFloat = 2.5
        static let selectionDotSize: CGFloat = 12
        static let labelWidth: CGFloat = 88
    }

    private let model: ChartValuesViewModel

    @State private var selectedElement: ChartDateValue?

    public init(model: ChartValuesViewModel) {
        self.model = model
    }

    public var body: some View {
        VStack(spacing: 0) {
            priceHeader
            chart
        }
    }
}

// MARK: - UI

extension ChartView {
    private var priceHeader: some View {
        Group {
            if let element = selectedElement {
                ChartHeaderView(model: model.headerViewModel(for: element))
            } else if let chartHeaderViewModel = model.chartHeaderViewModel {
                ChartHeaderView(model: chartHeaderViewModel)
            }
        }
        .padding(.top, Spacing.small)
        .padding(.bottom, Spacing.tiny)
    }

    private var chart: some View {
        Chart {
            ForEach(model.charts, id: \.date) { item in
                AreaMark(
                    x: .value(ChartKey.date, item.date),
                    y: .value(ChartKey.value, item.value)
                )
                .interpolationMethod(.catmullRom)
                .foregroundStyle(areaGradient)
                .alignsMarkStylesWithPlotArea()

                LineMark(
                    x: .value(ChartKey.date, item.date),
                    y: .value(ChartKey.value, item.value)
                )
                .lineStyle(StrokeStyle(lineWidth: Metrics.lineWidth, lineCap: .round, lineJoin: .round))
                .foregroundStyle(model.lineColor)
                .interpolationMethod(.catmullRom)
            }

            if let selectedElement {
                RuleMark(x: .value(ChartKey.date, selectedElement.date))
                    .foregroundStyle(model.lineColor.opacity(.medium))
                    .lineStyle(StrokeStyle(lineWidth: 1, dash: [4, 4]))

                PointMark(x: .value(ChartKey.date, selectedElement.date), y: .value(ChartKey.value, selectedElement.value))
                    .symbol {
                        Circle()
                            .fill(
                                RadialGradient(
                                    colors: [Colors.white, model.lineColor.opacity(.strong)],
                                    center: .center,
                                    startRadius: 0,
                                    endRadius: Metrics.selectionDotSize / 2
                                )
                            )
                            .frame(width: Metrics.selectionDotSize, height: Metrics.selectionDotSize)
                            .shadow(color: model.lineColor.opacity(.semiStrong), radius: 6)
                            .overlay(Circle().strokeBorder(model.lineColor, lineWidth: Metrics.lineWidth))
                    }
            }
        }
        .chartOverlay { proxy in
            GeometryReader { geometry in
                Rectangle()
                    .fill(.clear)
                    .contentShape(Rectangle())
                    .gesture(
                        DragGesture(minimumDistance: 0)
                            .onChanged { value in
                                onDragChange(location: value.location, proxy: proxy, geometry: geometry)
                            }
                            .onEnded { _ in
                                onDragEnd()
                            }
                    )

                if let lastPoint = model.charts.last,
                   let plotFrame = proxy.plotFrame,
                   let xPos = proxy.position(forX: lastPoint.date),
                   let yPos = proxy.position(forY: lastPoint.value) {
                    let origin = geometry[plotFrame].origin
                    PulsingDotView(color: model.lineColor)
                        .position(x: origin.x + xPos, y: origin.y + yPos)
                        .opacity(selectedElement == nil ? 1 : 0)
                        .animation(.easeInOut(duration: .AnimationDuration.normal), value: selectedElement == nil)
                }
            }
        }
        .padding(.vertical, Spacing.large)
        .chartXAxis(.hidden)
        .chartYAxis(.hidden)
        .chartYScale(domain: model.values.yScale)
        .chartXScale(domain: model.values.xScale)
        .chartBackground { proxy in
            GeometryReader { geometry in
                if let plotFrame = proxy.plotFrame {
                    let chartBounds = geometry[plotFrame]

                    if let lowerBoundX = proxy.position(forX: model.values.lowerBoundDate) {
                        boundLabel(model.lowerBoundValueText)
                            .offset(x: labelX(lowerBoundX, geoWidth: geometry.size.width), y: chartBounds.maxY + Spacing.small)
                    }

                    if let upperBoundX = proxy.position(forX: model.values.upperBoundDate) {
                        boundLabel(model.upperBoundValueText)
                            .offset(x: labelX(upperBoundX, geoWidth: geometry.size.width), y: chartBounds.minY - Spacing.large)
                    }
                }
            }
        }
    }

    private var areaGradient: LinearGradient {
        .linearGradient(
            stops: [
                .init(color: model.lineColor.opacity(.opacity45), location: 0),
                .init(color: model.lineColor.opacity(.opacity38), location: 0.25),
                .init(color: model.lineColor.opacity(.opacity28), location: 0.5),
                .init(color: model.lineColor.opacity(.light), location: 0.75),
                .init(color: model.lineColor.opacity(.faint), location: 0.92),
                .init(color: model.lineColor.opacity(0), location: 1.0)
            ],
            startPoint: .top,
            endPoint: .bottom
        )
    }

    private func boundLabel(_ text: String) -> some View {
        Text(text)
            .font(.caption2)
            .foregroundStyle(Colors.gray)
            .frame(width: Metrics.labelWidth)
    }

    private func labelX(_ x: CGFloat, geoWidth: CGFloat) -> CGFloat {
        let half = Metrics.labelWidth / 2
        return x < half ? x - half / 2 : min(x - half, geoWidth - Metrics.labelWidth)
    }
}

// MARK: - Actions

extension ChartView {
    private func onDragChange(location: CGPoint, proxy: ChartProxy, geometry: GeometryProxy) {
        guard let plotFrame = proxy.plotFrame else { return }

        let relativeX = location.x - geometry[plotFrame].origin.x
        guard let targetDate = proxy.value(atX: relativeX) as Date?,
              let element = model.charts.min(by: { abs($0.date.distance(to: targetDate)) < abs($1.date.distance(to: targetDate)) }) else {
            return
        }

        if element.date != selectedElement?.date {
            UIImpactFeedbackGenerator(style: .light).impactOccurred()
        }
        selectedElement = element
    }

    private func onDragEnd() {
        selectedElement = nil
    }
}
