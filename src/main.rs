use macroquad::prelude::*;
use macroquad::rand::srand;
use macroquad::rand::gen_range;

/*************************************************************
Classic minesweeper levels:
Beginner     -> 8 x 8 grid, 10 mines
Intermediate -> 16 x 16 grid, 40 mines
Expert       -> 30 x 16 grid, 99 mines
*************************************************************/

// TODO:
// - Track total flagged cells and limit to number of mines
// - Update logic, so that first celected cell is not a mine
// - Add timer and display time on screen
// - Improve main manu and add level selection
// - Add top menu and adjust grid positioning
// - Clean-up draw_grid function, match on game state
// - Move grid, tile, input handling etc code in own modules

const WINDOW_WIDTH:i32 = 800;
const WINDOW_HEIGHT:i32 = 600;
const MAX_TILE_SIZE:f32 = 80.0;
const BACKGROUND:Color = Color::new(0.05, 0.05, 0.05, 1.0);

#[allow(dead_code)]
struct Game {
    rows:u8,
    columns:u8,
    tiles:u16,
    cell_size:f32,
    mines:u16,
    mines_flagged:u16,
    level_time:u64,
}

impl Game {
    fn new(rows:u8, columns:u8, mines:u16) -> Self {
        Self {
            rows,
            columns,
            tiles: rows as u16 * columns as u16,
            cell_size: calculate_tile_size(rows, columns, MAX_TILE_SIZE),
            mines,
            mines_flagged: 0,
            level_time: 0,
        }
    }
}

#[derive(PartialEq)]
enum GameState {
    MeinMenu,
    GameWon,
    GameLost,
    GameRunning,
}

#[derive(Debug, Clone, Copy)]
struct Tile {
    revealed: bool,
    has_mine: bool,
    flagged: bool,
    adjacent_mines:u8,
}
struct Assets {
    one: Rect,
    two: Rect,
    three:Rect,
    four:Rect,
    five:Rect,
    six:Rect,
    seven:Rect,
    eight:Rect,
    bomb:Rect,
    explosion:Rect,
    flag:Rect,
    spritesheet:Texture2D,
    font:Font,
}

impl Default for Assets{
    fn default() -> Self {
        set_pc_assets_folder("../assets");

        // embed the spritesheet into the binary exe file
        let spritesheet = Texture2D::from_file_with_format(include_bytes!("../assets/minesweeper.png"), Some(ImageFormat::Png));
        let font = load_ttf_font_from_bytes(include_bytes!("../assets/Russo_One.ttf")).unwrap();
        spritesheet.set_filter(FilterMode::Nearest);
        build_textures_atlas();

        Self {
            one:Rect::new(0.0, 0.0, 40.0, 40.0),
            two:Rect::new(41.0, 0.0, 40.0, 40.0),
            three:Rect::new(82.0, 0.0, 40.0, 40.0),
            four:Rect::new(123.0, 0.0, 40.0, 40.0),
            five:Rect::new(0.0, 41.0, 40.0, 40.0),
            six:Rect::new(41.0, 41.0, 40.0, 40.0),
            seven:Rect::new(82.0, 41.0, 40.0, 40.0),
            eight:Rect::new(123.0, 41.0, 40.0, 40.0),
            bomb:Rect::new(0.0, 82.0, 40.0, 40.0),
            explosion:Rect::new(41.0, 82.0, 40.0, 40.0),
            flag:Rect::new(82.0, 82.0, 40.0, 40.0),
            spritesheet,
            font,

        }
    }
}

impl Assets {
    fn draw(&self, rect:Rect, x:f32, y:f32, tile_size:f32){
        let size = tile_size / 1.5;
        draw_texture_ex(
            &self.spritesheet, 
            x - size / 2.0, 
            y - size / 2.0, 
            WHITE,
            DrawTextureParams {
                source: Some(rect),
                dest_size: Some(Vec2::new(size, size)),
                ..Default::default()
            }
        );
    }
}

fn calculate_tile_size(rows:u8, columns:u8, max_tile_size:f32) -> f32 {
    let screen_width = screen_width();
    let screen_height = screen_height();

    ((screen_width / columns as f32).min(screen_height / (rows as f32 + 1.0))).min(max_tile_size)
}

