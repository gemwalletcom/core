use crate::{models::*, DatabaseClient};

use diesel::prelude::*;

pub trait ParserStateStore {
    fn get_parser_state(&mut self, chain: &str) -> Result<ParserState, diesel::result::Error>;
    fn add_parser_state(&mut self, chain: &str) -> Result<usize, diesel::result::Error>;
    fn get_parser_states(&mut self) -> Result<Vec<ParserState>, diesel::result::Error>;
    fn set_parser_state_latest_block(&mut self, chain: &str, block: i32) -> Result<usize, diesel::result::Error>;
    fn set_parser_state_current_block(&mut self, chain: &str, block: i32) -> Result<usize, diesel::result::Error>;
}

pub trait ParserStateRepository {
    //     fn get_parser_state(&mut self, _chain: Chain) -> Result<ParserState, diesel::result::Error>;
    //     fn add_parser_state(&mut self, _chain: Chain) -> Result<usize, diesel::result::Error>;
    //     fn get_parser_states(&mut self) -> Result<Vec<ParserState>, diesel::result::Error>;
    //     fn set_parser_state_latest_block(&mut self, _chain: Chain, block: i32) -> Result<usize, diesel::result::Error>;
}

impl ParserStateStore for DatabaseClient {
    fn get_parser_state(&mut self, chain_str: &str) -> Result<ParserState, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        parser_state
            .filter(chain.eq(chain_str))
            .select(ParserState::as_select())
            .first(&mut self.connection)
    }

    fn add_parser_state(&mut self, chain_str: &str) -> Result<usize, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        diesel::insert_into(parser_state)
            .values(chain.eq(chain_str))
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    fn get_parser_states(&mut self) -> Result<Vec<ParserState>, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        parser_state.select(ParserState::as_select()).load(&mut self.connection)
    }

    fn set_parser_state_latest_block(&mut self, chain_str: &str, block: i32) -> Result<usize, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        diesel::update(parser_state.find(chain_str))
            .set(latest_block.eq(block))
            .execute(&mut self.connection)
    }

    fn set_parser_state_current_block(&mut self, chain_str: &str, block: i32) -> Result<usize, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        diesel::update(parser_state.find(chain_str))
            .set(current_block.eq(block))
            .execute(&mut self.connection)
    }
}

impl ParserStateRepository for DatabaseClient {}
