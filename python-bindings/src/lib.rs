use engine::{
    apply_move as engine_apply_move, new_game as engine_new_game, replay as engine_replay,
    validate_move as engine_validate_move, EngineError, Explosion as EngineExplosion,
    GameResult as EngineGameResult, GameState as EngineGameState, GameStatus as EngineGameStatus,
    IllegalMove, InvalidBoard, InvalidState, Move as EngineMove, MoveResult as EngineMoveResult,
    OrbPlacement, OrbTransfer as EngineOrbTransfer, PlayerId, PlayerState as EnginePlayerState,
    ReactionStep as EngineReactionStep, ReactionTimeline as EngineReactionTimeline, ReplayError,
};
use pyo3::{
    exceptions::{PyException, PyValueError},
    prelude::*,
    types::{PyAny, PyDict, PyType},
};
use serde_json;

pyo3::create_exception!(chain_reaction, PyEngineError, PyException);
pyo3::create_exception!(chain_reaction, PyIllegalMoveError, PyEngineError);
pyo3::create_exception!(chain_reaction, PyInvalidBoardError, PyEngineError);
pyo3::create_exception!(chain_reaction, PyInvalidStateError, PyEngineError);
pyo3::create_exception!(chain_reaction, PyReplayError, PyException);

#[pyfunction]
fn new_game(width: u16, height: u16, players: Vec<PlayerId>) -> PyResult<PyGameState> {
    engine_new_game(width, height, players)
        .map(PyGameState::from)
        .map_err(engine_error_to_py)
}

#[pyfunction]
fn validate_move(state: &PyGameState, player: PlayerId, mv: &PyMove) -> PyResult<()> {
    engine_validate_move(&state.inner, player, mv.inner)
        .map_err(|err| Python::with_gil(|py| illegal_move_to_py(py, err)))
}

#[pyfunction]
fn apply_move(state: &PyGameState, player: PlayerId, mv: &PyMove) -> PyResult<PyMoveResult> {
    engine_apply_move(&state.inner, player, mv.inner)
        .map(PyMoveResult::from)
        .map_err(engine_error_to_py)
}

#[pyfunction]
fn replay(py: Python<'_>, initial_state: &PyGameState, moves: &PyAny) -> PyResult<PyGameState> {
    let py_moves: Vec<Py<PyMove>> = moves.extract()?;
    let engine_moves: Vec<EngineMove> = py_moves.iter().map(|mv| mv.borrow(py).inner).collect();

    engine_replay(&initial_state.inner, &engine_moves)
        .map(PyGameState::from)
        .map_err(|err| Python::with_gil(|py| replay_error_to_py(py, err)))
}

#[pyclass(name = "Move")]
#[derive(Clone, Copy)]
pub struct PyMove {
    pub(crate) inner: EngineMove,
}

#[pymethods]
impl PyMove {
    #[new]
    pub fn new(row: u16, col: u16) -> Self {
        Self {
            inner: EngineMove::new(row, col),
        }
    }

    #[getter]
    fn row(&self) -> u16 {
        self.inner.row
    }

    #[getter]
    fn col(&self) -> u16 {
        self.inner.col
    }

    pub fn as_tuple(&self) -> (u16, u16) {
        (self.inner.row, self.inner.col)
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Move(row={}, col={})",
            self.inner.row, self.inner.col
        ))
    }
}

impl From<EngineMove> for PyMove {
    fn from(inner: EngineMove) -> Self {
        Self { inner }
    }
}

#[pyclass(name = "GameState")]
#[derive(Clone)]
pub struct PyGameState {
    pub(crate) inner: EngineGameState,
}

#[pymethods]
impl PyGameState {
    pub fn clone_state(&self) -> Self {
        self.clone()
    }

    #[getter]
    fn width(&self) -> u16 {
        self.inner.board.width()
    }

    #[getter]
    fn height(&self) -> u16 {
        self.inner.board.height()
    }

    #[getter]
    fn current_player(&self) -> PlayerId {
        self.inner.current_player()
    }

    #[getter]
    fn turn_number(&self) -> u64 {
        self.inner.turn_number
    }

    #[getter]
    fn status(&self) -> PyGameStatus {
        PyGameStatus::from(self.inner.status.clone())
    }

    pub fn living_players(&self) -> Vec<PlayerId> {
        self.inner.living_players()
    }

    pub fn players(&self) -> Vec<PlayerSummary> {
        self.inner
            .players
            .iter()
            .cloned()
            .map(PlayerSummary::from)
            .collect()
    }

