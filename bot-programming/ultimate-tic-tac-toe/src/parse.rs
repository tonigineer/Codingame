use std::io;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

pub fn read_input() -> ((i32, i32), Vec<(i32, i32)>) {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let mut inputs = input_line.split_once(" ").unwrap();
    let opponent = (parse_input!(inputs.0, i32), parse_input!(inputs.1, i32));

    input_line.clear();
    io::stdin().read_line(&mut input_line).unwrap();
    let valid_action_count = parse_input!(input_line, i32);

    let mut moves = Vec::with_capacity(valid_action_count as usize);
    for _ in 0..valid_action_count as usize {
        input_line.clear();
        io::stdin().read_line(&mut input_line).unwrap();
        inputs = input_line.split_once(" ").unwrap();
        moves.push((parse_input!(inputs.0, i32), parse_input!(inputs.1, i32)));
    }

    (opponent, moves)
}
