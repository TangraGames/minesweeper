#[derive(Debug, Clone, Copy)]
struct Tile {
    revealed: bool,
    has_mine: bool,
    flagged: bool,
    adjacent_mines:u8,
}
struct Grid {
    tiles: Vec<Vec<Tile>>,
    rows: u8,
    cols: u8,
    tile_size:u8,
    top_left_x:i32, // screen coordinates of top left corner of grid
    top_left_y:i32, // screen coordinates of top left corner of grid
}

impl Grid {
    fn new(rows: u8, cols: u8, tile_size:u8, top_left_x:i32, top_left_y:i32) -> Grid {}
}
struct Level {
    rows:u8,
    cols:u8,
    mines:u8,
}
enum State {
    MeinMenu,
    GameWon,
    GameLost,
    GameRunning,
    GamePaused,
}
struct Game {
    state: State,
    grid: Vec<Vec<Tile>>,
    mines:u8,
    screen_width:u16,
    screen_height:u16,
    top_margin:u16,
    bottom_margin:u16,
    left_margin:u16,
    right_margin:u16,
    levels:[Level; 3],
}

