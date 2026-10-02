use engine::{Board, InvalidBoard};

#[test]
fn critical_mass_matches_neighbor_count() {
    let board = Board::new(5, 5).unwrap();
    let cases = vec![
        ((0, 0), 2u32),
        ((0, 4), 2),
        ((4, 0), 2),
        ((4, 4), 2),
        ((0, 2), 3),
        ((2, 0), 3),
        ((4, 2), 3),
        ((2, 4), 3),
        ((2, 2), 4),
        ((3, 1), 4),
    ];
    for ((row, col), expected) in cases {
        let idx = board.index(row, col).unwrap();
        assert_eq!(
            board.critical_mass_index(idx),
            expected,
            "mismatch at ({row},{col})"
        );
    }
}

#[test]
fn thin_boards_use_correct_masses() {
    let two_by_three = Board::new(2, 3).unwrap();
    assert_eq!(two_by_three.critical_mass(0, 0), 2);
    assert_eq!(two_by_three.critical_mass(1, 0), 3);
    assert_eq!(two_by_three.critical_mass(1, 1), 3);

    let three_by_two = Board::new(3, 2).unwrap();
    assert_eq!(three_by_two.critical_mass(0, 0), 2);
    assert_eq!(three_by_two.critical_mass(0, 1), 3);
}

#[test]
fn invalid_board_dimensions_rejected() {
    assert!(matches!(Board::new(0, 5), Err(InvalidBoard::ZeroDimension)));
    assert!(matches!(Board::new(5, 0), Err(InvalidBoard::ZeroDimension)));
    assert!(matches!(Board::new(1, 5), Err(InvalidBoard::TooSmall)));
    assert!(matches!(Board::new(5, 1), Err(InvalidBoard::TooSmall)));
}
