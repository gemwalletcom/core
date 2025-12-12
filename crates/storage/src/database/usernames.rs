use crate::DatabaseClient;
use crate::models::Username;
use diesel::prelude::*;

pub enum UsernameLookup<'a> {
    Username(&'a str),
    Address(&'a str),
}

pub(crate) trait UsernamesStore {
    fn get_username(&mut self, lookup: UsernameLookup) -> Result<Username, diesel::result::Error>;
    fn username_exists(&mut self, lookup: UsernameLookup) -> Result<bool, diesel::result::Error>;
    fn create_username(&mut self, username: Username) -> Result<Username, diesel::result::Error>;
    fn update_username(&mut self, address: &str, new_username: &str) -> Result<Username, diesel::result::Error>;
}

impl UsernamesStore for DatabaseClient {
    fn get_username(&mut self, lookup: UsernameLookup) -> Result<Username, diesel::result::Error> {
        use crate::schema::usernames::dsl;
        match lookup {
            UsernameLookup::Username(username) => dsl::usernames
                .filter(dsl::username.eq(username))
                .select(Username::as_select())
                .first(&mut self.connection),
            UsernameLookup::Address(address) => dsl::usernames
                .filter(dsl::address.eq(address))
                .select(Username::as_select())
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

    fn create_username(&mut self, username: Username) -> Result<Username, diesel::result::Error> {
        use crate::schema::usernames::dsl;
        diesel::insert_into(dsl::usernames)
            .values(&username)
            .returning(Username::as_returning())
            .get_result(&mut self.connection)
    }

    fn update_username(&mut self, address: &str, new_username: &str) -> Result<Username, diesel::result::Error> {
        use crate::schema::usernames::dsl;
        diesel::update(dsl::usernames.filter(dsl::address.eq(address).and(dsl::username.eq(address))))
            .set(dsl::username.eq(new_username))
            .returning(Username::as_returning())
            .get_result(&mut self.connection)
    }
}
