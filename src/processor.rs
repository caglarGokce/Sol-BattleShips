use crate::instruction::GameInstruction;
use crate::state::{ClaimVictory, CounterFinder, FinderFinder, GameState, Init, InitTournamentCounter, Join, Move, TGameState, Tournament, TournamentAccount, TournamentCounter
};

use borsh::{BorshDeserialize, BorshSerialize};
use std::str::FromStr;
use solana_program::{
  account_info::{next_account_info, AccountInfo},
  entrypoint::ProgramResult,
  pubkey::Pubkey,
  sysvar::{clock::Clock, Sysvar,},
  keccak,
  rent::Rent,
  system_instruction,
  program::invoke_signed,
  system_program::ID,
};



pub struct Processor;
impl Processor {
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    let instruction = GameInstruction::unpack(instruction_data)?;

    match instruction {

      GameInstruction::InitGame {init} => {
        Self::init_game(accounts,init, program_id)
      }
      GameInstruction::Play { mymove } => {
        Self::play(program_id, accounts, mymove)
      }
      GameInstruction::JoinGame { join } => {
        Self::join_the_game(accounts,program_id,join)
      }
      GameInstruction::ClaimvictoryScore {claim} => {
        Self::claim_victory_by_score(accounts,program_id,claim)
      }
      GameInstruction::ClaimvictoryTime => {
        Self::claim_victory_by_time(accounts,program_id)
      }
      GameInstruction::Abort => {
        Self::abort_game(accounts,program_id)
      }
      GameInstruction::InitializeTournament { t } => {
        Self::init_tournament(accounts,t)
      }
      GameInstruction::JoinTournament => {
        Self::join_tournament(accounts,program_id)
      }
      GameInstruction::MatchInitialize {init} => {
        Self::tournament_match_initialize(accounts,program_id,init)
      }
      GameInstruction::MatchAccept { join } => {
        Self::tournament_match_accept(accounts,program_id,join)
      }
      GameInstruction::PlayTournament { mymove } => {
        Self::tournament_play(program_id, accounts, mymove)
      }
      GameInstruction::TournamentClaimvictoryScore {claim} => {
        Self::tournament_claim_victory_by_score(program_id,accounts,claim)
      }
      GameInstruction::Eliminate => {
        Self::eliminate_player_who_hasnt_initialized_his_game(accounts,program_id)
      }
      GameInstruction::EliminatePlay => {
        Self::eliminate_player_who_hasnt_made_his_move(accounts,program_id)
      }
      GameInstruction::InitCounter {t_counter} => {
        Self::initialize_counter(accounts,program_id,t_counter)
      }
      GameInstruction::UpdateCounterFinder {c_finder} => {
        Self::update_counter_finder(accounts,c_finder)
      }
      GameInstruction::UpdateFinderFinder {f_finder} => {
        Self::update_finder_finder(accounts,f_finder)
      }
      GameInstruction::DeleteAuth => {
        Self::delete_account_authority(accounts)
      }
      GameInstruction::Delete => {
        Self::delete_account(accounts)
      }

    }
  }

  fn init_game(
    accounts: &[AccountInfo],
    init: Init,
    program_id:&Pubkey) -> ProgramResult {


    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let host: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let game_state: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    let game_state_check: Pubkey = Pubkey::create_with_seed(initializer.key, &init.gameseed, program_id).unwrap();

    if game_state.key != &game_state_check{panic!()}
    if game_state.owner != program_id{panic!()}

    
    let mut state: GameState = GameState::try_from_slice(&game_state.data.borrow())?;

    if state.initialized != 0 {panic!()}
    if init.gameseed.len() != 5 {panic!()}

    state.host=host.key.to_bytes();
    state.waiting=1;
    state.lamports=init.lamports;
    state.initializer= initializer.key.to_bytes();
    state.initializergamehash= init.initializer_game_hash;
    state.guest= [0;32];
    state.guestgamehash= [0;32];
    state.initialized = 1;
    state.gameseed = init.gameseed;


    let r: u64 = Rent::default().minimum_balance(322);
    if **game_state.lamports.borrow() < init.lamports + r {panic!()}

    state.serialize(&mut &mut game_state.data.borrow_mut()[..])?;
   
    Ok(())
  }
  fn join_the_game(
    accounts: &[AccountInfo], 
    program_id: &Pubkey,
    join: Join ) -> ProgramResult {

    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    let guest: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let game_state: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let temp_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let host: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    if game_state.owner != program_id{panic!()}

    let mut state: GameState = GameState::try_from_slice(&game_state.data.borrow())?;

    let host_address: Pubkey = Pubkey::new_from_array(state.host);

    if &host_address != host.key {panic!()}

    if state.initialized != 1 {panic!()}

    let clock: Clock= Clock::get()?;
    let current_time: u64 = clock.unix_timestamp as u64;

    if join.guest_game_hash.len() != 44 {panic!()}//////////////////////////////////////////////////////

    state.guest = guest.key.to_bytes();
    state.guestgamehash = join.guest_game_hash;
    state.waiting = 2;
    state.lastplaytime = current_time;
    state.whoseturn = 1;
    state.initialized = 2;

    **temp_account.lamports.borrow_mut()-= state.lamports;
    **game_state.lamports.borrow_mut()+= state.lamports;

    state.serialize(&mut &mut game_state.data.borrow_mut()[..])?;

    Ok(())
  }
  fn play(        
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    m: Move,) -> ProgramResult {


    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let guest: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let host: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let game_state: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    let mut state: GameState = GameState::try_from_slice(&game_state.data.borrow())?;


    let guest_check: Pubkey = Pubkey::new_from_array(state.guest);
    let initializer_check: Pubkey = Pubkey::new_from_array(state.initializer);
    let host_check: Pubkey = Pubkey::new_from_array(state.host);
    let game_state_check: Pubkey = Pubkey::create_with_seed(initializer.key, &state.gameseed, program_id).unwrap();

    if initializer.key != &initializer_check{panic!()}
    if guest.key != &guest_check{panic!()}
    if host.key != &host_check{panic!()}
    if game_state.key != &game_state_check{panic!()}
    if state.initialized != 2 {panic!()}
    if m.mymove > 110 {panic!()}
    if m.mymove < 11 {panic!()}

    let clock: Clock= Clock::get()?;
    let current_time: u64 = clock.unix_timestamp as u64;

    let mut turn:u8 = 0;
    let hit:u8 = m.opponents_last_move_is_hit;
    let index: usize = m.mymove as usize;
    let index2: usize = state.lastmove as usize;


    if state.whoseturn == 1{ // 1 initializer
      if !initializer.is_signer{panic!()};
      turn = 2;
      state.ishots += 1;
      state.ghits += hit;
      if state.guest_board_state[index] != 0{panic!()};
      state.guest_board_state[index] += 1;
      state.initializer_board_state[index2] += hit;
    }//hit 2
     //miss 1
     //unchecked 0

    if state.whoseturn == 2{ // 2 guest
      if !guest.is_signer{panic!()};
      turn = 1;
      state.gshots += 1;
      state.ihits += hit;
      if state.initializer_board_state[index] != 0{panic!()};
      state.initializer_board_state[index] += 1;
      state.guest_board_state[index2] += hit;
    }

    state.lastplaytime = current_time;
    state.whoseturn  = turn;
    state.lastmove = m.mymove;

    state.serialize(&mut &mut game_state.data.borrow_mut()[..])?;


    Ok(())
  }
  fn claim_victory_by_score(        
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    claim: ClaimVictory,) -> ProgramResult {

      let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

      let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
      let guest: &AccountInfo<'_> = next_account_info(accounts_iter)?;
      let host: &AccountInfo<'_> = next_account_info(accounts_iter)?;
      let game_state: &AccountInfo<'_> = next_account_info(accounts_iter)?;
  
      let mut state: GameState = GameState::try_from_slice(&game_state.data.borrow())?;
  

  
      let guest_check: Pubkey = Pubkey::new_from_array(state.guest);
      let initializer_check: Pubkey = Pubkey::new_from_array(state.initializer);
      let host_check: Pubkey = Pubkey::new_from_array(state.host);
      let game_state_check: Pubkey = Pubkey::create_with_seed(initializer.key, &state.gameseed, program_id).unwrap();
  
      if initializer.key != &initializer_check{panic!()}
      if guest.key != &guest_check{panic!()}
      if host.key != &host_check{panic!()}
      if game_state.key != &game_state_check{panic!()}
      if state.initialized != 2 {panic!()}

      let mut game_arr:[u8;17]=[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];

    if claim.s1ax == 1{
      if claim.s1cox > 6 {panic!()}
      if claim.s1cox < 1 {panic!()}
      if claim.s1coy > 10 {panic!()}
      if claim.s1coy < 1 {panic!()}
      game_arr[0] = claim.s1cox+(claim.s1coy*10);
      game_arr[1] = (claim.s1cox + 1)+(claim.s1coy*10);
      game_arr[2] = (claim.s1cox + 2)+(claim.s1coy*10);
      game_arr[3] = (claim.s1cox + 3)+(claim.s1coy*10);
      game_arr[4] = (claim.s1cox + 4)+(claim.s1coy*10);
    }
    if claim.s1ax == 2{
      if claim.s1coy > 6 {panic!()}
      if claim.s1coy < 1 {panic!()}
      if claim.s1cox > 10 {panic!()}
      if claim.s1cox < 1 {panic!()}
      game_arr[0] = (claim.s1coy*10)+(claim.s1cox);
      game_arr[1] = ((claim.s1coy + 1)*10)+(claim.s1cox);
      game_arr[2] = ((claim.s1coy + 2)*10)+(claim.s1cox);
      game_arr[3] = ((claim.s1coy + 3)*10)+(claim.s1cox);
      game_arr[4] = ((claim.s1coy + 4)*10)+(claim.s1cox);
    }
    if claim.s2ax == 1{
      if claim.s2cox > 6 {panic!()}
      if claim.s2cox < 1 {panic!()}
      if claim.s2coy > 10 {panic!()}
      if claim.s2coy < 1 {panic!()}
      game_arr[5] = claim.s2cox+(claim.s2coy*10);
      game_arr[6] = (claim.s2cox + 1)+(claim.s2coy*10);
      game_arr[7] = (claim.s2cox + 2)+(claim.s2coy*10);
      game_arr[8] = (claim.s2cox + 3)+(claim.s2coy*10);
    }
    if claim.s2ax == 2{
      if claim.s2coy > 6 {panic!()}
      if claim.s2coy < 1 {panic!()}
      if claim.s2cox > 10 {panic!()}
      if claim.s2cox < 1 {panic!()}
      game_arr[5] = (claim.s2coy*10)+(claim.s2cox);
      game_arr[6] = ((claim.s2coy + 1)*10)+(claim.s2cox);
      game_arr[7] = ((claim.s2coy + 2)*10)+(claim.s2cox);
      game_arr[8] = ((claim.s2coy + 3)*10)+(claim.s2cox);
    }
    if claim.s3ax == 1{
      if claim.s3cox > 6 {panic!()}
      if claim.s3cox < 1 {panic!()}
      if claim.s3coy > 10 {panic!()}
      if claim.s3coy < 1 {panic!()}
      game_arr[9] = claim.s3cox+(claim.s3coy*10);
      game_arr[10] = (claim.s3cox + 1)+(claim.s3coy*10);
      game_arr[11] = (claim.s3cox + 2)+(claim.s3coy*10);
    }
    if claim.s3ax == 2{
      if claim.s3coy > 6 {panic!()}
      if claim.s3coy < 1 {panic!()}
      if claim.s3cox > 10 {panic!()}
      if claim.s3cox < 1 {panic!()}
      game_arr[9] = (claim.s3coy*10)+(claim.s3cox);
      game_arr[10] = ((claim.s3coy + 1)*10)+(claim.s3cox);
      game_arr[11] = ((claim.s3coy + 2)*10)+(claim.s3cox);
    }
    if claim.s4ax == 1{
      if claim.s4cox > 6 {panic!()}
      if claim.s4cox < 1 {panic!()}
      if claim.s4coy > 10 {panic!()}
      if claim.s4coy < 1 {panic!()}
      game_arr[12] = claim.s4cox+(claim.s4coy*10);
      game_arr[13] = (claim.s4cox + 1)+(claim.s4coy*10);
      game_arr[14] = (claim.s4cox + 2)+(claim.s4coy*10);
    }
    if claim.s4ax == 2{
      if claim.s4coy > 6 {panic!()}
      if claim.s4coy < 1 {panic!()}
      if claim.s4cox > 10 {panic!()}
      if claim.s4cox < 1 {panic!()}
      game_arr[12] = (claim.s4coy*10)+(claim.s4cox);
      game_arr[13] = ((claim.s4coy + 1)*10)+(claim.s4cox);
      game_arr[14] = ((claim.s4coy + 2)*10)+(claim.s4cox);
    }
    if claim.s5ax == 1{
      if claim.s5cox > 6 {panic!()}
      if claim.s5cox < 1 {panic!()}
      if claim.s5coy > 10 {panic!()}
      if claim.s5coy < 1 {panic!()}
      game_arr[15] = claim.s5cox+(claim.s5coy*10);
      game_arr[16] = (claim.s5cox + 1)+(claim.s5coy*10);
    }
    if claim.s5ax == 2{
      if claim.s5coy > 6 {panic!()}
      if claim.s5coy < 1 {panic!()}
      if claim.s5cox > 10 {panic!()}
      if claim.s5cox < 1 {panic!()}
      game_arr[15] = (claim.s5coy*10)+(claim.s5cox);
      game_arr[16] = ((claim.s5coy + 1)*10)+(claim.s5cox);
    }

    let game_hash: keccak::Hash = keccak::hashv(&[&claim.seed.to_string().as_bytes(),&game_arr,&claim.seed.to_string().as_bytes()]);


    if claim.claims == 1 {
      if state.initializergamehash != game_hash.0{panic!()}
    }
    else {
      if state.guestgamehash != game_hash.0{panic!()}
    }

    if claim.claims == 1 {
      for x in 1..18 {
        let u = x as usize;
        let s = game_arr[u] as usize;
        if state.initializer_board_state[s] > 0 && state.initializer_board_state[s] != 2 {
          state.ihits = 0;
          state.ghits = 17;
        }
      }
    }else{
      for x in 1..18 {
        let u = x as usize;
        let s = game_arr[u] as usize;
        if state.guest_board_state[s] > 0 && state.guest_board_state[s] != 2 {
          state.ihits = 17;
          state.ghits = 0;
        }
      }
    }

      let mut draw:bool=false;
      let mut iwins:bool=false;
      let mut gwins:bool=false;
  
      if state.ishots != state.gshots{panic!()}
      if state.ihits == 17{iwins = true;}
      if state.ghits == 17{gwins = true;}
      if gwins && iwins{draw = true;iwins = false;gwins = false;}
      
      if iwins == true{
  
        let host_fee = (state.lamports/100)*10;
  
        **game_state.lamports.borrow_mut()-= host_fee;
        **host.lamports.borrow_mut()+= host_fee;
  
        let value = **game_state.lamports.borrow();
  
        **game_state.lamports.borrow_mut()-= value;
        **initializer.lamports.borrow_mut()+= value;
  
      }
      if gwins == true{
        let host_fee = (state.lamports/100)*10;
  
        **game_state.lamports.borrow_mut()-= host_fee;
        **host.lamports.borrow_mut()+= host_fee;
  
        let rew = (&state.lamports*2)-&host_fee;
  
        **game_state.lamports.borrow_mut()-= rew;
        **guest.lamports.borrow_mut()+= rew;
  
        let value = **game_state.lamports.borrow();
  
        **game_state.lamports.borrow_mut()-= value;
        **initializer.lamports.borrow_mut()+= value;
      }
      if draw == true{
        **game_state.lamports.borrow_mut()-= state.lamports;
        **guest.lamports.borrow_mut()+= state.lamports;
  
        let value = **game_state.lamports.borrow();
  
        **game_state.lamports.borrow_mut()-= value;
        **initializer.lamports.borrow_mut()+= value;
      }
  
      Ok(())

  }
  fn claim_victory_by_time(        
    accounts: &[AccountInfo],
    program_id: &Pubkey,) -> ProgramResult {


    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let guest: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let host: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let game_state: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    let state: GameState = GameState::try_from_slice(&game_state.data.borrow())?;


    let guest_check: Pubkey = Pubkey::new_from_array(state.guest);
    let initializer_check: Pubkey = Pubkey::new_from_array(state.initializer);
    let host_check: Pubkey = Pubkey::new_from_array(state.host);
    let game_state_check: Pubkey = Pubkey::create_with_seed(initializer.key, &state.gameseed, program_id).unwrap();

    if initializer.key != &initializer_check{panic!()}
    if guest.key != &guest_check{panic!()}
    if host.key != &host_check{panic!()}
    if game_state.key != &game_state_check{panic!()}
    if state.initialized != 2 {panic!()}

    let clock: Clock= Clock::get()?;
    let current_time: u64 = clock.unix_timestamp as u64;

    let time_passed: u64 = &current_time - &state.lastplaytime;

    if time_passed<120{panic!()}

    let mut gwins:bool=false;
    let mut iwins:bool=false;

    if state.whoseturn == 1{
      gwins = true;
    }
    if state.whoseturn == 2{
      iwins = true;
    }
    if iwins == true{

      let host_fee: u64 = (state.lamports/100)*10;

      **game_state.lamports.borrow_mut()-= host_fee;
      **host.lamports.borrow_mut()+= host_fee;

      let value: u64 = **game_state.lamports.borrow();

      **game_state.lamports.borrow_mut()-= value;
      **initializer.lamports.borrow_mut()+= value;

    }
    if gwins == true{
      let host_fee: u64 = (state.lamports/100)*10;

      **game_state.lamports.borrow_mut()-= host_fee;
      **host.lamports.borrow_mut()+= host_fee;

      let rew: u64 = (&state.lamports*2)-&host_fee;

      **game_state.lamports.borrow_mut()-= rew;
      **guest.lamports.borrow_mut()+= rew;

      let value: u64 = **game_state.lamports.borrow();

      **game_state.lamports.borrow_mut()-= value;
      **initializer.lamports.borrow_mut()+= value;
    }

    Ok(())
  }
  fn abort_game(        
    accounts: &[AccountInfo],
    program_id: &Pubkey,) -> ProgramResult {

    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let game_state: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    let state: GameState = GameState::try_from_slice(&game_state.data.borrow())?;


    let initializer_check: Pubkey = Pubkey::new_from_array(state.initializer);
    let game_state_check: Pubkey = Pubkey::create_with_seed(initializer.key, &state.gameseed, program_id).unwrap();

    if initializer.key != &initializer_check{panic!()}
    if game_state.key != &game_state_check{panic!()}
    if state.initialized != 1 {panic!()}

    let value: u64 = **game_state.lamports.borrow();

    **game_state.lamports.borrow_mut()-= value;
    **initializer.lamports.borrow_mut()+= value;

    Ok(())
  }
  fn init_tournament(        
    accounts: &[AccountInfo],
    t: Tournament) -> ProgramResult {


    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let tournament: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    let authority: Pubkey = Pubkey::from_str("4YbLBRXwseG1NuyJbteSD5u81Q2QjFqJBp6JmxwYBKYm").unwrap();

    if initializer.key != &authority {panic!()}
    if !initializer.is_signer{panic!()}

    t.serialize(&mut &mut tournament.data.borrow_mut()[..])?;
    

    Ok(())
  }
  fn join_tournament(        
    accounts: &[AccountInfo],
    program_id: &Pubkey,) -> ProgramResult {


    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    let participant: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let tournament_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let tournament: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let counter: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    let t: Tournament = Tournament::try_from_slice(&tournament.data.borrow())?;
    let mut t_counter: TournamentCounter = TournamentCounter::try_from_slice(&counter.data.borrow())?;

    let value: u64 = **tournament_account.lamports.borrow();

    if value < t.entrance_fee{panic!()}
    if tournament.is_writable{panic!()}
    if tournament.owner != program_id{panic!()}
    if t.is_init != 1 {panic!()}
    if tournament_account.owner != program_id{panic!()}
    if t_counter.player_participating >= t_counter.capacity{panic!()}


    t_counter.player_participating += 1;

    let mut str_no: String = String::new();
    let str_tournament_id: String = t.tournament_id;
    let str_counter_no: String = t_counter.counter_no.to_string();
    let str_player_no: String = t_counter.player_participating.to_string();
    let mut somestr: String = String::from("pppppp");
    let mut somestr2: String = String::from("cccc");
    let revstr2: String = str_counter_no.chars().rev().collect::<String>();
    let revstr: String = str_player_no.chars().rev().collect::<String>();
    let offset: &usize = &revstr.len();
    let offset2: &usize = &revstr2.len();
    somestr.replace_range(..offset, &revstr);
    somestr2.replace_range(..offset2, &revstr2);
    str_no += &somestr.chars().rev().collect::<String>();
    str_no += &somestr2.chars().rev().collect::<String>();
    str_no += &str_tournament_id;    


    let mut opp: u32 = t_counter.player_participating + 1;

    if t_counter.player_participating%2 == 0{
      opp = t_counter.player_participating - 1;
    }

    let tournament_account_check: Pubkey = Pubkey::create_with_seed(participant.key, &str_no, program_id).unwrap();

    if tournament_account.key != &tournament_account_check{panic!()}

    let t_account: TournamentAccount = TournamentAccount{
      player_find:str_no,
      tournamentid:str_tournament_id,
      player:participant.key.to_bytes(),
      opponent:opp,
      level:0,
      playerno_int:t_counter.player_participating,
      opponent_played_on:t.starts_at,
      is_playing:0,
      waiting_opponent_to_join:0
    };

    let full: String = String::from("F");

    if t_counter.player_participating == t_counter.capacity{
      t_counter.empty = full;
    }

    t_account.serialize(&mut &mut tournament_account.data.borrow_mut()[..])?;
    t_counter.serialize(&mut &mut counter.data.borrow_mut()[..])?;

    Ok(())
  }
  fn tournament_match_initialize(        
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    init: Init) -> ProgramResult {


    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let initializer_tour_acc: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let game_state: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let opponent_tour_acc: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let opponent: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let tournament: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    let t: Tournament = Tournament::try_from_slice(&tournament.data.borrow())?;

    let o_t_account: TournamentAccount = TournamentAccount::try_from_slice(&opponent_tour_acc.data.borrow())?;
    let opponent_check: Pubkey = Pubkey::new_from_array(o_t_account.player);

    let mut t_account: TournamentAccount = TournamentAccount::try_from_slice(&initializer_tour_acc.data.borrow())?;
    let initializer_account_check: Pubkey = Pubkey::new_from_array(t_account.player);

    let mut substract:bool = false;
    let initializer_no: u32 = o_t_account.playerno_int;
    let pwr: u32 = o_t_account.level as u32;
    let mut pwrplus: u32 = o_t_account.level as u32;
    pwrplus += 1;
    let pwoftwo:u32 = 2;
    let dividableby: &u32 = &pwoftwo.pow(pwrplus);
    if initializer_no%dividableby == 0 {
      substract = true;
    }
    let mut opponent_no: u32 = 0;
    if substract == false {
      opponent_no = &initializer_no + &pwoftwo.pow(pwr);
    }
    if substract == true {
      opponent_no = &initializer_no - &pwoftwo.pow(pwr);
    }
    let mut game_seed: String = String::new();
    let opponent_no_str: &String = &opponent_no.to_string();
    let initializer_no_str: &String = &initializer_no.to_string();
    let seed: String = String::from("v");
    if opponent_no > initializer_no{
      game_seed += initializer_no_str;
      game_seed += &seed;
      game_seed += opponent_no_str;
    }
    if opponent_no < initializer_no{
      game_seed += opponent_no_str;
      game_seed += &seed;
      game_seed += initializer_no_str;
    }

    let mut game_id: String = String::new();

    game_id += &t.tournament_id;
    game_id += &game_seed;


    let game_state_check: Pubkey = Pubkey::create_program_address(&[t.tournament_id.to_string().as_ref(),game_seed.as_ref(), &[init.bump]], program_id).unwrap();

    invoke_signed(
      &system_instruction::create_account( 
          &initializer.key, 
          &game_state.key,
          t.rent,
          140, //312???m
          &program_id
      ),
      &[
        initializer.clone(), 
        game_state.clone(),
      ],
      &[&[t.tournament_id.to_string().as_ref(),game_seed.as_ref(), &[init.bump]]],
    )?;

    if !initializer.is_signer{panic!()}
    if t.is_init != 1{panic!()}
    if t.tournament_id != t_account.tournamentid{panic!()}
    if t.tournament_id != o_t_account.tournamentid{panic!()}
    if o_t_account.level == t_account.level && o_t_account.waiting_opponent_to_join != 0 {panic!()}
    if opponent.key != &opponent_check{panic!()}
    if initializer.key != &initializer_account_check{panic!()}
    if game_state.key != &game_state_check{panic!()}
    if tournament.owner != program_id{panic!()}
    if game_state.owner != program_id{panic!()}
    if opponent_tour_acc.owner != program_id{panic!()}
    if initializer_tour_acc.owner != program_id{panic!()}
    if tournament.is_writable {panic!()}


    let mut state: TGameState = TGameState::try_from_slice(&game_state.data.borrow())?;


    if state.initialized != 0 {panic!()}
    if init.initializer_game_hash.len() != 44 {panic!()}///////////////////////////////////////////

    state.game_id=game_id;
    state.lamports=0;
    state.initializer= initializer_tour_acc.key.to_bytes();
    state.initializergamehash= init.initializer_game_hash;
    state.guest= [0;32];
    state.guestgamehash= [0;32];
    state.initialized = 3;


    t_account.opponent = opponent_no;
    t_account.waiting_opponent_to_join = 1;

    state.serialize(&mut &mut game_state.data.borrow_mut()[..])?;
    t_account.serialize(&mut &mut initializer_tour_acc.data.borrow_mut()[..])?;

    Ok(())
  }
  fn tournament_match_accept(
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    join: Join) -> ProgramResult {


    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let initializer_tour_acc: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let game_state: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let opponent_tour_acc: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let opponent: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let tournament: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    let t: Tournament = Tournament::try_from_slice(&tournament.data.borrow())?;

    let mut o_t_account: TournamentAccount = TournamentAccount::try_from_slice(&opponent_tour_acc.data.borrow())?;
    let opponent_check: Pubkey = Pubkey::new_from_array(o_t_account.player);

    let mut t_account: TournamentAccount = TournamentAccount::try_from_slice(&initializer_tour_acc.data.borrow())?;
    let initializer_account_check: Pubkey = Pubkey::new_from_array(t_account.player);

        if opponent.key != &opponent_check{panic!()}
    if initializer.key != &initializer_account_check{panic!()}

    let mut state: TGameState = TGameState::try_from_slice(&game_state.data.borrow())?;


    if !opponent.is_signer{panic!()}
    if t.is_init != 1{panic!()}
    if t.tournament_id != t_account.tournamentid{panic!()}
    if t.tournament_id != o_t_account.tournamentid{panic!()}
    if o_t_account.level == t_account.level {panic!()}
    if o_t_account.playerno_int == t_account.opponent {panic!()}

    if state.initializer != t_account.player {panic!()}
    if tournament.owner != program_id{panic!()}
    if opponent_tour_acc.owner != program_id{panic!()}
    if initializer_tour_acc.owner != program_id{panic!()}
    if game_state.owner != program_id{panic!()}
    if state.initialized != 3 {panic!()}
    if tournament.is_writable {panic!()}

    if join.guest_game_hash.len() != 44 {panic!()}

    state.guest = opponent.key.to_bytes();
    state.guestgamehash = join.guest_game_hash;
    state.initialized = 4;
    state.whoseturn = 1;

    let clock: Clock= Clock::get()?;
    let current_time: u64 = clock.unix_timestamp as u64;

    o_t_account.is_playing = 1;
    t_account.opponent_played_on = current_time;
    t_account.is_playing = 1;
    t_account.waiting_opponent_to_join = 0;

    o_t_account.serialize(&mut &mut opponent_tour_acc.data.borrow_mut()[..])?;
    t_account.serialize(&mut &mut initializer_tour_acc.data.borrow_mut()[..])?;
    state.serialize(&mut &mut game_state.data.borrow_mut()[..])?;

    Ok(())
  }
  fn tournament_play(        
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    m: Move,) -> ProgramResult {
    

    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let initializer_tour_acc: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let opponent: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let opponent_tour_acc: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let game_state: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let tournament: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    let mut state: TGameState = TGameState::try_from_slice(&game_state.data.borrow())?;

    let t: Tournament = Tournament::try_from_slice(&tournament.data.borrow())?;

    let opponent_tournament_account: TournamentAccount = TournamentAccount::try_from_slice(&opponent_tour_acc.data.borrow())?;

    let opponent_check: Pubkey = Pubkey::new_from_array(opponent_tournament_account.player);

    let initializer_tournament_account: TournamentAccount = TournamentAccount::try_from_slice(&initializer_tour_acc.data.borrow())?;

    let initializer_account_check: Pubkey = Pubkey::new_from_array(initializer_tournament_account.player);

    let initializer_check: Pubkey = Pubkey::new_from_array(state.initializer);

    if t.is_init != 1{panic!()}
    if t.tournament_id != initializer_tournament_account.tournamentid{panic!()}
    if t.tournament_id != opponent_tournament_account.tournamentid{panic!()}
    if state.initializer != initializer_tournament_account.player{panic!()}
    if state.guest != opponent_tournament_account.player{panic!()}
    if opponent_tournament_account.level == initializer_tournament_account.level {panic!()}
    if opponent_tournament_account.playerno_int == initializer_tournament_account.opponent {panic!()}
    if opponent_tournament_account.opponent == initializer_tournament_account.playerno_int {panic!()}
    if opponent.key != &opponent_check{panic!()}
    if initializer.key != &initializer_account_check{panic!()}
    if initializer.key != &initializer_check{panic!()}
    if tournament.owner != program_id{panic!()}
    if opponent_tour_acc.owner != program_id{panic!()}
    if initializer_tour_acc.owner != program_id{panic!()}
    if game_state.owner != program_id{panic!()}
    if tournament.is_writable {panic!()}
    if opponent_tournament_account.is_playing != 1{panic!()}
    if initializer_tournament_account.is_playing != 1{panic!()}

    
    if state.initialized != 4 {panic!()}
    if m.mymove > 110 {panic!()}
    if m.mymove < 11 {panic!()}

    let clock: Clock= Clock::get()?;
    let current_time: u64 = clock.unix_timestamp as u64;

    let mut turn:u8 = 0;
    let hit:u8 = m.opponents_last_move_is_hit;
    let index: usize = m.mymove as usize;
    let index2: usize = state.lastmove as usize;


    if state.whoseturn == 1{ // 1 initializer
      if !initializer.is_signer{panic!()};
      turn = 2;
      state.ishots += 1;
      state.ghits += hit;
      if state.guest_board_state[index] != 0{panic!()};
      state.guest_board_state[index] += 1;
      state.initializer_board_state[index2] += hit;
    }//hit 2
     //miss 1
     //unchecked 0

    if state.whoseturn == 2{ // 2 guest
      if !opponent.is_signer{panic!()};
      turn = 1;
      state.gshots += 1;
      state.ihits += hit;
      if state.initializer_board_state[index] != 0{panic!()};
      state.initializer_board_state[index] += 1;
      state.guest_board_state[index2] += hit;
    }

    state.lastplaytime = current_time;
    state.whoseturn  = turn;
    state.lastmove = m.mymove;


    state.serialize(&mut &mut game_state.data.borrow_mut()[..])?;

    Ok(())
  }
  fn tournament_claim_victory_by_score(        
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    claim: ClaimVictory,) -> ProgramResult {
    

    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let initializer_tour_acc: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let opponent: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let opponent_tour_acc: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let game_state: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let tournament: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let host: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    let mut state: TGameState = TGameState::try_from_slice(&game_state.data.borrow())?;

    let t: Tournament = Tournament::try_from_slice(&tournament.data.borrow())?;

    let mut o_t_account: TournamentAccount = TournamentAccount::try_from_slice(&opponent_tour_acc.data.borrow())?;

    let opponent_check: Pubkey = Pubkey::new_from_array(o_t_account.player);

    let mut t_account: TournamentAccount = TournamentAccount::try_from_slice(&initializer_tour_acc.data.borrow())?;

    let initializer_account_check: Pubkey = Pubkey::new_from_array(t_account.player);

    let initializer_check: Pubkey = Pubkey::new_from_array(state.initializer);

    if t.is_init != 1{panic!()}
    if t.tournament_id != t_account.tournamentid{panic!()}
    if t.tournament_id != o_t_account.tournamentid{panic!()}
    if state.initializer != t_account.player{panic!()}
    if state.guest != o_t_account.player{panic!()}
    if o_t_account.level == t_account.level {panic!()}
    if o_t_account.playerno_int == t_account.opponent {panic!()}
    if o_t_account.opponent == t_account.playerno_int {panic!()}
    if opponent.key != &opponent_check{panic!()}
    if initializer.key != &initializer_account_check{panic!()}
    if initializer.key != &initializer_check{panic!()}
    if tournament.owner != program_id{panic!()}
    if opponent_tour_acc.owner != program_id{panic!()}
    if initializer_tour_acc.owner != program_id{panic!()}
    if game_state.owner != program_id{panic!()}
    if tournament.is_writable {panic!()}
    if o_t_account.is_playing != 1{panic!()}
    if t_account.is_playing != 1{panic!()}

    
    if state.initialized != 4 {panic!()}


    let clock: Clock= Clock::get()?;
    let current_time: u64 = clock.unix_timestamp as u64;

    let mut game_arr:[u8;17]=[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];

    if claim.s1ax == 1{
      if claim.s1cox > 6 {panic!()}
      if claim.s1cox < 1 {panic!()}
      if claim.s1coy > 10 {panic!()}
      if claim.s1coy < 1 {panic!()}
      game_arr[0] = claim.s1cox+(claim.s1coy*10);
      game_arr[1] = (claim.s1cox + 1)+(claim.s1coy*10);
      game_arr[2] = (claim.s1cox + 2)+(claim.s1coy*10);
      game_arr[3] = (claim.s1cox + 3)+(claim.s1coy*10);
      game_arr[4] = (claim.s1cox + 4)+(claim.s1coy*10);
    }
    if claim.s1ax == 2{
      if claim.s1coy > 6 {panic!()}
      if claim.s1coy < 1 {panic!()}
      if claim.s1cox > 10 {panic!()}
      if claim.s1cox < 1 {panic!()}
      game_arr[0] = (claim.s1coy*10)+(claim.s1cox);
      game_arr[1] = ((claim.s1coy + 1)*10)+(claim.s1cox);
      game_arr[2] = ((claim.s1coy + 2)*10)+(claim.s1cox);
      game_arr[3] = ((claim.s1coy + 3)*10)+(claim.s1cox);
      game_arr[4] = ((claim.s1coy + 4)*10)+(claim.s1cox);
    }
    if claim.s2ax == 1{
      if claim.s2cox > 6 {panic!()}
      if claim.s2cox < 1 {panic!()}
      if claim.s2coy > 10 {panic!()}
      if claim.s2coy < 1 {panic!()}
      game_arr[5] = claim.s2cox+(claim.s2coy*10);
      game_arr[6] = (claim.s2cox + 1)+(claim.s2coy*10);
      game_arr[7] = (claim.s2cox + 2)+(claim.s2coy*10);
      game_arr[8] = (claim.s2cox + 3)+(claim.s2coy*10);
    }
    if claim.s2ax == 2{
      if claim.s2coy > 6 {panic!()}
      if claim.s2coy < 1 {panic!()}
      if claim.s2cox > 10 {panic!()}
      if claim.s2cox < 1 {panic!()}
      game_arr[5] = (claim.s2coy*10)+(claim.s2cox);
      game_arr[6] = ((claim.s2coy + 1)*10)+(claim.s2cox);
      game_arr[7] = ((claim.s2coy + 2)*10)+(claim.s2cox);
      game_arr[8] = ((claim.s2coy + 3)*10)+(claim.s2cox);
    }
    if claim.s3ax == 1{
      if claim.s3cox > 6 {panic!()}
      if claim.s3cox < 1 {panic!()}
      if claim.s3coy > 10 {panic!()}
      if claim.s3coy < 1 {panic!()}
      game_arr[9] = claim.s3cox+(claim.s3coy*10);
      game_arr[10] = (claim.s3cox + 1)+(claim.s3coy*10);
      game_arr[11] = (claim.s3cox + 2)+(claim.s3coy*10);
    }
    if claim.s3ax == 2{
      if claim.s3coy > 6 {panic!()}
      if claim.s3coy < 1 {panic!()}
      if claim.s3cox > 10 {panic!()}
      if claim.s3cox < 1 {panic!()}
      game_arr[9] = (claim.s3coy*10)+(claim.s3cox);
      game_arr[10] = ((claim.s3coy + 1)*10)+(claim.s3cox);
      game_arr[11] = ((claim.s3coy + 2)*10)+(claim.s3cox);
    }
    if claim.s4ax == 1{
      if claim.s4cox > 6 {panic!()}
      if claim.s4cox < 1 {panic!()}
      if claim.s4coy > 10 {panic!()}
      if claim.s4coy < 1 {panic!()}
      game_arr[12] = claim.s4cox+(claim.s4coy*10);
      game_arr[13] = (claim.s4cox + 1)+(claim.s4coy*10);
      game_arr[14] = (claim.s4cox + 2)+(claim.s4coy*10);
    }
    if claim.s4ax == 2{
      if claim.s4coy > 6 {panic!()}
      if claim.s4coy < 1 {panic!()}
      if claim.s4cox > 10 {panic!()}
      if claim.s4cox < 1 {panic!()}
      game_arr[12] = (claim.s4coy*10)+(claim.s4cox);
      game_arr[13] = ((claim.s4coy + 1)*10)+(claim.s4cox);
      game_arr[14] = ((claim.s4coy + 2)*10)+(claim.s4cox);
    }
    if claim.s5ax == 1{
      if claim.s5cox > 6 {panic!()}
      if claim.s5cox < 1 {panic!()}
      if claim.s5coy > 10 {panic!()}
      if claim.s5coy < 1 {panic!()}
      game_arr[15] = claim.s5cox+(claim.s5coy*10);
      game_arr[16] = (claim.s5cox + 1)+(claim.s5coy*10);
    }
    if claim.s5ax == 2{
      if claim.s5coy > 6 {panic!()}
      if claim.s5coy < 1 {panic!()}
      if claim.s5cox > 10 {panic!()}
      if claim.s5cox < 1 {panic!()}
      game_arr[15] = (claim.s5coy*10)+(claim.s5cox);
      game_arr[16] = ((claim.s5coy + 1)*10)+(claim.s5cox);
    }

    let game_hash: keccak::Hash = keccak::hashv(&[&claim.seed.to_string().as_bytes(),&game_arr,&claim.seed.to_string().as_bytes()]);


    if claim.claims == 1 {
      if state.initializergamehash != game_hash.0{panic!()}
    }
    else {
      if state.guestgamehash != game_hash.0{panic!()}
    }

    if claim.claims == 1 {
      for x in 1..18 {
        let u: usize = x as usize;
        let s: usize = game_arr[u] as usize;
        if state.initializer_board_state[s] > 0 && state.initializer_board_state[s] != 2 {
          state.ihits = 0;
          state.ghits = 17;
        }
      }
    }else{
      for x in 1..18 {
        let u: usize = x as usize;
        let s: usize = game_arr[u] as usize;
        if state.guest_board_state[s] > 0 && state.guest_board_state[s] != 2 {
          state.ihits = 17;
          state.ghits = 0;
        }
      }
    }


    let mut draw:bool=false;
    let mut iwins:bool=false;
    let mut gwins:bool=false;

    let mut the_no = 0;
    if t_account.playerno_int>o_t_account.playerno_int{
      the_no = t_account.playerno_int;
    }
    if t_account.playerno_int<o_t_account.playerno_int{
      the_no = o_t_account.playerno_int;
    }

    if state.ishots != state.gshots{panic!()}
    if state.ihits == 17{iwins = true;}
    if state.ghits == 17{gwins = true;}
    if gwins && iwins{draw = true;iwins = false;gwins = false;}

    if iwins == true{

      let us: usize = o_t_account.level as usize;
      let multiply: u64 = t.lvl_get[us] as u64; 
      let reward:u64 = multiply*t.entrance_fee;

      let multiply: u64 = t.host_get as u64;
      let host_fee: u64 = (t.entrance_fee/100)*multiply;

      **opponent_tour_acc.lamports.borrow_mut()-= host_fee;
      **host.lamports.borrow_mut()+= host_fee;

      **opponent_tour_acc.lamports.borrow_mut()-= reward;
      **opponent.lamports.borrow_mut()+= reward;

      let value: u64 = **opponent_tour_acc.lamports.borrow();

      **opponent_tour_acc.lamports.borrow_mut()-= value;
      **initializer_tour_acc.lamports.borrow_mut()+= value;

      let game: u64 = **game_state.lamports.borrow();

      **game_state.lamports.borrow_mut()-= game;
      **initializer_tour_acc.lamports.borrow_mut()+= game;

      let str_player_no: String = the_no.to_string();
      let mut somestr: String = String::from("pppppp");
      let revstr: String = str_player_no.chars().rev().collect::<String>();
      let offset: &usize = &revstr.len();
      somestr.replace_range(..offset, &revstr);
      let offset2: usize = somestr.len();
      let mut find_me: String = t_account.player_find;
      find_me.replace_range(..offset2,&somestr);

      t_account.player_find = find_me;
      t_account.is_playing = 0;
      t_account.waiting_opponent_to_join = 0;
      t_account.level += 1;
      t_account.playerno_int = the_no;
      t_account.opponent_played_on = current_time;

      t_account.serialize(&mut &mut initializer_tour_acc.data.borrow_mut()[..])?;

    }
    if gwins == true{
      let us: usize = t_account.level as usize;
      let multiply: u64 = t.lvl_get[us] as u64; 
      let reward:u64 = multiply*t.entrance_fee;

      let multiply: u64 = t.host_get as u64;
      let host_fee: u64 = (t.entrance_fee/100)*multiply;

      **initializer_tour_acc.lamports.borrow_mut()-= host_fee;
      **host.lamports.borrow_mut()+= host_fee;

      **initializer_tour_acc.lamports.borrow_mut()-= reward;
      **initializer.lamports.borrow_mut()+= reward;

      let value: u64 = **initializer_tour_acc.lamports.borrow();

      **initializer_tour_acc.lamports.borrow_mut()-= value;
      **opponent_tour_acc.lamports.borrow_mut()+= value;

      let game: u64 = **game_state.lamports.borrow();

      **game_state.lamports.borrow_mut()-= game;
      **initializer_tour_acc.lamports.borrow_mut()+= game;

      let str_player_no: String = the_no.to_string();
      let mut somestr: String = String::from("pppppp");
      let revstr: String = str_player_no.chars().rev().collect::<String>();
      let offset: &usize = &revstr.len();
      somestr.replace_range(..offset, &revstr);
      let offset2: usize = somestr.len();
      let mut find_me: String = o_t_account.player_find;
      find_me.replace_range(..offset2,&somestr);

      o_t_account.player_find = find_me;
      o_t_account.is_playing = 0;
      o_t_account.waiting_opponent_to_join = 0;
      o_t_account.level += 1;
      o_t_account.playerno_int = the_no;
      o_t_account.opponent_played_on = current_time;

      o_t_account.serialize(&mut &mut opponent_tour_acc.data.borrow_mut()[..])?;

    }
    if draw == true{

      o_t_account.is_playing = 0;
      o_t_account.waiting_opponent_to_join = 0;
      o_t_account.opponent_played_on = current_time;

      t_account.is_playing = 0;
      t_account.waiting_opponent_to_join = 0;
      t_account.opponent_played_on = current_time;

      let value: u64 = **game_state.lamports.borrow();

      **game_state.lamports.borrow_mut()-= value;
      **initializer.lamports.borrow_mut()+= value;

      t_account.serialize(&mut &mut initializer_tour_acc.data.borrow_mut()[..])?;
      o_t_account.serialize(&mut &mut opponent_tour_acc.data.borrow_mut()[..])?;

    }
    
    Ok(())
  }
  fn eliminate_player_who_hasnt_initialized_his_game(
    accounts: &[AccountInfo],
    program_id: &Pubkey,) -> ProgramResult {


    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    let eliminate: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let eliminate_tour_acc: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let opponent: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let opponent_tour_acc: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let tournament: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let host: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    let t: Tournament = Tournament::try_from_slice(&tournament.data.borrow())?;

    let el_t_account: TournamentAccount = TournamentAccount::try_from_slice(&eliminate_tour_acc.data.borrow())?;

    let el_check: Pubkey = Pubkey::new_from_array(el_t_account.player);

    let mut op_t_account: TournamentAccount = TournamentAccount::try_from_slice(&opponent_tour_acc.data.borrow())?;

    let opponent_check: Pubkey = Pubkey::new_from_array(op_t_account.player);


    if t.is_init != 1{panic!()}
    if t.tournament_id != el_t_account.tournamentid{panic!()}
    if t.tournament_id != op_t_account.tournamentid{panic!()}
    if eliminate.key != &el_check{panic!()}
    if opponent.key != &opponent_check{panic!()}
    if tournament.owner != program_id{panic!()}
    if el_t_account.waiting_opponent_to_join != 0 {panic!()}
    if eliminate_tour_acc.owner != program_id{panic!()}
    if tournament.is_writable {panic!()}
    if el_t_account.is_playing != 0  {panic!()}
    if el_t_account.opponent != op_t_account.playerno_int{panic!()}
    if el_t_account.playerno_int != op_t_account.opponent{panic!()}
    if el_t_account.level != op_t_account.level{panic!()}

    let clock: Clock= Clock::get()?;
    let current_time: u64 = clock.unix_timestamp as u64;
    let time_passed: u64 = current_time - el_t_account.opponent_played_on;

    if time_passed < t.time_is_up {panic!()}

    let mut the_no: u32 = 0;
    if el_t_account.playerno_int>op_t_account.playerno_int{
      the_no = el_t_account.playerno_int;
    }
    if el_t_account.playerno_int<op_t_account.playerno_int{
      the_no = op_t_account.playerno_int;
    }

    let us: usize = el_t_account.level as usize;
    let multiply: u64 = t.lvl_get[us] as u64;
    let reward:u64 = multiply*t.entrance_fee;

    let multiply: u64 = t.host_get as u64;
    let host_fee: u64 = (t.entrance_fee/100)*multiply;

    **eliminate_tour_acc.lamports.borrow_mut()-= host_fee;
    **host.lamports.borrow_mut()+= host_fee;

    **eliminate_tour_acc.lamports.borrow_mut()-= reward;
    **eliminate.lamports.borrow_mut()+= reward;

    let value: u64 = **eliminate_tour_acc.lamports.borrow();

    **eliminate_tour_acc.lamports.borrow_mut()-= value;
    **opponent_tour_acc.lamports.borrow_mut()+= value;

    let str_player_no: String = the_no.to_string();
    let mut somestr: String = String::from("pppppp");
    let revstr: String = str_player_no.chars().rev().collect::<String>();
    let offset: &usize = &revstr.len();
    somestr.replace_range(..offset, &revstr);
    let offset2: usize = somestr.len();
    let mut find_me: String = op_t_account.player_find;
    find_me.replace_range(..offset2,&somestr);

    op_t_account.player_find = find_me;
    op_t_account.is_playing = 0;
    op_t_account.waiting_opponent_to_join = 0;
    op_t_account.level += 1;
    op_t_account.playerno_int = the_no;

    op_t_account.serialize(&mut &mut opponent_tour_acc.data.borrow_mut()[..])?;


    Ok(())
  }
  fn eliminate_player_who_hasnt_made_his_move(
    accounts: &[AccountInfo],
    program_id: &Pubkey,) -> ProgramResult {


    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    let initializer: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let initializer_tour_acc: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let opponent: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let opponent_tour_acc: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let game_state: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let tournament: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    let host: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    let state: TGameState = TGameState::try_from_slice(&game_state.data.borrow())?;

    let t: Tournament = Tournament::try_from_slice(&tournament.data.borrow())?;

    let mut opponent_tournament_account: TournamentAccount = TournamentAccount::try_from_slice(&opponent_tour_acc.data.borrow())?;

    let opponent_check: Pubkey = Pubkey::new_from_array(opponent_tournament_account.player);

    let mut initializer_tournament_account: TournamentAccount = TournamentAccount::try_from_slice(&initializer_tour_acc.data.borrow())?;

    let initializer_account_check: Pubkey = Pubkey::new_from_array(initializer_tournament_account.player);

    let initializer_check: Pubkey = Pubkey::new_from_array(state.initializer);

    if t.is_init != 1{panic!()}
    if t.tournament_id != initializer_tournament_account.tournamentid{panic!()}
    if t.tournament_id != opponent_tournament_account.tournamentid{panic!()}
    if state.initializer != initializer_tournament_account.player{panic!()}
    if state.guest != opponent_tournament_account.player{panic!()}
    if opponent_tournament_account.level == initializer_tournament_account.level {panic!()}
    if opponent_tournament_account.opponent == initializer_tournament_account.playerno_int {panic!()}
    if opponent_tournament_account.playerno_int == initializer_tournament_account.opponent {panic!()}
    if opponent.key != &opponent_check{panic!()}
    if initializer.key != &initializer_account_check{panic!()}
    if initializer.key != &initializer_check{panic!()}
    if tournament.owner != program_id{panic!()}
    if opponent_tour_acc.owner != program_id{panic!()}
    if initializer_tour_acc.owner != program_id{panic!()}
    if game_state.owner != program_id{panic!()}
    if tournament.is_writable {panic!()}
    if opponent_tournament_account.is_playing != 1{panic!()}
    if initializer_tournament_account.is_playing != 1{panic!()}
    if state.initialized != 4 {panic!()}


    let mut the_no: u32 = 0;
    if initializer_tournament_account.playerno_int>opponent_tournament_account.playerno_int{
      the_no = initializer_tournament_account.playerno_int;
    }
    if initializer_tournament_account.playerno_int<opponent_tournament_account.playerno_int{
      the_no = opponent_tournament_account.playerno_int;
    }

    let mut iwins:bool=false;
    let mut gwins:bool=false;

    let clock: Clock= Clock::get()?;
    let current_time: u64 = clock.unix_timestamp as u64;


    if state.whoseturn == 1{
      let time_passed: u64 = current_time - state.lastplaytime;
      if time_passed > t.time_is_up {gwins=true;}
      if time_passed < t.time_is_up {panic!()}
    }
    if state.whoseturn == 2{
      let time_passed: u64 = current_time - state.lastplaytime;
      if time_passed > t.time_is_up {iwins=true;}
      if time_passed < t.time_is_up {panic!()}
    }

    if !iwins && !gwins{panic!()}
    if !iwins && gwins{panic!()}

    if iwins == true{

      let us: usize = opponent_tournament_account.level as usize;
      let multiply: u64 = t.lvl_get[us] as u64; 
      let reward:u64 = multiply*t.entrance_fee;

      let multiply: u64 = t.host_get as u64;
      let host_fee: u64 = (t.entrance_fee/100)*multiply;

      **opponent_tour_acc.lamports.borrow_mut()-= host_fee;
      **host.lamports.borrow_mut()+= host_fee;

      **opponent_tour_acc.lamports.borrow_mut()-= reward;
      **opponent.lamports.borrow_mut()+= reward;

      let value: u64 = **opponent_tour_acc.lamports.borrow();

      **opponent_tour_acc.lamports.borrow_mut()-= value;
      **initializer_tour_acc.lamports.borrow_mut()+= value;

      let game: u64 = **game_state.lamports.borrow();

      **game_state.lamports.borrow_mut()-= game;
      **initializer_tour_acc.lamports.borrow_mut()+= game;

      let str_player_no: String = the_no.to_string();
      let mut somestr: String = String::from("pppppp");
      let revstr: String = str_player_no.chars().rev().collect::<String>();
      let offset: &usize = &revstr.len();
      somestr.replace_range(..offset, &revstr);
      let offset2: usize = somestr.len();
      let mut find_me: String = initializer_tournament_account.player_find;
      find_me.replace_range(..offset2,&somestr);

      initializer_tournament_account.player_find = find_me;
      initializer_tournament_account.is_playing = 0;
      initializer_tournament_account.waiting_opponent_to_join = 0;
      initializer_tournament_account.level += 1;
      initializer_tournament_account.playerno_int = the_no;
      initializer_tournament_account.opponent_played_on = current_time;

      initializer_tournament_account.serialize(&mut &mut initializer_tour_acc.data.borrow_mut()[..])?;

    }
    if gwins == true{

      let us: usize = initializer_tournament_account.level as usize;
      let multiply: u64 = t.lvl_get[us] as u64; 
      let reward:u64 = multiply*t.entrance_fee;

      let multiply: u64 = t.host_get as u64;
      let host_fee: u64 = (t.entrance_fee/100)*multiply;

      **initializer_tour_acc.lamports.borrow_mut()-= host_fee;
      **host.lamports.borrow_mut()+= host_fee;

      **initializer_tour_acc.lamports.borrow_mut()-= reward;
      **initializer.lamports.borrow_mut()+= reward;

      let value: u64 = **initializer_tour_acc.lamports.borrow();

      **initializer_tour_acc.lamports.borrow_mut()-= value;
      **opponent_tour_acc.lamports.borrow_mut()+= value;

      let game: u64 = **game_state.lamports.borrow();

      **game_state.lamports.borrow_mut()-= game;
      **initializer_tour_acc.lamports.borrow_mut()+= game;

      let str_player_no: String = the_no.to_string();
      let mut somestr: String = String::from("pppppp");
      let revstr: String = str_player_no.chars().rev().collect::<String>();
      let offset: &usize = &revstr.len();
      somestr.replace_range(..offset, &revstr);
      let offset2: usize = somestr.len();
      let mut find_me: String = opponent_tournament_account.player_find;
      find_me.replace_range(..offset2,&somestr);

      opponent_tournament_account.player_find = find_me;
      opponent_tournament_account.is_playing = 0;
      opponent_tournament_account.waiting_opponent_to_join = 0;
      opponent_tournament_account.level += 1;
      opponent_tournament_account.playerno_int = the_no;
      opponent_tournament_account.opponent_played_on = current_time;

      opponent_tournament_account.serialize(&mut &mut opponent_tour_acc.data.borrow_mut()[..])?;

    }

    Ok(())
  }
  fn initialize_counter(        
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    t_counter: InitTournamentCounter) -> ProgramResult {

      let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

      let authority: &AccountInfo<'_> = next_account_info(accounts_iter)?;
      let tournament_counter: &AccountInfo<'_> = next_account_info(accounts_iter)?;

      let a_k: Pubkey = Pubkey::from_str("4YbLBRXwseG1NuyJbteSD5u81Q2QjFqJBp6JmxwYBKYm").unwrap();
      if authority.key != &a_k {panic!()}
      if !authority.is_signer {panic!()}

      let counter_no_str: String = t_counter.counter_no.to_string();

      if tournament_counter.owner == &ID && **tournament_counter.lamports.borrow() != 0{
      let value: u64 = **tournament_counter.lamports.borrow(); 
      invoke_signed(&system_instruction::transfer(tournament_counter.key, authority.key, value),
              &[
              authority.clone(), 
              tournament_counter.clone(),
          ],
          &[&[b"count",counter_no_str.as_ref(), &[t_counter.bump]]])?;
      }

      invoke_signed(
          &system_instruction::create_account(
              &authority.key, 
              &tournament_counter.key,
              t_counter.rent,
              24,
              &program_id
          ),
          &[
              authority.clone(), 
              tournament_counter.clone(),
          ],
          &[&[b"count",counter_no_str.as_ref(), &[t_counter.bump]]], 
      )?;

      let counter: TournamentCounter = TournamentCounter{
        empty:"E".to_string(),
        counter_no:t_counter.counter_no,
        player_participating:0,
        capacity:t_counter.capacity,
        tournament_id:t_counter.tournament_id,
      };

      counter.serialize(&mut &mut tournament_counter.data.borrow_mut()[..])?;


      Ok(())
  }
  fn update_counter_finder(        
    accounts: &[AccountInfo],
    c_finder: CounterFinder) -> ProgramResult {
  
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();
  
        let authority: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let finder: &AccountInfo<'_> = next_account_info(accounts_iter)?;
  
        let a_k: Pubkey = Pubkey::from_str("4YbLBRXwseG1NuyJbteSD5u81Q2QjFqJBp6JmxwYBKYm").unwrap();
        if authority.key != &a_k {panic!()}
        if !authority.is_signer {panic!()}
  
      let f: CounterFinder = CounterFinder{
        finder_no:c_finder.finder_no,
        counters:c_finder.counters,
        tournament_id:c_finder.tournament_id,
      };

      f.serialize(&mut &mut finder.data.borrow_mut()[..])?;

        Ok(())
  }
  fn update_finder_finder(        
        accounts: &[AccountInfo],
        f_finder: FinderFinder) -> ProgramResult {
    
          let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();
  
          let authority: &AccountInfo<'_> = next_account_info(accounts_iter)?;
          let finder: &AccountInfo<'_> = next_account_info(accounts_iter)?;
    
          let a_k: Pubkey = Pubkey::from_str("4YbLBRXwseG1NuyJbteSD5u81Q2QjFqJBp6JmxwYBKYm").unwrap();
          if authority.key != &a_k {panic!()}
          if !authority.is_signer {panic!()}
    
        let f: FinderFinder = FinderFinder{
          finder_no:f_finder.finder_no,
          counters:f_finder.counters,
          tournament_id:f_finder.tournament_id,
        };
  
        f.serialize(&mut &mut finder.data.borrow_mut()[..])?;
    
          Ok(())
  }
  fn delete_account_authority(        
    accounts: &[AccountInfo]) -> ProgramResult {

      let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

      let authority: &AccountInfo<'_> = next_account_info(accounts_iter)?;
      let account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

      let a_k: Pubkey = Pubkey::from_str("4YbLBRXwseG1NuyJbteSD5u81Q2QjFqJBp6JmxwYBKYm").unwrap();
      if authority.key != &a_k {panic!()}
      if !authority.is_signer {panic!()}

      let value: u64 = **account.lamports.borrow();

      **account.lamports.borrow_mut()-= value;
      **authority.lamports.borrow_mut()+= value;

      Ok(())
  }
  fn delete_account(        
      accounts: &[AccountInfo]) -> ProgramResult {
  
        let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

        let authority: &AccountInfo<'_> = next_account_info(accounts_iter)?;
        let account: &AccountInfo<'_> = next_account_info(accounts_iter)?;
  

        if !account.is_signer {panic!()}

        if !account.is_signer {panic!()}
  
        let value: u64 = **account.lamports.borrow();
  
        **account.lamports.borrow_mut()-= value;
        **authority.lamports.borrow_mut()+= value;

  
        Ok(())
  }

}


