fn main() {
    let mut board = [[0i32; 3]; 3];
    board[1][1] = 3; // center
    board[0][1] = 2;
    board[1][0] = 2;
    board[1][2] = 2;
    board[2][1] = 2;
    step(&mut board);
    println!("after wave1: {:?}", board);
    step_neighbors(&mut board);
    println!("after wave2: {:?}", board);
}

fn step(board: &mut [[i32; 3]; 3]) {
    // center explodes
    board[1][1] -= 4;
    board[0][1] += 1;
    board[2][1] += 1;
    board[1][0] += 1;
    board[1][2] += 1;
}

fn step_neighbors(board: &mut [[i32; 3]; 3]) {
    // edges explode
    // top
    board[0][1] -= 3;
    board[1][1] += 1;
    board[0][0] += 1;
    board[0][2] += 1;
    // bottom
    board[2][1] -= 3;
    board[1][1] += 1;
    board[2][0] += 1;
    board[2][2] += 1;
    // left
    board[1][0] -= 3;
    board[1][1] += 1;
    board[0][0] += 1;
    board[2][0] += 1;
    // right
    board[1][2] -= 3;
    board[1][1] += 1;
    board[0][2] += 1;
    board[2][2] += 1;
}
