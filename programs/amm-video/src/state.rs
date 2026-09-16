use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub seed: u64,
    pub authority: Option<Pubkey>,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub fee: u16,              // swap fee in basis points (e.g. 30 = 0.30%)
    pub locked: bool,
    pub config_bump: u8,
    pub lp_bump: u8,
}