    pub fn board_snapshot(&self) -> Vec<Vec<Option<CellView>>> {
        let height = self.inner.board.height() as usize;
        let width = self.inner.board.width() as usize;
        (0..height)
            .map(|row| {
                (0..width)
                    .map(|col| {
                        let idx = self
                            .inner
                            .board
                            .index(row as u16, col as u16)
                            .expect("index must exist");
                        let cell = self.inner.board.cell(idx);
                        cell.owner.map(|owner| CellView {
                            owner,
                            orb_count: cell.orb_count,
                        })
                    })
                    .collect()
            })
            .collect()
    }

    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner).map_err(|err| PyValueError::new_err(err.to_string()))
    }

    #[classmethod]
    pub fn from_json(_cls: &PyType, data: &str) -> PyResult<Self> {
        serde_json::from_str::<EngineGameState>(data)
            .map(PyGameState::from)
            .map_err(|err| PyValueError::new_err(err.to_string()))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "GameState(width={} height={} turn={} status={})",
            self.width(),
            self.height(),
            self.turn_number(),
            self.status().kind()
        ))
    }
}

impl From<EngineGameState> for PyGameState {
    fn from(inner: EngineGameState) -> Self {
        Self { inner }
    }
}

#[pyclass(name = "MoveResult")]
#[derive(Clone)]
pub struct PyMoveResult {
    inner: EngineMoveResult,
}

#[pymethods]
impl PyMoveResult {
    #[getter]
    fn final_state(&self) -> PyGameState {
        PyGameState::from(self.inner.final_state.clone())
    }

    #[getter]
    fn timeline(&self) -> PyReactionTimeline {
        PyReactionTimeline::from(self.inner.timeline.clone())
    }

    #[getter]
    fn outcome(&self) -> PyGameResult {
        PyGameResult::from(self.inner.outcome.clone())
    }

    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner).map_err(|err| PyValueError::new_err(err.to_string()))
    }

    fn __repr__(&self) -> PyResult<String> {
        let status = match self.inner.outcome.status {
            EngineGameStatus::Active => "active",
            EngineGameStatus::Won { .. } => "won",
        };
        Ok(format!(
            "MoveResult(turn={} status={})",
            self.inner.final_state.turn_number, status
        ))
    }
}

impl From<EngineMoveResult> for PyMoveResult {
    fn from(inner: EngineMoveResult) -> Self {
        Self { inner }
    }
}

#[pyclass(name = "ReactionTimeline")]
#[derive(Clone)]
pub struct PyReactionTimeline {
    inner: EngineReactionTimeline,
}

#[pymethods]
impl PyReactionTimeline {
    #[getter]
    fn placement(&self) -> PyOrbPlacement {
        PyOrbPlacement::from(self.inner.placement.clone())
    }

    #[getter]
    fn steps(&self) -> Vec<PyReactionStep> {
        self.inner
            .steps
            .iter()
            .cloned()
            .map(PyReactionStep::from)
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "ReactionTimeline(steps={})",
            self.inner.steps.len()
        ))
    }
}

impl From<EngineReactionTimeline> for PyReactionTimeline {
    fn from(inner: EngineReactionTimeline) -> Self {
        Self { inner }
    }
}

#[pyclass(name = "ReactionStep")]
#[derive(Clone)]
pub struct PyReactionStep {
    inner: EngineReactionStep,
}

#[pymethods]
impl PyReactionStep {
    #[getter]
    fn explosions(&self) -> Vec<PyExplosion> {
        self.inner
            .explosions
            .iter()
            .cloned()
            .map(PyExplosion::from)
            .collect()
    }

    #[getter]
    fn transfers(&self) -> Vec<PyOrbTransfer> {
        self.inner
            .transfers
            .iter()
            .cloned()
            .map(PyOrbTransfer::from)
            .collect()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "ReactionStep(explosions={}, transfers={})",
            self.inner.explosions.len(),
            self.inner.transfers.len()
        ))
    }
}

impl From<EngineReactionStep> for PyReactionStep {
    fn from(inner: EngineReactionStep) -> Self {
        Self { inner }
    }
}

#[pyclass(name = "OrbPlacement")]
#[derive(Clone)]
pub struct PyOrbPlacement {
    inner: OrbPlacement,
}

#[pymethods]
impl PyOrbPlacement {
    #[getter]
    fn cell(&self) -> u32 {
        self.inner.cell
    }

    #[getter]
    fn player(&self) -> PlayerId {
        self.inner.player
    }
}

impl From<OrbPlacement> for PyOrbPlacement {
    fn from(inner: OrbPlacement) -> Self {
        Self { inner }
    }
}

#[pyclass(name = "Explosion")]
#[derive(Clone)]
pub struct PyExplosion {
    inner: EngineExplosion,
}

