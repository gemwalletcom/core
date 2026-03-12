// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Components

@MainActor
public protocol ChartListViewable: AnyObject, Observable {
    var chartState: StateViewType<ChartValuesViewModel> { get }
    var selectedPeriod: ChartPeriod { get set }
    var periods: [ChartPeriod] { get }
    func fetch() async
}

extension ChartListViewable {
    public var periods: [ChartPeriod] { [.hour, .day, .week, .month, .year, .all] }
}
