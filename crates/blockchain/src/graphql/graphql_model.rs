#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct GraphqlRequest {
    operation_name: String,
    variables: HashMap<String, String>,
    query: String,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
struct GraphqlData<T> {
    data: T,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct GraphqlError {
    message: String,
}