#[pymethods]
impl PyExplosion {
    #[getter]
    fn cell(&self) -> u32 {
        self.inner.cell
    }

    #[getter]
    fn player(&self) -> PlayerId {
        self.inner.player
    }
}

impl From<EngineExplosion> for PyExplosion {
    fn from(inner: EngineExplosion) -> Self {
        Self { inner }
    }
}

#[pyclass(name = "OrbTransfer")]
#[derive(Clone)]
pub struct PyOrbTransfer {
    inner: EngineOrbTransfer,
}

#[pymethods]
impl PyOrbTransfer {
    #[getter]
    fn from_cell(&self) -> u32 {
        self.inner.from
    }

    #[getter]
    fn to_cell(&self) -> u32 {
        self.inner.to
    }

    #[getter]
    fn player(&self) -> PlayerId {
        self.inner.player
    }
}

impl From<EngineOrbTransfer> for PyOrbTransfer {
    fn from(inner: EngineOrbTransfer) -> Self {
        Self { inner }
    }
}

#[pyclass(name = "GameResult")]
#[derive(Clone)]
pub struct PyGameResult {
    inner: EngineGameResult,
}

#[pymethods]
impl PyGameResult {
    #[getter]
    fn status(&self) -> PyGameStatus {
        PyGameStatus::from(self.inner.status.clone())
    }

    #[getter]
    fn winner(&self) -> Option<PlayerId> {
        self.inner.winner
    }

    #[getter]
    fn losers(&self) -> Vec<PlayerId> {
        self.inner.losers.clone()
    }

    pub fn status_kind(&self) -> &'static str {
        match self.inner.status {
            EngineGameStatus::Active => "active",
            EngineGameStatus::Won { .. } => "won",
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "GameResult(status={}, winner={:?})",
            self.status_kind(),
            self.winner()
        ))
    }
}

impl From<EngineGameResult> for PyGameResult {
    fn from(inner: EngineGameResult) -> Self {
        Self { inner }
    }
}

#[pyclass(name = "GameStatus")]
#[derive(Clone)]
pub struct PyGameStatus {
    inner: EngineGameStatus,
}

#[pymethods]
impl PyGameStatus {
    pub fn kind(&self) -> &'static str {
        match self.inner {
            EngineGameStatus::Active => "active",
            EngineGameStatus::Won { .. } => "won",
        }
    }

    pub fn winner(&self) -> Option<PlayerId> {
        match self.inner {
            EngineGameStatus::Won { winner, .. } => Some(winner),
            _ => None,
        }
    }

    pub fn eliminated(&self) -> Vec<PlayerId> {
        match &self.inner {
            EngineGameStatus::Won { eliminated, .. } => eliminated.clone(),
            _ => Vec::new(),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(match &self.inner {
            EngineGameStatus::Active => "GameStatus(active)".to_string(),
            EngineGameStatus::Won { winner, eliminated } => format!(
                "GameStatus(won, winner={}, eliminated={:?})",
                winner, eliminated
            ),
        })
    }
}

impl From<EngineGameStatus> for PyGameStatus {
    fn from(inner: EngineGameStatus) -> Self {
        Self { inner }
    }
}

#[derive(Clone)]
pub struct PlayerSummary {
    id: PlayerId,
    eliminated: bool,
    has_moved: bool,
}

impl From<EnginePlayerState> for PlayerSummary {
    fn from(player: EnginePlayerState) -> Self {
        Self {
            id: player.id,
            eliminated: player.eliminated,
            has_moved: player.has_moved,
        }
    }
}

impl IntoPy<PyObject> for PlayerSummary {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let dict = PyDict::new(py);
        let _ = dict.set_item("id", self.id);
        let _ = dict.set_item("eliminated", self.eliminated);
        let _ = dict.set_item("has_moved", self.has_moved);
        dict.into()
    }
}

#[derive(Clone)]
pub struct CellView {
    owner: PlayerId,
    orb_count: u32,
}

impl IntoPy<PyObject> for CellView {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let dict = PyDict::new(py);
        let _ = dict.set_item("owner", self.owner);
        let _ = dict.set_item("orb_count", self.orb_count);
        dict.into()
    }
}

fn engine_error_to_py(err: EngineError) -> PyErr {
    Python::with_gil(|py| match err {
        EngineError::IllegalMove(inner) => illegal_move_to_py(py, inner),
        EngineError::InvalidBoard(inner) => invalid_board_to_py(py, inner),
        EngineError::InvalidState(inner) => invalid_state_to_py(py, inner),
    })
}

fn illegal_move_to_py(py: Python<'_>, err: IllegalMove) -> PyErr {
    with_detail(
        py,
        PyIllegalMoveError::new_err(err.to_string()),
        illegal_move_detail(py, &err),
    )
}

