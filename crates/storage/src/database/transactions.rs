use crate::{models::*, DatabaseClient};
use diesel::prelude::*;

impl DatabaseClient {
    pub fn get_transactions_by_id(&mut self, _id: &str) -> Result<Vec<Transaction>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        transactions
            .filter(id.eq(_id))
            .order(created_at.asc())
            .select(Transaction::as_select())
            .load(&mut self.connection)
    }
}
