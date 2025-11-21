use prometheus_client::registry::Registry;

pub trait MetricsDomain {
    fn name(&self) -> &'static str;
    fn init(&mut self, registry: &mut Registry);
}
