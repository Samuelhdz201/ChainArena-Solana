use anchor_lang::prelude::*;

declare_id!("54N5nsEJgPWf4ghPn6teZrseTxLo7wr1vLBMLGhruVgx"); 

#[program]
pub mod lol_tournament {
    use super::*;

    pub fn create_tournament(
        ctx: Context<CreateTournament>,
        tournament_id: String,
        entry_fee: u64,
        max_players: u8,
    ) -> Result<()> {
        require!(tournament_id.len() <= 50, TournamentError::NameTooLong);
        require!(max_players >= 2 && max_players <= 16, TournamentError::InvalidPlayerCount);

        let tournament = &mut ctx.accounts.tournament;
        tournament.organizer = ctx.accounts.organizer.key();
        tournament.oracle = ctx.accounts.organizer.key();
        tournament.tournament_id = tournament_id.clone();
        tournament.entry_fee = entry_fee;
        tournament.max_players = max_players;
        tournament.registered_players = 0;
        tournament.prize_pool = 0;
        tournament.state = TournamentState::Open;
        tournament.winner = Pubkey::default();
        tournament.prize_claimed = false;
        tournament.players = Vec::new();
        tournament.bump = ctx.bumps.tournament;

        emit!(TournamentCreated {
            tournament_id,
            entry_fee,
            max_players,
        });

        msg!("Tournament created!");
        Ok(())
    }

    pub fn register_player(ctx: Context<RegisterPlayer>) -> Result<()> {
        let player_key = ctx.accounts.player.key();

        // ✅ Guardar valores antes del borrow mutable
        let entry_fee = ctx.accounts.tournament.entry_fee;
        let registered = ctx.accounts.tournament.registered_players;
        let max = ctx.accounts.tournament.max_players;
        let state = ctx.accounts.tournament.state.clone();
        let already_in = ctx.accounts.tournament.players.contains(&player_key);

        require!(state == TournamentState::Open, TournamentError::NotOpen);
        require!(registered < max, TournamentError::TournamentFull);
        require!(!already_in, TournamentError::AlreadyRegistered);

        // Transferir entry fee al escrow
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.player.to_account_info(),
                to: ctx.accounts.tournament.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, entry_fee)?;

        let tournament = &mut ctx.accounts.tournament;
        tournament.players.push(player_key);
        tournament.prize_pool += entry_fee;
        tournament.registered_players += 1;

        let current_players = tournament.registered_players;
        let prize_pool = tournament.prize_pool;

        emit!(PlayerRegistered {
            player: player_key,
            prize_pool,
        });

        if current_players == max {
            tournament.state = TournamentState::InProgress;
            emit!(TournamentFull {
                total_players: current_players,
                prize_pool,
            });
            msg!("Tournament full — started automatically!");
        }

        msg!("Player registered! Total: {}", current_players);
        Ok(())
    }

    pub fn start_tournament(ctx: Context<ManageTournament>) -> Result<()> {
        let tournament = &mut ctx.accounts.tournament;

        require!(tournament.state == TournamentState::Open, TournamentError::NotOpen);
        require!(ctx.accounts.organizer.key() == tournament.organizer, TournamentError::Unauthorized);
        require!(tournament.registered_players >= 2, TournamentError::NotEnoughPlayers);

        tournament.state = TournamentState::InProgress;

        emit!(TournamentStarted {
            registered_players: tournament.registered_players,
            prize_pool: tournament.prize_pool,
        });

        msg!("Tournament started manually!");
        Ok(())
    }

    pub fn declare_winner(ctx: Context<DeclareWinner>, winner: Pubkey) -> Result<()> {
        let tournament = &mut ctx.accounts.tournament;

        require!(tournament.state == TournamentState::InProgress, TournamentError::NotInProgress);
        require!(ctx.accounts.oracle.key() == tournament.oracle, TournamentError::Unauthorized);
        require!(tournament.players.contains(&winner), TournamentError::InvalidWinner);
        require!(tournament.winner == Pubkey::default(), TournamentError::AlreadyDeclared);

        tournament.winner = winner;
        tournament.state = TournamentState::Finished;

        emit!(WinnerDeclared {
            winner,
            prize_pool: tournament.prize_pool,
        });

        msg!("Winner declared: {}", winner);
        Ok(())
    }

    pub fn claim_prize(ctx: Context<ClaimPrize>) -> Result<()> {
        let tournament = &mut ctx.accounts.tournament;

        require!(tournament.state == TournamentState::Finished, TournamentError::NotFinished);
        require!(ctx.accounts.winner.key() == tournament.winner, TournamentError::NotWinner);
        require!(!tournament.prize_claimed, TournamentError::AlreadyClaimed);

        let prize = tournament.prize_pool;
        tournament.prize_claimed = true;
        tournament.prize_pool = 0;

        **ctx.accounts.tournament.to_account_info().try_borrow_mut_lamports()? -= prize;
        **ctx.accounts.winner.to_account_info().try_borrow_mut_lamports()? += prize;

        emit!(PrizeClaimed {
            winner: ctx.accounts.winner.key(),
            amount: prize,
        });

        msg!("Prize claimed: {} lamports", prize);
        Ok(())
    }
}

