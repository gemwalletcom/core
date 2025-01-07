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
    data: Option<T>,
    errors: Option<Vec<GraphqlError>>,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct GraphqlError {
    message: String,
}
