#![feature(custom_attribute)]
#![feature(specialization)]

extern crate pyo3;

use goban::pieces::goban::Goban;
use goban::pieces::stones::Color;
use goban::pieces::util::coord::{Coord, Order};
use goban::rules::game::Game;
use goban::rules::game::GobanSizes;
use goban::rules::game::Move;
use goban::rules::EndGame;
use goban::rules::Player;
use goban::rules::Rule;
use pyo3::prelude::*;
use goban::rules::Player::{White, Black};

fn vec_color_to_u8(vec: &Vec<Color>) -> Vec<u8> {
    vec.iter().map(|color| *color as u8).collect()
}

#[pymodule]
pub fn libgoban(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<IGoban>()?;
    m.add_class::<IGame>()?;
    Ok(())
}

#[pyclass(name = Goban)]
pub struct IGoban {
    goban: Goban,
}

#[pymethods]
impl IGoban {
    #[new]
    pub fn __new__(obj: &PyRawObject, arr: Vec<u8>) {
        let stones: Vec<Color> = arr.into_iter().map(|v| v.into()).collect();
        obj.init({
            IGoban {
                goban: Goban::from_array(&stones, Order::RowMajor),
            }
        });
    }

    pub fn raw(&self) -> PyResult<Vec<u8>> {
        Ok(self.goban.tab().iter().map(|color| *color as u8).collect())
    }

    pub fn raw_split(&self) -> PyResult<(Vec<bool>, Vec<bool>)> {
        Ok((self.goban.b_stones().clone(), self.goban.w_stones().clone()))
    }

    pub fn pretty_string(&self) -> PyResult<String> {
        Ok(self.goban.pretty_string())
    }
}

#[pyclass]
pub struct IGame {
    game: Game,
}

#[pymethods]
impl IGame {
    #[new]
    ///
    /// By default the rule are chinese
    ///
    pub fn __new__(obj: &PyRawObject, size: usize) {
        let s = match size {
            9 => GobanSizes::Nine,
            13 => GobanSizes::Thirteen,
            19 => GobanSizes::Nineteen,
            _ => panic!("You must choose 9, 13, 19"),
        };

        obj.init({
            IGame {
                game: Game::new(s, Rule::Chinese),
            }
        });
    }

    pub fn put_handicap(&mut self, coords: Vec<Coord>) -> PyResult<()> {
        self.game.put_handicap(&coords);
        Ok(())
    }

    pub fn size(&self) -> usize {
        *self.game.goban().size()
    }

    ///
    /// Get all the plays
    /// each element represents an vector.
    ///
    pub fn plays(&self) -> PyResult<Vec<IGoban>> {
        Ok(self
            .game
            .plays()
            .iter()
            .map(|goban| IGoban {
                goban: goban.clone(),
            })
            .collect())
    }

    pub fn raw_plays(&self) -> PyResult<Vec<Vec<u8>>> {
        Ok(self
            .game
            .plays()
            .iter()
            .map(|goban| vec_color_to_u8(&goban.tab()))
            .collect())
    }

    pub fn raw_plays_split(&self) -> PyResult<Vec<(Vec<bool>, Vec<bool>)>> {
        Ok(self
            .game
            .plays()
            .iter()
            .map(|goban| (goban.b_stones().clone(), goban.w_stones().clone()))
            .collect())
    }

    ///
    /// Return the underlying goban
    ///
    pub fn goban(&self) -> PyResult<IGoban> {
        Ok(IGoban {
            goban: self.game.goban().clone(),
        })
    }

    ///
    /// Get the goban in a Vec<u8>
    ///
    pub fn raw_goban(&self) -> PyResult<Vec<u8>> {
        Ok(vec_color_to_u8(self.game.goban().tab()))
    }

    ///
    /// Get the goban in a split.
    ///
    pub fn raw_goban_split(&self) -> PyResult<(Vec<bool>, Vec<bool>)> {
        Ok((
            self.game.goban().b_stones().clone(),
            self.game.goban().w_stones().clone(),
        ))
    }

    ///
    /// Resume the game after to passes
    ///
    pub fn resume(&mut self) -> PyResult<()> {
        self.game.resume();
        Ok(())
    }

    ///
    /// Get prisoners of the game.
    /// (black prisoners, white prisoners)
    ///
    pub fn prisoners(&self) -> PyResult<(u32, u32)> {
        Ok(*self.game.prisoners())
    }

    ///
    /// Return the komi of the game
    ///
    pub fn komi(&self) -> PyResult<f32> {
        Ok(*self.game.komi())
    }

    ///
    /// Set the komi
    ///
    pub fn set_komi(&mut self, komi: f32) -> PyResult<()> {
        self.game.set_komi(komi);
        Ok(())
    }
    ///
    /// Return true if the game is over
    ///
    pub fn over(&self) -> PyResult<bool> {
        Ok(self.game.is_over())
    }

    ///
    /// Returns the score
    /// (black score, white score)
    /// returns -1 if resign
    /// ex:
    /// (-1,0) Black resigned so white won
    /// (0,-1) White resigned so black won
    ///
    pub fn outcome(&self) -> PyResult<Option<(f32, f32)>> {
        Ok(match self.game.outcome() {
            None => None,
            Some(endgame) => match endgame {
                EndGame::Score(x, y) => Some((x, y)),
                EndGame::WinnerByResign(res) => match res {
                    // White win
                    Player::White => Some((-1., 0.)),
                    // Black win
                    Player::Black => Some((0., -1.)),
                },
            },
        })
    }

    /// Get the current turn
    /// true White
    /// false Black
    pub fn turn(&self) -> bool {
        match self.game.turn() {
            Player::White => true,
            Player::Black => false,
        }
    }

    ///
    /// Don't check if the play is legal.
    ///
    pub fn play(&mut self, play: Option<Coord>) -> PyResult<()> {
        match play {
            Some(mov) => self.game.play(Move::Play(mov.0, mov.1)),
            None => self.game.play(Move::Pass),
        };
        Ok(())
    }

    /// Resign
    /// player
    /// true White
    /// false Black
    pub fn resign(&mut self, player: bool) -> PyResult<()> {
        self.game.play(Move::Resign(
            if player { White } else { Black }
        ));
        Ok(())
    }

    /// All the legals
    pub fn legals(&self) -> PyResult<Vec<Coord>> {
        Ok(self.game.legals().collect())
    }

    pub fn pop(&mut self) -> PyResult<()> {
        self.game.pop();
        Ok(())
    }

    pub fn calculate_territories(&self) -> PyResult<(f32, f32)> {
        Ok(self.game.goban().calculate_territories())
    }

    pub fn display_goban(&self) -> PyResult<()> {
        self.game.display_goban();
        Ok(())
    }
}
