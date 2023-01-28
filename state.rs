use anchor_lang::prelude::*;
#[account]
pub struct Master {
    pub last_bet_id: u64,
}
#[account]
pub struct Bet {
    pub id: u64,
    pub amount: u64,
    pub prediction_a: BetPrediction,
    pub prediction_b: Option<BetPrediction>,
    pub state: BetState,
    pub pyth_price_key: Pubkey,
    pub expiry_ts: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BetPrediction {
    //the address that bets
    pub player: Pubkey,
    //price prediction in USD
    pub price: f64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum BetState {
    Created,
    Started,
    PlayerAWon,
    PlayerBWon,
    Draw,
}