// calculate grid offsets to center the grid on the screen
fn calculate_grid_offsets(rows:u8, columns:u8, max_tile_size:f32) -> (f32, f32) {
    let tile_size = calculate_tile_size(rows, columns, max_tile_size);
    let x_offset = (screen_width() - columns as f32 * tile_size) / 2.0;
    let y_offset = (screen_height() - rows as f32 * tile_size) / 2.0;
    (x_offset, y_offset)
}

fn is_tile_in_grid(row:i32, col:i32, grid_rows:u8, grid_cols:u8) ->bool {
    return row >= 0 && row < grid_rows as i32 && col >= 0 && col < grid_cols as i32;
}

fn screen_to_tile_id(mouse_x:f32, mouse_y:f32, columns:i32, rows:i32, tile_size:f32)-> i32 {
    let row:i32 = (mouse_y / tile_size) as i32;
    let col:i32 = (mouse_x / tile_size) as i32;
    if is_tile_in_grid(row, col, rows as u8, columns as u8) {
        return row * columns + col;
    }
    else {
        return -1;
    }
}

fn draw_grid(arr: &[Tile], assets:&Assets, state:&GameState, rows:u8, columns:u8, max_tile_size:f32) {
    let tile_size = calculate_tile_size(rows, columns, max_tile_size);
    let x_offset = (screen_width() - columns as f32 * tile_size) / 2.0;
    let y_offset = (screen_height() - rows as f32 * tile_size) / 2.0;
    for i in 0..arr.len() {
        let x:f32 = x_offset + (i as u8 % columns) as f32 * tile_size;
        let y:f32 = y_offset + (i as u8 / columns) as f32 * tile_size;

        if arr[i].revealed {
            draw_rectangle(x, y, tile_size, tile_size, GRAY);
            if arr[i].has_mine && (state == &GameState::GameRunning || state == &GameState::GameLost){
                assets.draw(assets.explosion, x + tile_size / 2.0, y + tile_size / 2.0, tile_size);
            }

            else if arr[i].has_mine && state == &GameState::GameWon {
                draw_rectangle(x, y, tile_size, tile_size, LIGHTGRAY);
                assets.draw(assets.bomb, x + tile_size / 2.0, y + tile_size / 2.0, tile_size);
            }
            else {

                match arr[i].adjacent_mines {
                    1 => assets.draw(assets.one, x + tile_size / 2.0, y + tile_size / 2.0, tile_size),
                    2 => assets.draw(assets.two, x + tile_size / 2.0, y + tile_size / 2.0, tile_size),
                    3 => assets.draw(assets.three, x + tile_size / 2.0, y + tile_size / 2.0, tile_size),
                    4 => assets.draw(assets.four, x + tile_size / 2.0, y + tile_size / 2.0, tile_size),
                    5 => assets.draw(assets.five, x + tile_size / 2.0, y + tile_size / 2.0, tile_size),
                    6 => assets.draw(assets.six, x + tile_size / 2.0, y + tile_size / 2.0, tile_size),
                    7 => assets.draw(assets.seven, x + tile_size / 2.0, y + tile_size / 2.0, tile_size),
                    8 => assets.draw(assets.eight, x + tile_size / 2.0, y + tile_size / 2.0, tile_size),
                    _ => (),
                }

            }
        }
        else {
            draw_rectangle(x, y, tile_size, tile_size, LIGHTGRAY);
        }
        if arr[i].flagged {
            if state == &GameState::GameRunning {
                assets.draw(assets.flag, x + tile_size / 2.0, y + tile_size / 2.0, tile_size);
            }
            else if state == &GameState::GameLost || state == &GameState::GameWon {
                if arr[i].has_mine {
                    draw_rectangle(x, y, tile_size, tile_size, LIGHTGRAY);
                    assets.draw(assets.bomb, x + tile_size / 2.0, y + tile_size / 2.0, tile_size);
                }
                else {
                    let offset = tile_size / 3.0;
                    assets.draw(assets.flag, x + tile_size / 2.0, y + tile_size / 2.0, tile_size);
                    draw_line(x + offset, y + offset , x + tile_size - offset, y + tile_size - offset, 6.0, BLACK);
                    draw_line(x + tile_size - offset, y + offset , x + offset, y + tile_size - offset, 6.0, BLACK);
                }
            }
        }
        draw_rectangle_lines(x, y, tile_size, tile_size, 1.0,DARKGRAY);
    }
}

fn flag_tile(arr: &mut [Tile], tile_id:usize){
    if !arr[tile_id].revealed {
        arr[tile_id].flagged = !arr[tile_id].flagged;
    }
}

