use std::error::Error;

use primitives::Chain;
use storage::{Database, models::ParserStateRow};

pub struct ParserStateService {
    chain: Chain,
    database: Database,
}

impl ParserStateService {
    pub fn new(chain: Chain, database: Database) -> Self {
        Self { chain, database }
    }

    pub fn get_state(&self) -> Result<ParserStateRow, Box<dyn Error + Send + Sync>> {
        Ok(self.database.parser_state()?.get_parser_state(self.chain.as_ref())?)
    }

    pub fn set_current_block(&self, block: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.database.parser_state()?.set_parser_state_current_block(self.chain.as_ref(), block)?;
        Ok(())
    }

    pub fn set_latest_block(&self, block: i64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.database.parser_state()?.set_parser_state_latest_block(self.chain.as_ref(), block)?;
        Ok(())
    }
}
