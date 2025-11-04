use typeshare::typeshare;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[typeshare(swift = "Equatable, Sendable")]
pub enum TpslType {
    TakeProfit,
    StopLoss,
}