fn reveal_tile(arr: &mut [Tile], tile_id:usize, rows:u8, cols:u8){
    if !arr[tile_id].flagged {
        arr[tile_id].revealed = true;
        if arr[tile_id].adjacent_mines == 0 {
            reveal_adjacent_tiles(arr, tile_id, rows, cols);
        }
    }
}

fn reveal_adjacent_tiles(arr:&mut [Tile], tile_id:usize, rows:u8, cols:u8) {
    if arr[tile_id].adjacent_mines == 0 && !arr[tile_id].has_mine {
        for r in -1..2 {
            for c in -1..2{
                if r == 0 && c == 0 {
                    continue;
                }
                else {
                    let next_row:i32 = tile_id as i32 / cols as i32 + r;
                    let next_col:i32 = tile_id as i32 % cols as i32 + c;
                    if is_tile_in_grid(next_row, next_col, rows, cols)
                    {
                        let next_id = (next_row * cols as i32 + next_col) as usize;
                        if arr[next_id].revealed ==false {
                            arr[next_id].adjacent_mines = num_adjacent_mines(&arr, rows, cols, next_id);
                            if !arr[next_id].has_mine {
                                reveal_tile(arr, next_id, rows, cols);
                            }
                        }
                    }
                }
            }
        }
    } 

}

// chording action - both mouse buttons pressed on a revelealed tile whith a number equal to flagged adjacent cells
fn reveal_all_adjacent_tiles(arr: &mut [Tile], tile_id:usize, rows:u8, cols:u8) {
    // count flagged adjecent cells
    let mut flagged_cells = 0;

    for r in -1..2 {
        for c in -1..2{
            if r == 0 && c == 0 {
                continue;
            }
            else {
                let next_row:i32 = tile_id as i32 / cols as i32 + r;
                let next_col:i32 = tile_id as i32 % cols as i32 + c;
                if is_tile_in_grid(next_row, next_col, rows, cols)
                {
                    let next_id = (next_row * cols as i32 + next_col) as usize;
                    if arr[next_id].flagged { flagged_cells += 1; }
                }
            }
        }
        // check if number of flagged adjacent cells = number of adjecent mines
        if flagged_cells == arr[tile_id].adjacent_mines {
            for r in -1..2 {
                for c in -1..2{
                    if r == 0 && c == 0 {
                        continue;
                    }
                    else {
                        let next_row:i32 = tile_id as i32 / cols as i32 + r;
                        let next_col:i32 = tile_id as i32 % cols as i32 + c;
                        if is_tile_in_grid(next_row, next_col, rows, cols)
                        {
                            let next_id = (next_row * cols as i32 + next_col) as usize;
                            arr[next_id].adjacent_mines = num_adjacent_mines(&arr, rows, cols, next_id);
                            reveal_tile(arr, next_id, rows, cols);
                        }
                    }
                }
            }
        }
    } 
}

fn num_adjacent_mines(arr: &[Tile], grid_rows:u8, grid_cols:u8, tile_id:usize)->u8{
    let mut mines:u8 = 0;
    let row:i32 = tile_id as i32 / grid_cols as i32;
    let col:i32 = tile_id as i32 % grid_cols as i32;

    for r in -1..2 {
        for c in -1..2{
            if r == 0 && c == 0 {
                continue;
            }
            else {
                let next_row:i32 = row + r;
                let next_col:i32 = col + c;
                let next_id = (next_row * grid_cols as i32 + next_col) as usize;
    
                if is_tile_in_grid(next_row, next_col, grid_rows, grid_cols) {
                    if arr[next_id].has_mine {
                        mines += 1;
                    }
             
                }
            }
        }
    }
    return mines;
}

fn initialize_grid(arr: &mut[Tile], num_mines:u16, num_tiles:u16) {
    //srand(get_time() as u64);
    for i in 0..num_tiles {
        arr[i as usize] = Tile { revealed: false, has_mine: false, flagged: false, adjacent_mines:0 };
    }

    let mut placed_mines:u16 = 0;

    while placed_mines < num_mines {
        let n:usize = gen_range(0, num_tiles as usize);
        if arr[n].has_mine == true {
            continue;
        }
        else {
            arr[n].has_mine = true;
            placed_mines +=1;
        }
    }
}

