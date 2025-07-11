use std::error::Error;

use crate::database::parser_state::ParserStateStore;
use crate::DatabaseClient;

pub trait ParserStateRepository {
    fn get_parser_state(&mut self, chain: &str) -> Result<crate::models::ParserState, Box<dyn Error + Send + Sync>>;
    fn add_parser_state(&mut self, chain: &str) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn get_parser_states(&mut self) -> Result<Vec<crate::models::ParserState>, Box<dyn Error + Send + Sync>>;
    fn set_parser_state_latest_block(&mut self, chain: &str, block: i32) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn set_parser_state_current_block(&mut self, chain: &str, block: i32) -> Result<usize, Box<dyn Error + Send + Sync>>;
}

impl ParserStateRepository for DatabaseClient {
    fn get_parser_state(&mut self, chain: &str) -> Result<crate::models::ParserState, Box<dyn Error + Send + Sync>> {
        Ok(ParserStateStore::get_parser_state(self, chain)?)
    }

    fn add_parser_state(&mut self, chain: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(ParserStateStore::add_parser_state(self, chain)?)
    }

    fn get_parser_states(&mut self) -> Result<Vec<crate::models::ParserState>, Box<dyn Error + Send + Sync>> {
        Ok(ParserStateStore::get_parser_states(self)?)
    }

    fn set_parser_state_latest_block(&mut self, chain: &str, block: i32) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(ParserStateStore::set_parser_state_latest_block(self, chain, block)?)
    }

    fn set_parser_state_current_block(&mut self, chain: &str, block: i32) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(ParserStateStore::set_parser_state_current_block(self, chain, block)?)
    }
}
