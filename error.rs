use anchor_lang::prelude::*;
#[error_code]
pub enum BetError {
    #[msg("Cannot enter")]
    CannotEnter,
    #[msg("Cannot claim")]
    CannotClaim,
    #[msg("Cannot close")]
    CannotClose,
    #[msg("Given Key for the Pyth account does not match")]
    InvalidPythKey,
    #[msg("Invalid Pyth account")]
    InvalidPythAccount,
    #[msg("Price is too big to parse to u32")]
    PriceTooBig,
}
