use crate::DatabaseError;

use crate::DatabaseClient;
use crate::database::parser_state::ParserStateStore;
use primitives::Chain;

pub trait ParserStateRepository {
    fn get_parser_state(&mut self, chain: Chain) -> Result<crate::models::ParserStateRow, DatabaseError>;
    fn add_parser_state(&mut self, chain: Chain, block_time_ms: i32) -> Result<usize, DatabaseError>;
    fn get_parser_states(&mut self) -> Result<Vec<crate::models::ParserStateRow>, DatabaseError>;
    fn set_parser_state_latest_block(&mut self, chain: Chain, block: i64) -> Result<usize, DatabaseError>;
    fn set_parser_state_current_block(&mut self, chain: Chain, block: i64) -> Result<usize, DatabaseError>;
}

impl ParserStateRepository for DatabaseClient {
    fn get_parser_state(&mut self, chain: Chain) -> Result<crate::models::ParserStateRow, DatabaseError> {
        Ok(ParserStateStore::get_parser_state(self, chain)?)
    }

    fn add_parser_state(&mut self, chain: Chain, block_time_ms: i32) -> Result<usize, DatabaseError> {
        Ok(ParserStateStore::add_parser_state(self, chain, block_time_ms)?)
    }

    fn get_parser_states(&mut self) -> Result<Vec<crate::models::ParserStateRow>, DatabaseError> {
        Ok(ParserStateStore::get_parser_states(self)?)
    }

    fn set_parser_state_latest_block(&mut self, chain: Chain, block: i64) -> Result<usize, DatabaseError> {
        Ok(ParserStateStore::set_parser_state_latest_block(self, chain, block)?)
    }

    fn set_parser_state_current_block(&mut self, chain: Chain, block: i64) -> Result<usize, DatabaseError> {
        Ok(ParserStateStore::set_parser_state_current_block(self, chain, block)?)
    }
}
