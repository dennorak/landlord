use rand::{thread_rng, prelude::SliceRandom};
use std::collections::HashMap;

macro_rules! sort_field_mode {
    ($vec:ident, $main_field:ident, $second_field:ident) => {
        let mut counts = HashMap::new();
        for item in $vec.iter()
        {
            let count = counts.entry(item.$main_field.clone()).or_insert(0);
            *count += 1;
        }
        $vec.sort_by(|a, b| {
            let count_a = counts.get(&a.$main_field).unwrap();
            let count_b = counts.get(&b.$main_field).unwrap();
            if count_a == count_b
            {
                a.$second_field.cmp(&b.$second_field)
            }
            else
            {
                count_a.cmp(count_b)
            }
        });
    };
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Suit
{
    Spades,
    Hearts,
    Diamonds,
    Clubs,
    Joker,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Card
{
    pub rank: u8,
    pub suit: Suit,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Play
{
    Single(Card),
    Pair(Vec<Card>),
    TripleSolo(Vec<Card>),
    TripleSingle
    {
        triple: Vec<Card>,
        single: Card,
    },
    TripleDouble
    {
        triple: Vec<Card>,
        double: Vec<Card>,
    },
    Airplane // consecutive triples (excl. rank 2) (any number of triples)
    {
        triples: Vec<Vec<Card>>,
    },
    QuadTwoSingle
    {
        quad: Vec<Card>,
        single_1: Vec<Card>,
        single_2: Vec<Card>,
    },
    QuadTwoPair
    {
        quad: Vec<Card>,
        pair_1: Vec<Card>,
        pair_2: Vec<Card>,
    },
    Bomb(Vec<Card>),
    Sequence(Vec<Card>), // cards 3-A in a sequence
}

pub fn get_deck() -> Vec<Card>
{
    let mut deck = Vec::new();
    for rank in 1..14
    {
        for suit in [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs].iter()
        {
            deck.push(Card { rank, suit: suit.clone() });
        }
    }
    deck.push(Card { rank: 1, suit: Suit::Joker });
    deck.push(Card { rank: 2, suit: Suit::Joker });
    deck
}

pub struct Player
{
    pub hand: Vec<Card>,
}

pub struct Game
{
    pub players: [Player; 3],
    pub current_turn_idx: usize,
    pub pass_count: u8,
    pub play_sequence: Vec<Play>,
    pub center_pile: Vec<Card>,
    pub winner: Option<usize>,
    pub landlord: Option<usize>,
}

impl Game
{
    fn new() -> Self
    {
        // set up game state
        let mut players = [
            Player { hand: Vec::new() },
            Player { hand: Vec::new() },
            Player { hand: Vec::new() }
        ];
        let mut center_pile = Vec::new();

        // shuffle deck
        let mut deck = get_deck();
        deck.shuffle(&mut thread_rng());

        // deal cards for each player
        for i in 0..3
        {
            for _ in 0..17
            {
                players[i].hand.push(deck.pop().unwrap());
            }
        }

        // put the rest of the deck in the center pile
        for card in deck.iter()
        {
            center_pile.push(card.clone());
        }

        // return the new game
        Game {
            players,
            current_turn_idx: 0,
            pass_count: 0,
            play_sequence: Vec::new(),
            center_pile,
            winner: None,
            landlord: None,
        }
    }

    pub fn take_landlord(&mut self, player_idx: usize) -> Result<(), String>
    {
        // make sure the player is valid
        if player_idx >= 3
        {
            return Err("Invalid player index".to_string());
        }
        // make sure the pile exists
        if self.center_pile.len() == 0
        {
            return Err("Center pile is empty".to_string());
        }

        // add the center pile to the player's hand
        for card in self.center_pile.iter()
        {
            self.players[player_idx].hand.push(card.clone());
        }
        // clear the center pile
        self.center_pile.clear();
        // set the current turn to the player
        self.current_turn_idx = player_idx;
        // set the landlord
        self.landlord = Some(player_idx);
        Ok(())
    }

    pub fn play_cards(&mut self, player_idx: usize, cards: &mut Vec<Card>) -> Result<(), String>
    {
        // make sure game isn't won
        if self.winner.is_some()
        {
            return Err("Game is already won".to_string());
        }
        // make sure the player is valid
        if player_idx >= 3
        {
            return Err("Invalid player index".to_string());
        }
        // make sure the player has the cards
        for card in cards.iter()
        {
            if !self.players[player_idx].hand.contains(card)
            {
                return Err("Player does not have the card".to_string());
            }
        }

        // get play from cards
        let play = Self::get_play(cards)?;
        
        // make sure the cards are valid
        if !Self::is_valid_play(&self.play_sequence, &play, self.pass_count)
        {
            return Err("Invalid play".to_string());
        }
        // remove the cards from the player's hand
        for card in cards.iter()
        {
            let idx = self.players[player_idx].hand.iter().position(|x| *x == *card).unwrap();
            self.players[player_idx].hand.remove(idx);
        }
        // set the current sequence
        self.play_sequence.push(play);
        // set the current turn to the next player
        self.current_turn_idx = (self.current_turn_idx + 1) % 3;
        // reset the pass count
        self.pass_count = 0;

        // check if the player won
        if self.players[player_idx].hand.len() == 0
        {
            self.winner = Some(player_idx);
        }
        Ok(())
    }

    pub fn pass(&mut self, player_idx: usize) -> Result<(), String>
    {
        // make sure game isn't won
        if self.winner.is_some()
        {
            return Err("Game is already won".to_string());
        }
        // make sure the player is valid
        if player_idx >= 3
        {
            return Err("Invalid player index".to_string());
        }
        // check the player didn't pass twice in a row
        if self.pass_count == 2
        {
            return Err("Can't pass again on same turn".to_string());
        }
        // increment the pass count
        self.pass_count += 1;
        // set the current turn to the next player
        self.current_turn_idx = (self.current_turn_idx + 1) % 3;
        Ok(())
    }

    pub fn get_player(&self, player_idx: usize) -> &Player
    {
        &self.players[player_idx]
    }

    pub fn get_center_pile(&self) -> &Vec<Card>
    {
        &self.center_pile
    }

    pub fn get_play_sequence(&self) -> &Vec<Play>
    {
        &self.play_sequence
    }

    pub fn get_pass_count(&self) -> u8
    {
        self.pass_count
    }

    pub fn get_current_turn_idx(&self) -> usize
    {
        self.current_turn_idx
    }

    pub fn get_landlord(&self) -> Option<usize>
    {
        self.landlord
    }

    pub fn get_winner(&self) -> Option<usize>
    {
        self.winner
    }

    fn get_play(cards: &mut Vec<Card>) -> Result<Play, String>
    {
        sort_field_mode!(cards, suit, rank);

        match cards.len()
        {
            1 => Ok(Play::Single(cards[0].clone())),
            2 => {
                unimplemented!();
            },
            _ => Err("Invalid play".to_string())
        }
    }

    fn is_valid_play(sequence: &Vec<Play>, play: &Play, pass_count: u8) -> bool
    {
        unimplemented!();
    }
}

fn main() {
    println!("Hello, world!");
}