fn update_game_state(arr: &[Tile], state: &mut GameState, num_mines:u16) {
    let mut revealed_tiles = 0;
    let mut flagged_mines = 0;

    for tile in arr {
        if *state == GameState::GameRunning {
            if tile.revealed && tile.has_mine {
                *state = GameState::GameLost;
                return;
            }
            if tile.revealed && !tile.has_mine {
                revealed_tiles += 1;
            }
            if tile.flagged && tile.has_mine {
                flagged_mines += 1;
            }
        }

    }

    if revealed_tiles == arr.len() - num_mines as usize {
        *state = GameState::GameWon;
    } else if flagged_mines == num_mines {
        *state = GameState::GameWon;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Rusty Mines".to_owned(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        high_dpi: true,
        window_resizable: false,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    srand(macroquad::miniquad::date::now() as u64);
    let assets:Assets = Default::default();
    let game:Game = Game::new(10, 10, 12);

    let mut state = GameState::MeinMenu;
    let mut grid:Vec<Tile> = vec![Tile { revealed: false, has_mine: false, flagged: false, adjacent_mines: 0 }; game.tiles as usize];

    loop {
        clear_background(BACKGROUND);
        let tile_size = calculate_tile_size(game.rows, game.columns, MAX_TILE_SIZE);

        match state {
            GameState::MeinMenu => {
                let screen_width = screen_width();
                let screen_height = screen_height();
                
                
                let text1 = "Rusty Mines";
                let font1_size = tile_size as u16;
                let text1_size = measure_text(&text1, Some(&assets.font),font1_size,1.0);
                let text1_x = screen_width / 2.0 - text1_size.width / 2.0;
                let text1_y = screen_height / 2.0 - text1_size.height / 2.0;

                draw_text_ex(
                    &text1,
                    text1_x,
                    text1_y,
                    TextParams {
                        font: Some(&assets.font),
                        font_size: font1_size as u16,
                        color: ORANGE,
                        ..Default::default()
                    }
                );

                let text2 = "Press ENTER to play...";
                let font2_size = (tile_size / 1.5) as u16;
                let text2_size = measure_text(&text2, Some(&assets.font),font2_size,1.0);
                let text2_x = screen_width / 2.0 - text2_size.width / 2.0;
                let text2_y = screen_height / 2.0 - text2_size.height / 2.0 + text1_size.height/2.0 + 20.0;

                draw_text_ex(
                    &text2,
                    text2_x,
                    text2_y,
                    TextParams {
                        font: Some(&assets.font),
                        font_size: font2_size as u16,
                        color: WHITE,
                        ..Default::default()
                    }
                );

                if is_key_pressed(KeyCode::Enter) {
                    initialize_grid(& mut grid, game.mines, game.tiles); 
                    state = GameState::GameRunning;
                }
            }

            GameState::GameRunning => {
                update_game_state(&grid, &mut state, game.mines);
        
                //calculate grid offsets to center the grid on the screen
                let (x_offset, y_offset) = calculate_grid_offsets(game.rows, game.columns, MAX_TILE_SIZE);

                draw_grid(&grid, &assets, &state, game.rows, game.columns, game.cell_size);
       
                if is_mouse_button_pressed(MouseButton::Right) {
                    let (mouse_x, mouse_y) = mouse_position();
                    let tile_id = screen_to_tile_id(mouse_x - x_offset, mouse_y - y_offset, game.columns as i32, game.rows as i32, tile_size);
                    if tile_id >= 0 {
                        flag_tile(& mut grid, tile_id as usize);
                    }
                }

                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mouse_x, mouse_y) = mouse_position();
                    let tile_id = screen_to_tile_id(mouse_x - x_offset, mouse_y - y_offset, game.columns as i32, game.rows as i32, tile_size);
                    if tile_id >= 0 {
                        grid[tile_id as usize].adjacent_mines = num_adjacent_mines(&grid, game.rows, game.columns, tile_id as usize);
                        reveal_tile(& mut grid, tile_id as usize, game.rows, game.columns);
                    }
                }

                if is_mouse_button_down(MouseButton::Left) && is_mouse_button_down(MouseButton::Right) {
                    let (mouse_x, mouse_y) = mouse_position();
                    let tile_id = screen_to_tile_id(mouse_x - x_offset, mouse_y - y_offset, game.columns as i32, game.rows as i32, tile_size);
                    if tile_id >= 0 {
                        reveal_all_adjacent_tiles(&mut grid, tile_id as usize, game.rows, game.columns);
                    }
                }            }

            GameState::GameLost => {
                draw_grid(&grid, &assets, &state, game.rows, game.columns, MAX_TILE_SIZE);

                let screen_width = screen_width();
                let screen_height = screen_height();

                let text_1 = "BOOM! You Lost...";
                let font1_size = (tile_size / 1.5) as u16;
                let text1_size = measure_text(&text_1, Some(&assets.font), font1_size, 1.0);
                let text1_x = screen_width / 2.0 - text1_size.width / 2.0;
                let text1_y = screen_height / 2.0 - text1_size.height / 2.0;

                let text2 = "Press ENTER to play again...";
                let font2_size = (tile_size / 2.0) as u16;
                let text2_size = measure_text(&text2, Some(&assets.font),font2_size,1.0);
                let text2_x = screen_width / 2.0 - text2_size.width / 2.0;
                let text2_y = screen_height / 2.0 - text2_size.height / 2.0 + text1_size.height/2.0 + 20.0;

                let margin = 5.0;
                let rectx = text1_x.min(text2_x) - margin;
                let recty= text1_y.min(text2_y) - text1_size.height - margin;
                let rectw = text1_size.width.max(text2_size.width) + margin + margin;
                let recth = text1_size.height + 20.0 + text2_size.height + margin;
                let rect_col:Color = Color::new(0.0, 0.0, 0.0, 0.5);
                draw_rectangle(rectx, recty, rectw, recth, rect_col);

                draw_text_ex(
                    &text_1,
                    text1_x,
                    text1_y,
                    TextParams {
                        font: Some(&assets.font),
                        font_size: font1_size as u16,
                        color: RED,
                        ..Default::default()
                    }
                );

                draw_text_ex(
                    &text2,
                    text2_x,
                    text2_y,
                    TextParams {
                        font: Some(&assets.font),
                        font_size: font2_size as u16,
                        color: WHITE,
                        ..Default::default()
                    }
                );
                if is_key_pressed(KeyCode::Enter) {
                    initialize_grid(& mut grid, game.mines, game.tiles); 
                    state = GameState::GameRunning;
                }
            }
            GameState::GameWon => {

                for tile in &mut grid {
                    tile.revealed = true;
                }
                draw_grid(&grid, &assets, &state, game.rows, game.columns, MAX_TILE_SIZE);

                let screen_width = screen_width();
                let screen_height = screen_height();

                let text1 = "You Won!";
                let font1_size = (tile_size / 1.5) as u16;
                let text1_size = measure_text(&text1, Some(&assets.font),font1_size,1.0);
                let text1_x = screen_width / 2.0 - text1_size.width / 2.0;
                let text1_y = screen_height / 2.0 - text1_size.height / 2.0;

                let text2 = "Press ENTER to play again...";
                let font2_size = (tile_size / 2.0) as u16;
                let text2_size = measure_text(&text2, Some(&assets.font),font2_size,1.0);
                let text2_x = screen_width / 2.0 - text2_size.width / 2.0;
                let text2_y = screen_height / 2.0 - text2_size.height / 2.0 + text1_size.height/2.0 + 20.0;

                let margin = 5.0;
                let rectx = text1_x.min(text2_x) - margin;
                let recty= text1_y.min(text2_y) - text1_size.height - margin;
                let rectw = text1_size.width.max(text2_size.width) + margin + margin;
                let recth = text1_size.height + 20.0 + text2_size.height + margin;
                let rect_col:Color = Color::new(0.0, 0.0, 0.0, 0.5);
                draw_rectangle(rectx, recty, rectw, recth, rect_col);
                
                draw_text_ex(
                    &text1,
                    text1_x,
                    text1_y,
                    TextParams {
                        font: Some(&assets.font),
                        font_size: font1_size as u16,
                        color: GREEN,
                        ..Default::default()
                    }
                );
                
                draw_text_ex(
                    &text2,
                    text2_x,
                    text2_y,
                    TextParams {
                        font: Some(&assets.font),
                        font_size: font2_size as u16,
                        color: WHITE,
                        ..Default::default()
                    }
                );

                if is_key_pressed(KeyCode::Enter) { 
                    initialize_grid(& mut grid, game.mines, game.tiles);
                    state = GameState::GameRunning;
                }
            }
        }
        next_frame().await;
    }
}
