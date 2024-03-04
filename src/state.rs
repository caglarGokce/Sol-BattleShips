use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GameState {
    pub host:[u8;32],
    pub waiting:u8,
    pub initialized:u8,
    pub gameseed:String,
    pub lamports:u64,
    pub initializer: [u8;32],
    pub initializergamehash: [u8;32],
    pub guest: [u8;32],
    pub guestgamehash: [u8;32],
    pub whoseturn:u8,
    pub lastplaytime:u64,
    pub lastmove:u8,
    pub ishots:u8,
    pub ihits:u8,
    pub gshots:u8,
    pub ghits:u8,
    pub initializer_board_state:[u8;128],
    pub guest_board_state:[u8;128],
}
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct TGameState {
    pub game_id:String,
    pub initialized:u8,
    pub gameseed:String,
    pub lamports:u64,
    pub initializer: [u8;32],
    pub initializergamehash: [u8;32],
    pub guest: [u8;32],
    pub guestgamehash: [u8;32],
    pub whoseturn:u8,
    pub lastplaytime:u64,
    pub lastmove:u8,
    pub ishots:u8,
    pub ihits:u8,
    pub gshots:u8,
    pub ghits:u8,
    pub initializer_board_state:[u8;128],
    pub guest_board_state:[u8;128],
}
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct Init{
    pub seed:String,
    pub gameseed:String,
    pub lamports:u64,
    pub initializer_game_hash:[u8;32],
    pub bump:u8
}
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct Join{
    pub seed:String,
    pub guest_game_hash:[u8;32],
}
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct Move{
    pub mymove:u8,
    pub opponents_last_move_is_hit:u8,
}
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct ClaimVictory{
    pub claims:u8,
    pub seed:String,
    pub s1ax:u8,
    pub s1cox:u8,
    pub s1coy:u8,
    pub s2ax:u8,
    pub s2cox:u8,
    pub s2coy:u8,
    pub s3ax:u8,
    pub s3cox:u8,
    pub s3coy:u8,
    pub s4ax:u8,
    pub s4cox:u8,
    pub s4coy:u8,
    pub s5ax:u8,
    pub s5cox:u8,
    pub s5coy:u8,
}
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct Tournament{
    pub is_init:u8,
    pub tournament_id:String,
    pub entrance_fee:u64,
    pub tournament_size:u32,
    pub rent:u64,
    pub starts_at:u64,
    pub time_is_up:u64,
    pub should_match_in:u64,
    pub lvl_get:[u8;30],
    pub winner_get:u8,
    pub organizer_get:u8,
    pub host_get:u8,
    pub number_of_counters:u16
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct TournamentAccount{
    pub player_find:String,
    pub tournamentid:String,
    pub player:[u8;32],
    pub opponent:u32,
    pub level:u8,
    pub playerno_int:u32,
    pub opponent_played_on:u64,
    pub is_playing:u8,
    pub waiting_opponent_to_join:u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct TournamentCounter{
    pub empty:String,
    pub counter_no:u16,
    pub player_participating:u32,
    pub capacity:u32,
    pub tournament_id:String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)] //update 32 times
pub struct CounterFinder{
    pub finder_no:u8,
    pub counters:[u8;256],
    pub tournament_id:String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)] //update once
pub struct FinderFinder{
    pub finder_no:u8,
    pub counters:[u8;32],
    pub tournament_id:String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct InitTournamentCounter{
    pub counter_no:u16,
    pub capacity:u32,
    pub tournament_id:String,
    pub bump:u8,
    pub rent:u64
}

/*
10   11   12   13   14   15   16   17   18   19
20   21   22   23   24   25   26   27   28   29
30   31   32   33   34   35   36   37   38   39
40   41   42   43   44   45   46   47   48   49
50   51   52   53   54   55   56   57   58   59
60   61   62   63   64   65   66   67   68   69
70   71   72   73   74   75   76   77   78   79
80   81   82   83   84   85   86   87   88   89
90   91   92   93   94   95   96   97   98   99
100  101  102  103  104  105  106  107  108  109
*/

/*
1	Carrier	5
2	Battleship	4
3	Cruiser	3
4	Submarine	3
5	Destroyer	2
*/