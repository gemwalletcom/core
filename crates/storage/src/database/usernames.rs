use crate::DatabaseClient;
use crate::models::{NewUsernameRow, UsernameRow};
use diesel::prelude::*;
use diesel::sql_types::Text;

diesel::define_sql_function!(fn lower(x: Text) -> Text);

pub enum UsernameLookup<'a> {
    Username(&'a str),
    WalletId(i32),
}

pub(crate) trait UsernamesStore {
    fn get_username(&mut self, lookup: UsernameLookup) -> Result<UsernameRow, diesel::result::Error>;
    fn username_exists(&mut self, lookup: UsernameLookup) -> Result<bool, diesel::result::Error>;
    fn create_username(&mut self, username: NewUsernameRow) -> Result<UsernameRow, diesel::result::Error>;
    fn update_username(&mut self, wallet_id: i32, new_username: &str) -> Result<UsernameRow, diesel::result::Error>;
    fn change_username(&mut self, old_username: &str, new_username: &str) -> Result<(), diesel::result::Error>;
}

impl UsernamesStore for DatabaseClient {
    fn get_username(&mut self, lookup: UsernameLookup) -> Result<UsernameRow, diesel::result::Error> {
        use crate::schema::usernames::dsl;
        match lookup {
            UsernameLookup::Username(username) => dsl::usernames
                .filter(lower(dsl::username).eq(username.to_lowercase()))
                .select(UsernameRow::as_select())
                .first(&mut self.connection),
            UsernameLookup::WalletId(wallet_id) => dsl::usernames
                .filter(dsl::wallet_id.eq(wallet_id))
                .select(UsernameRow::as_select())
                .first(&mut self.connection),
        }
    }

    fn username_exists(&mut self, lookup: UsernameLookup) -> Result<bool, diesel::result::Error> {
        match self.get_username(lookup) {
            Ok(_) => Ok(true),
            Err(diesel::result::Error::NotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn create_username(&mut self, username: NewUsernameRow) -> Result<UsernameRow, diesel::result::Error> {
        use crate::schema::usernames::dsl;
        diesel::insert_into(dsl::usernames)
            .values(&username)
            .returning(UsernameRow::as_returning())
            .get_result(&mut self.connection)
    }

    fn update_username(&mut self, wallet_id: i32, new_username: &str) -> Result<UsernameRow, diesel::result::Error> {
        use crate::schema::usernames::dsl;
        diesel::update(dsl::usernames.filter(dsl::wallet_id.eq(wallet_id)))
            .set(dsl::username.eq(new_username))
            .returning(UsernameRow::as_returning())
            .get_result(&mut self.connection)
    }

    fn change_username(&mut self, old_username: &str, new_username: &str) -> Result<(), diesel::result::Error> {
        use crate::schema::usernames::dsl;
        diesel::update(dsl::usernames.filter(dsl::username.eq(old_username)))
            .set(dsl::username.eq(new_username))
            .execute(&mut self.connection)?;
        Ok(())
    }
}
