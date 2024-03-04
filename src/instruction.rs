use crate::error::GameError::InvalidInstruction;
use crate::state::{Init,Join,Move,ClaimVictory,Tournament,CounterFinder,FinderFinder,InitTournamentCounter};
use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

#[derive(Debug, PartialEq)]
pub enum GameInstruction {

  InitGame{ init : Init},
  JoinGame{ join : Join},
  Play {mymove : Move},
  ClaimvictoryScore{claim : ClaimVictory},
  ClaimvictoryTime,
  Abort,
  InitializeTournament{t : Tournament},
  JoinTournament,
  MatchInitialize{ init : Init},
  MatchAccept{ join : Join},
  PlayTournament {mymove : Move},
  TournamentClaimvictoryScore{claim : ClaimVictory},
  Eliminate,
  EliminatePlay,
  InitCounter{ t_counter : InitTournamentCounter},
  UpdateCounterFinder{ c_finder : CounterFinder},
  UpdateFinderFinder{ f_finder : FinderFinder},
  DeleteAuth,
  Delete,
}

impl GameInstruction {
  pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
    let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
    
    Ok(match tag {
      0 => Self::InitGame{
        init: Init::try_from_slice(&rest)?,
      },
      1 => Self::JoinGame{
        join: Join::try_from_slice(&rest)?,
      },
      2 => Self::Play{
        mymove: Move::try_from_slice(&rest)?,
      },
      3 => Self::ClaimvictoryScore{
        claim: ClaimVictory::try_from_slice(&rest)?,
      },
      4 => Self::ClaimvictoryTime,
      5 => Self::Abort,
      6 => Self::InitializeTournament{
        t: Tournament::try_from_slice(&rest)?,
      },
      7 => Self::JoinTournament,
      8 => Self::MatchInitialize{
        init: Init::try_from_slice(&rest)?,
      },
      9 => Self::MatchAccept{
        join: Join::try_from_slice(&rest)?,
      },
      10 => Self::PlayTournament{
        mymove: Move::try_from_slice(&rest)?,
      },
      11 => Self::TournamentClaimvictoryScore{
        claim: ClaimVictory::try_from_slice(&rest)?,
      },
      12 => Self::Eliminate,
      13 => Self::EliminatePlay,
      14 => Self::InitCounter{
        t_counter: InitTournamentCounter::try_from_slice(&rest)?,
      },
      15 => Self::UpdateCounterFinder{
        c_finder: CounterFinder::try_from_slice(&rest)?,
      },
      16 => Self::UpdateFinderFinder{
        f_finder: FinderFinder::try_from_slice(&rest)?,
      },
      17 => Self::DeleteAuth,
      18 => Self::Delete,

      _ => return Err(InvalidInstruction.into()),
    })
  }
}
