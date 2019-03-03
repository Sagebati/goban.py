#![feature(custom_attribute)]
#![feature(specialization)]

extern crate pyo3;

use pyo3::prelude::*;
use goban::rules::game::Game;
use goban::rules::game::GobanSizes;
use goban::rules::game::Move;
use goban::pieces::util::Coord;
use goban::rules::EndGame;
use goban::rules::Rule;
use goban::rules::Player;

#[pymodule]
pub fn libshusaku(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<IGame>()?;
    Ok(())
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
        obj.init({
            IGame { game: Game::new(GobanSizes::Custom(size), Rule::Chinese) }
        });
    }

    ///
    /// Get all the plays
    /// each element represents an vector.
    ///
    pub fn plays(&self) -> PyResult<Vec<Vec<u8>>> {
        Ok(self.game.plays()
            .into_iter()
            .map(|goban| goban.tab().clone()).collect())
    }

    pub fn goban(&self) -> PyResult<Vec<u8>> {
        Ok(self.game.goban().tab().clone())
    }

    ///
    /// Resume the game after to passes
    ///
    pub fn resume(&mut self) -> PyResult<()> {
        Ok(self.game.resume())
    }

    ///
    /// Set the komi
    ///
    pub fn komi(&mut self, komi: f32) -> PyResult<()> {
        self.game.set_komi(komi);
        Ok(())
    }
    ///
    /// Return true if the game is over
    ///
    pub fn over(&self) -> PyResult<bool> {
        Ok(self.game.over())
    }

    ///
    /// Returns the score
    /// (black score, white score)
    /// returns Big value if resign
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
                }
            }
        })
    }

    /// Get the current turn
    pub fn turn(&self) -> bool {
        match self.game.turn() {
            Player::White => true,
            Player::Black => false
        }
    }

    ///
    /// Don't check if the play is legal.
    ///
    pub fn play(&mut self, play: Coord) -> PyResult<()> {
        Ok(self.game.play(&Move::Play(play.0, play.1)))
    }

    /// Pass
    pub fn skip(&mut self) -> PyResult<()> {
        Ok(self.game.play(&Move::Pass))
    }

    /// Resign
    pub fn resign(&mut self) -> PyResult<()> {
        Ok(self.game.play(&Move::Resign))
    }

    /// All the legals
    pub fn legals(&self) -> PyResult<Vec<Coord>> {
        Ok(self.game.legals().collect())
    }

    pub fn pop(&mut self) -> PyResult<()> {
        Ok(self.game.pop())
    }

    pub fn calculate_territories(&self) -> PyResult<(f32, f32)> {
        Ok(self.game.calculate_territories())
    }

    pub fn display(&self) -> PyResult<()> {
        self.game.display();
        Ok(())
    }
}
