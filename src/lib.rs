#![feature(specialization)]

extern crate pyo3;

use goban::pieces::goban::Goban;
use goban::pieces::stones::Color;
use goban::pieces::util::coord::{Point, Order};
use goban::rules::{GobanSizes, Move};
use goban::rules::Player;
use goban::rules::Rule;
use pyo3::prelude::*;
use goban::rules::Player::{White, Black};
use std::ops::Deref;
use goban::rules::game::Game;


fn to_color(b: bool) -> Color {
    match b {
        true => Color::White,
        false => Color::Black
    }
}

fn vec_color_to_u8(vec: Vec<Color>) -> Vec<u8> {
    vec.iter().map(|color| *color as u8).collect()
}

fn vec_color_to_raw_split(vec: Vec<Color>) -> (Vec<bool>, Vec<bool>) {
    let mut black_stones = vec![false; vec.len()];
    let mut white_stones = vec![false; vec.len()];

    for i in 0..vec.len() {
        match vec[i] {
            Color::Black => black_stones[i] = true,
            Color::White => white_stones[i] = true,
            _ => ()
        }
    }
    (black_stones, white_stones)
}

#[pymodule]
pub fn libgoban(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<IGoban>()?;
    m.add_class::<IGame>()?;
    Ok(())
}

#[pyclass(name = Goban)]
#[derive(Clone)]
pub struct IGoban {
    goban: Goban,
}

impl Deref for IGoban {
    type Target = Goban;

    fn deref(&self) -> &Self::Target {
        &self.goban
    }
}

impl From<Goban> for IGoban {
    fn from(goban: Goban) -> Self {
        IGoban {
            goban
        }
    }
}

impl From<&Goban> for IGoban {
    fn from(goban: &Goban) -> Self {
        IGoban {
            goban: goban.clone()
        }
    }
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
        Ok(vec_color_to_u8(self.tab()))
    }

    pub fn raw_split(&self) -> PyResult<(Vec<bool>, Vec<bool>)> {
        Ok(vec_color_to_raw_split(self.tab()))
    }

    pub fn pretty_string(&self) -> PyResult<String> {
        Ok(self.goban.pretty_string())
    }
}

#[pyclass]
#[derive(Clone)]
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

    pub fn put_handicap(&mut self, coords: Vec<Point>) -> PyResult<()> {
        self.game.put_handicap(&coords);
        Ok(())
    }

    pub fn size(&self) -> PyResult<usize> {
        Ok(self.game.goban().size())
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
        Ok(
            vec_color_to_raw_split(self.game.goban().tab())
        )
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
        Ok(self.game.prisoners())
    }

    ///
    /// Return the komi of the game
    ///
    pub fn komi(&self) -> PyResult<f32> {
        Ok(self.game.komi())
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
    /// Returns the calculated score.
    ///
    pub fn calculate_score(&self) -> PyResult<(f32, f32)> {
        Ok(self.game.calculate_score())
    }

    ///
    /// Return the winner true for white
    /// false for black
    /// panics if the game is not finished
    ///
    pub fn get_winner(&self) -> PyResult<Option<bool>> {
        Ok(match self.game.outcome() {
            None => panic!("Game not finished"),
            Some(o) => match o.get_winner() {
                None => None,
                Some(o) => match o {
                    Black => Some(false),
                    White => Some(true)
                }
            }
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
    pub fn play(&mut self, play: Option<Point>) -> PyResult<()> {
        match play {
            Some(mov) => self.game.play(Move::Play(mov.0, mov.1)),
            None => self.game.play(Move::Pass),
        };
        Ok(())
    }

    pub fn play_and_clone(&mut self, play: Option<Point>) -> PyResult<Self> {
        self.play(play);
        Ok(self.clone())
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
    pub fn legals(&self) -> PyResult<Vec<Point>> {
        Ok(self.game.legals().collect())
    }

    pub fn calculate_territories(&self) -> PyResult<(f32, f32)> {
        Ok(self.game.goban().calculate_territories())
    }

    pub fn is_point_an_eye(&self, point: Point, color: bool) -> bool {
        self.game.goban().is_point_an_eye(point, to_color(color))
    }

    pub fn display_goban(&self) -> PyResult<()> {
        self.game.display_goban();
        Ok(())
    }
}