// ── Accounts ──────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(tournament_id: String)]
pub struct CreateTournament<'info> {
    #[account(
        init,
        payer = organizer,
        space = Tournament::SPACE,
        seeds = [b"tournament", organizer.key().as_ref(), tournament_id.as_bytes()],
        bump
    )]
    pub tournament: Account<'info, Tournament>,
    #[account(mut)]
    pub organizer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterPlayer<'info> {
    #[account(
        mut,
        seeds = [b"tournament", tournament.organizer.as_ref(), tournament.tournament_id.as_bytes()],
        bump = tournament.bump
    )]
    pub tournament: Account<'info, Tournament>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ManageTournament<'info> {
    #[account(
        mut,
        seeds = [b"tournament", tournament.organizer.as_ref(), tournament.tournament_id.as_bytes()],
        bump = tournament.bump
    )]
    pub tournament: Account<'info, Tournament>,
    pub organizer: Signer<'info>,
}

#[derive(Accounts)]
pub struct DeclareWinner<'info> {
    #[account(
        mut,
        seeds = [b"tournament", tournament.organizer.as_ref(), tournament.tournament_id.as_bytes()],
        bump = tournament.bump
    )]
    pub tournament: Account<'info, Tournament>,
    pub oracle: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimPrize<'info> {
    #[account(
        mut,
        seeds = [b"tournament", tournament.organizer.as_ref(), tournament.tournament_id.as_bytes()],
        bump = tournament.bump
    )]
    pub tournament: Account<'info, Tournament>,
    #[account(mut)]
    pub winner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// ── State ─────────────────────────────────────────────────

#[account]
pub struct Tournament {
    pub organizer: Pubkey,
    pub oracle: Pubkey,
    pub tournament_id: String,
    pub entry_fee: u64,
    pub max_players: u8,
    pub registered_players: u8,
    pub prize_pool: u64,
    pub state: TournamentState,
    pub winner: Pubkey,
    pub prize_claimed: bool,
    pub players: Vec<Pubkey>,
    pub bump: u8,
}

impl Tournament {
    pub const SPACE: usize = 8
        + 32          // organizer
        + 32          // oracle
        + 4 + 50      // tournament_id
        + 8           // entry_fee
        + 1           // max_players
        + 1           // registered_players
        + 8           // prize_pool
        + 1           // state
        + 32          // winner
        + 1           // prize_claimed
        + 4 + (32 * 16) // players vec (max 16)
        + 1;          // bump
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum TournamentState {
    Open,
    InProgress,
    Finished,
}

// ── Events ────────────────────────────────────────────────

#[event]
pub struct TournamentCreated {
    pub tournament_id: String,
    pub entry_fee: u64,
    pub max_players: u8,
}

#[event]
pub struct PlayerRegistered {
    pub player: Pubkey,
    pub prize_pool: u64,
}

#[event]
pub struct TournamentFull {
    pub total_players: u8,
    pub prize_pool: u64,
}

#[event]
pub struct TournamentStarted {
    pub registered_players: u8,
    pub prize_pool: u64,
}

#[event]
pub struct WinnerDeclared {
    pub winner: Pubkey,
    pub prize_pool: u64,
}

#[event]
pub struct PrizeClaimed {
    pub winner: Pubkey,
    pub amount: u64,
}

// ── Errors ────────────────────────────────────────────────

#[error_code]
pub enum TournamentError {
    #[msg("El torneo no está abierto")]
    NotOpen,
    #[msg("El torneo está lleno")]
    TournamentFull,
    #[msg("No autorizado")]
    Unauthorized,
    #[msg("El torneo no está en progreso")]
    NotInProgress,
    #[msg("El torneo no ha terminado")]
    NotFinished,
    #[msg("No eres el ganador")]
    NotWinner,
    #[msg("Premio ya reclamado")]
    AlreadyClaimed,
    #[msg("Jugador ya registrado")]
    AlreadyRegistered,
    #[msg("Ganador inválido — no es jugador del torneo")]
    InvalidWinner,
    #[msg("Nombre de torneo demasiado largo (max 50 chars)")]
    NameTooLong,
    #[msg("Número de jugadores inválido (2-16)")]
    InvalidPlayerCount,
    #[msg("No hay suficientes jugadores para iniciar")]
    NotEnoughPlayers,
    #[msg("Ganador ya declarado")]
    AlreadyDeclared,
}