use crate::{DatabaseClient, models::*, sql_types::ChainRow};

use diesel::prelude::*;
use primitives::Chain;

pub(crate) trait ParserStateStore {
    fn get_parser_state(&mut self, chain: Chain) -> Result<ParserStateRow, diesel::result::Error>;
    fn add_parser_state(&mut self, chain: Chain, block_time_ms: i32) -> Result<usize, diesel::result::Error>;
    fn get_parser_states(&mut self) -> Result<Vec<ParserStateRow>, diesel::result::Error>;
    fn set_parser_state_latest_block(&mut self, chain: Chain, block: i64) -> Result<usize, diesel::result::Error>;
    fn set_parser_state_current_block(&mut self, chain: Chain, block: i64) -> Result<usize, diesel::result::Error>;
}

impl ParserStateStore for DatabaseClient {
    fn get_parser_state(&mut self, chain_value: Chain) -> Result<ParserStateRow, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        parser_state
            .filter(chain.eq(ChainRow::from(chain_value)))
            .select(ParserStateRow::as_select())
            .first(&mut self.connection)
    }

    fn add_parser_state(&mut self, chain_value: Chain, block_time_ms: i32) -> Result<usize, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        diesel::insert_into(parser_state)
            .values((chain.eq(ChainRow::from(chain_value)), block_time.eq(block_time_ms)))
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    fn get_parser_states(&mut self) -> Result<Vec<ParserStateRow>, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        parser_state.select(ParserStateRow::as_select()).load(&mut self.connection)
    }

    fn set_parser_state_latest_block(&mut self, chain_value: Chain, block: i64) -> Result<usize, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        diesel::update(parser_state.find(ChainRow::from(chain_value)))
            .set(latest_block.eq(block))
            .execute(&mut self.connection)
    }

    fn set_parser_state_current_block(&mut self, chain_value: Chain, block: i64) -> Result<usize, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        diesel::update(parser_state.find(ChainRow::from(chain_value)))
            .set(current_block.eq(block))
            .execute(&mut self.connection)
    }
}

impl DatabaseClient {
    pub fn get_parser_state(&mut self, chain: Chain) -> Result<ParserStateRow, diesel::result::Error> {
        ParserStateStore::get_parser_state(self, chain)
    }

    pub fn add_parser_state(&mut self, chain: Chain, block_time_ms: i32) -> Result<usize, diesel::result::Error> {
        ParserStateStore::add_parser_state(self, chain, block_time_ms)
    }

    pub fn get_parser_states(&mut self) -> Result<Vec<ParserStateRow>, diesel::result::Error> {
        ParserStateStore::get_parser_states(self)
    }

    pub fn set_parser_state_latest_block(&mut self, chain: Chain, block: i64) -> Result<usize, diesel::result::Error> {
        ParserStateStore::set_parser_state_latest_block(self, chain, block)
    }

    pub fn set_parser_state_current_block(&mut self, chain: Chain, block: i64) -> Result<usize, diesel::result::Error> {
        ParserStateStore::set_parser_state_current_block(self, chain, block)
    }
}