fn invalid_board_to_py(py: Python<'_>, err: InvalidBoard) -> PyErr {
    with_detail(
        py,
        PyInvalidBoardError::new_err(err.to_string()),
        simple_detail(py, "InvalidBoard", err.to_string()),
    )
}

fn invalid_state_to_py(py: Python<'_>, err: InvalidState) -> PyErr {
    with_detail(
        py,
        PyInvalidStateError::new_err(err.to_string()),
        simple_detail(py, "InvalidState", err.to_string()),
    )
}

fn replay_error_to_py(py: Python<'_>, err: ReplayError) -> PyErr {
    let message = err.to_string();
    let detail = PyDict::new(py);
    match err {
        ReplayError::Engine(inner) => {
            let _ = detail.set_item("kind", "engine");
            match inner {
                EngineError::IllegalMove(illegal) => {
                    let _ = detail.set_item("error_type", "IllegalMove");
                    let _ = detail.set_item("error", illegal_move_detail(py, &illegal));
                }
                EngineError::InvalidBoard(board) => {
                    let _ = detail.set_item("error_type", "InvalidBoard");
                    let _ = detail.set_item(
                        "error",
                        simple_detail(py, "InvalidBoard", board.to_string()),
                    );
                }
                EngineError::InvalidState(state) => {
                    let _ = detail.set_item("error_type", "InvalidState");
                    let _ = detail.set_item(
                        "error",
                        simple_detail(py, "InvalidState", state.to_string()),
                    );
                }
            }
        }
        ReplayError::IllegalMove { step, error } => {
            let _ = detail.set_item("kind", "illegal_move");
            let _ = detail.set_item("step", step);
            let _ = detail.set_item("error", illegal_move_detail(py, &error));
        }
    }
    with_detail(py, PyReplayError::new_err(message), detail.into())
}

fn illegal_move_detail(py: Python<'_>, err: &IllegalMove) -> PyObject {
    let dict = PyDict::new(py);
    let _ = dict.set_item(
        "kind",
        match err {
            IllegalMove::GameAlreadyFinished => "GameAlreadyFinished",
            IllegalMove::NotPlayersTurn => "NotPlayersTurn",
            IllegalMove::PlayerEliminated => "PlayerEliminated",
            IllegalMove::OutOfBounds => "OutOfBounds",
            IllegalMove::CellOwnedByOpponent { .. } => "CellOwnedByOpponent",
            IllegalMove::InvalidCellIndex(_) => "InvalidCellIndex",
        },
    );
    let _ = dict.set_item("message", err.to_string());
    match err {
        IllegalMove::CellOwnedByOpponent { owner } => {
            let _ = dict.set_item("owner", owner);
        }
        IllegalMove::InvalidCellIndex(index) => {
            let _ = dict.set_item("cell_index", index);
        }
        _ => {}
    }
    dict.into()
}

fn simple_detail(py: Python<'_>, kind: &str, message: String) -> PyObject {
    let dict = PyDict::new(py);
    let _ = dict.set_item("kind", kind);
    let _ = dict.set_item("message", message);
    dict.into()
}

fn with_detail(py: Python<'_>, err: PyErr, detail: PyObject) -> PyErr {
    if let Ok(value) = err.value(py).setattr("detail", detail) {
        let _ = value;
    }
    err
}

#[pymodule]
pub fn chain_reaction(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add("EngineError", py.get_type::<PyEngineError>())?;
    m.add("IllegalMoveError", py.get_type::<PyIllegalMoveError>())?;
    m.add("InvalidBoardError", py.get_type::<PyInvalidBoardError>())?;
    m.add("InvalidStateError", py.get_type::<PyInvalidStateError>())?;
    m.add("ReplayError", py.get_type::<PyReplayError>())?;

    m.add_class::<PyMove>()?;
    m.add_class::<PyGameState>()?;
    m.add_class::<PyMoveResult>()?;
    m.add_class::<PyReactionTimeline>()?;
    m.add_class::<PyReactionStep>()?;
    m.add_class::<PyOrbPlacement>()?;
    m.add_class::<PyExplosion>()?;
    m.add_class::<PyOrbTransfer>()?;
    m.add_class::<PyGameResult>()?;
    m.add_class::<PyGameStatus>()?;

    m.add_function(wrap_pyfunction!(new_game, m)?)?;
    m.add_function(wrap_pyfunction!(validate_move, m)?)?;
    m.add_function(wrap_pyfunction!(apply_move, m)?)?;
    m.add_function(wrap_pyfunction!(replay, m)?)?;

    Ok(())
}
