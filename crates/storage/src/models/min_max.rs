use chrono::NaiveDateTime;

#[derive(Debug, Clone, Copy)]
pub(crate) struct DataPoint<T> {
    pub value: T,
    pub date: NaiveDateTime,
}

impl<T> From<(T, NaiveDateTime)> for DataPoint<T> {
    fn from((value, date): (T, NaiveDateTime)) -> Self {
        Self { value, date }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MinMax<T> {
    pub max: Option<DataPoint<T>>,
    pub min: Option<DataPoint<T>>,
}
