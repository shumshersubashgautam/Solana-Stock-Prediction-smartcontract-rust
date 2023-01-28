use anchor_lang::solana_program::clock::UnixTimestamp;

pub const MASTER_SEED: &[u8] = b"master";

pub const BET_SEED: &[u8] = b"bet";

//the closer we are to the expiry, the more chances a player to win
//adjust this  to make it fair
pub const MINIMUM_REMAINING_TIME_UNTIL_EXPIRY: UnixTimestamp = 120;

pub const MAXIMUM_CLAIMABLE_PERIOD:UnixTimestamp = 300 ;
