extern crate pyo3;

use goban::pieces::goban::Goban;
use goban::pieces::stones::{Color, Stone};
use goban::pieces::util::coord::{Point, Order};
use goban::rules::{GobanSizes, Move};
use goban::rules::Player;
use goban::rules::Rule;
use pyo3::prelude::*;
use goban::rules::Player::{White, Black};
use std::ops::Deref;
use goban::rules::game::Game;
use pyo3::exceptions;

#[inline]
fn to_color(b: bool) -> Color {
    match b {
        true => Color::White,
        false => Color::Black
    }
}

#[inline]
fn to_bit_tuple(b: Color) -> (bool, bool) {
    match b {
        Color::Black => (true, false),
        Color::White => (false, true),
        Color::None => (false, false)
    }
}

#[inline]
fn vec_color_to_u8(vec: Vec<Color>) -> Vec<u8> {
    vec.into_iter().map(|color| color as u8).collect()
}

#[inline]
fn vec_color_to_raw_split(vec: Vec<Color>) -> (Vec<bool>, Vec<bool>) {
    vec.into_iter().map(to_bit_tuple).unzip()
}

#[pymodule]
pub fn libgoban(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyGoban>()?;
    m.add_class::<PyGame>()?;
    Ok(())
}

#[pyclass(name = Goban)]
#[derive(Clone, Hash, Debug)]
pub struct PyGoban {
    goban: Goban,
}

impl Deref for PyGoban {
    type Target = Goban;

    fn deref(&self) -> &Self::Target {
        &self.goban
    }
}

impl From<Goban> for PyGoban {
    fn from(goban: Goban) -> Self {
        PyGoban {
            goban
        }
    }
}

impl From<&Goban> for PyGoban {
    fn from(goban: &Goban) -> Self {
        PyGoban {
            goban: goban.clone()
        }
    }
}

#[pymethods]
impl PyGoban {
    #[new]
    pub fn new(arr: Vec<u8>) -> Self {
        let stones: Vec<Color> = arr.into_iter().map(|v| v.into()).collect();
        PyGoban {
            goban: Goban::from_array(&stones, Order::RowMajor)
        }
    }

    pub fn raw(&self) -> PyResult<Vec<u8>> {
        Ok(vec_color_to_u8(self.goban.raw()))
    }

    pub fn raw_split(&self) -> PyResult<(Vec<bool>, Vec<bool>)> {
        Ok(vec_color_to_raw_split(self.goban.raw()))
    }

    pub fn pretty_string(&self) -> PyResult<String> {
        Ok(self.goban.pretty_string())
    }
}

#[pyclass(name = Game)]
#[derive(Clone, Debug)]
pub struct PyGame {
    game: Game,
}

#[pymethods]
impl PyGame {
    #[new]
    /// By default the rule are chinese
    pub fn new(size: usize) -> Self {
        let s = match size {
            9 => GobanSizes::Nine,
            13 => GobanSizes::Thirteen,
            19 => GobanSizes::Nineteen,
            _ => panic!("You must choose 9, 13, 19"),
        };

        PyGame {
            game: Game::new(s, Rule::Chinese)
        }
    }

    pub fn put_handicap(&mut self, coords: Vec<Point>) -> PyResult<()> {
        self.game.put_handicap(&coords);
        Ok(())
    }

    /// Returns the size of the goban (height, width)
    pub fn size(&self) -> PyResult<(u8, u8)> {
        Ok(self.game.goban().size())
    }

    /// Return the underlying goban
    pub fn goban(&self) -> PyResult<PyGoban> {
        Ok(PyGoban {
            goban: self.game.goban().clone(),
        })
    }

    /// Get the goban in a Vec<u8>
    pub fn raw_goban(&self) -> PyResult<Vec<u8>> {
        Ok(vec_color_to_u8(self.game.goban().raw()))
    }

    /// Get the goban in a split.
    pub fn raw_goban_split(&self) -> PyResult<(Vec<bool>, Vec<bool>)> {
        Ok(
            vec_color_to_raw_split(self.game.goban().raw())
        )
    }

    /// Resume the game after to passes
    pub fn resume(&mut self) -> PyResult<()> {
        self.game.resume();
        Ok(())
    }

    /// Get prisoners of the game.
    /// (black prisoners, white prisoners)
    pub fn prisoners(&self) -> PyResult<(u32, u32)> {
        Ok(self.game.prisoners())
    }

    /// Return the komi of the game
    pub fn komi(&self) -> PyResult<f32> {
        Ok(self.game.komi())
    }

    /// Set the komi
    pub fn set_komi(&mut self, komi: f32) -> PyResult<()> {
        self.game.set_komi(komi);
        Ok(())
    }

    /// Return true if the game is over
    pub fn over(&self) -> PyResult<bool> {
        Ok(self.game.is_over())
    }

    /// Returns the calculated score.
    pub fn calculate_score(&self) -> PyResult<(f32, f32)> {
        Ok(self.game.calculate_score())
    }

    /// Return the winner true for white
    /// false for black
    /// panics if the game is not finished
    pub fn get_winner(&self) -> PyResult<Option<bool>> {
        match self.game.outcome() {
            None => Err(exceptions::RuntimeError::py_err("Game not finished")),
            Some(o) => Ok(match o.get_winner() {
                None => None,
                Some(o) => match o {
                    Black => Some(false),
                    White => Some(true)
                }
            })
        }
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

    /// Play the move in the go game, pass None to Pass
    /// Don't check if the play is legal.
    pub fn play(&mut self, play: Option<Point>) -> PyResult<()> {
        match play {
            Some(mov) => self.game.play(Move::Play(mov.0, mov.1)),
            None => self.game.play(Move::Pass),
        };
        Ok(())
    }

    /// Play a move then return a clone
    pub fn play_and_clone(&self, play: Option<Point>) -> PyResult<Self> {
        let mut x = self.clone();
        x.play(play).expect("Play the move and clone the game");
        Ok(x)
    }

    /// Resign passing
    /// true resigns White
    /// false resigns Black
    pub fn resign(&mut self, player: bool) -> PyResult<()> {
        self.game.play(Move::Resign(
            if player { White } else { Black }
        ));
        Ok(())
    }

    /// All the legals moves of the baord.
    pub fn legals(&self) -> PyResult<Vec<Point>> {
        Ok(self.game.legals().collect())
    }

    /// return true if the point is legal
    pub fn is_legal(&self, point: Point) -> PyResult<bool> {
        Ok(self.game.check_point(point).is_none())
    }

    /// return all the empty intersection of the board,
    pub fn pseudo_legals(&self) -> PyResult<Vec<Point>> {
        Ok(self.game.pseudo_legals().collect())
    }

    /// Count the territory points for each player.
    pub fn calculate_territories(&self) -> PyResult<(usize, usize)> {
        Ok(self.game.goban().calculate_territories())
    }

    /// Test is a point is an eye.
    pub fn is_point_an_eye(&self, point: Point, color: bool) -> bool {
        self.game.check_eye(Stone { coordinates: point, color: to_color(color) })
    }

    pub fn display_goban(&self) -> PyResult<()> {
        self.game.display_goban();
        Ok(())
    }
}
