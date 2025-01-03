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
// - Fix screen for different levels, adjust tile sizes
// - Add top menu and adjust grid positioning
// - Clean-up draw_grid function, match on game state
// - Move grid, tile, input handling etc code in own modules

const WINDOW_WIDTH:i32 = 800;
const WINDOW_HEIGHT:i32 = 600;
#[allow(dead_code)]
struct Game {
    level:u8,
    rows:u8,
    columns:u8,
    tiles:u16,
    cell_size:f32,
    mines:u16,
    mines_flagged:u16,
    level_time:u64,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            level: 1,
            rows: 8,
            columns: 8,
            tiles: 64,
            cell_size: (WINDOW_WIDTH as f32 / 8.0).min(WINDOW_HEIGHT as f32 / 8.0),
            mines: 10,
            mines_flagged: 0,
            level_time: 0,
        }
    }
}

const ROWS:u8 = 8;
const COLS:u8 = 8;
const GRID_SIZE:u16 = ROWS as u16 * COLS as u16;
const CELL_SIZE:f32 = 60.0;
const MINES:u16 = 10;

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
    fn draw(&self, rect:Rect, x:f32, y:f32){
        let size = CELL_SIZE / 1.5;
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

fn draw_grid(arr: &[Tile], assets:&Assets, state:&GameState) {
    for i in 0..arr.len() {
        let x:f32 = (i as u8 % COLS) as f32 * CELL_SIZE;
        let y:f32 = (i as u8 / COLS) as f32 * CELL_SIZE;

        // let font_size = CELL_SIZE / 1.5;
        // let text_size = measure_text("0", Some(&assets.font), font_size as u16, 1.0);
        // let text_x = x + CELL_SIZE / 2.0 - text_size.width / 2.0;
        // let text_y: f32 = y + CELL_SIZE / 2.0 + text_size.height / 2.0;

        if arr[i].revealed {
            draw_rectangle(x, y, CELL_SIZE, CELL_SIZE, GRAY);
            if arr[i].has_mine && (state == &GameState::GameRunning || state == &GameState::GameLost){
                assets.draw(assets.explosion, x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0);
            }

            else if arr[i].has_mine && state == &GameState::GameWon {
                draw_rectangle(x, y, CELL_SIZE, CELL_SIZE, LIGHTGRAY);
                assets.draw(assets.bomb, x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0);
            }
            else {

                match arr[i].adjacent_mines {
                    1 => assets.draw(assets.one, x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0),
                    2 => assets.draw(assets.two, x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0),
                    3 => assets.draw(assets.three, x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0),
                    4 => assets.draw(assets.four, x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0),
                    5 => assets.draw(assets.five, x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0),
                    6 => assets.draw(assets.six, x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0),
                    7 => assets.draw(assets.seven, x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0),
                    8 => assets.draw(assets.eight, x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0),
                    _ => (),
                }

                // using text rather than images
                // if arr[i].adjacent_mines > 0 {
                //     draw_text_ex(
                //         &arr[i].adjacent_mines.to_string(),
                //         text_x,
                //         text_y,
                //         TextParams {
                //             font:Some(&assets.font),
                //             font_size: font_size as u16,
                //             color: BLACK,
                //             ..Default::default()
                //         }
                //     );
                // }
            }
        }
        else {
            draw_rectangle(x, y, CELL_SIZE, CELL_SIZE, LIGHTGRAY);
        }
        if arr[i].flagged {
            if state == &GameState::GameRunning {
                assets.draw(assets.flag, x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0);
            }
            else if state == &GameState::GameLost || state == &GameState::GameWon {
                if arr[i].has_mine {
                    draw_rectangle(x, y, CELL_SIZE, CELL_SIZE, LIGHTGRAY);
                    assets.draw(assets.bomb, x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0);
                }
                else {
                    let offset = CELL_SIZE / 3.0;
                    assets.draw(assets.flag, x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0);
                    draw_line(x + offset, y + offset , x + CELL_SIZE - offset, y + CELL_SIZE - offset, 6.0, BLACK);
                    draw_line(x + CELL_SIZE - offset, y + offset , x + offset, y + CELL_SIZE - offset, 6.0, BLACK);
                }
            }
        }
        draw_rectangle_lines(x, y, CELL_SIZE, CELL_SIZE, 1.0,DARKGRAY);
    }
}

fn flag_tile(arr: &mut [Tile], tile_id:usize){
    if !arr[tile_id].revealed {
        arr[tile_id].flagged = !arr[tile_id].flagged;
    }
}

fn reveal_tile(arr: &mut [Tile], tile_id:usize){
    if !arr[tile_id].flagged {
        arr[tile_id].revealed = true;
        if arr[tile_id].adjacent_mines == 0 {
            reveal_adjacent_tiles(arr, tile_id)
        }
    }
}

fn reveal_adjacent_tiles(arr:&mut [Tile], tile_id:usize) {
    if arr[tile_id].adjacent_mines == 0 && !arr[tile_id].has_mine {
        for r in -1..2 {
            for c in -1..2{
                if r == 0 && c == 0 {
                    continue;
                }
                else {
                    let next_row:i32 = tile_id as i32 / COLS as i32 + r;
                    let next_col:i32 = tile_id as i32 % COLS as i32 + c;
                    if is_tile_in_grid(next_row, next_col, ROWS, COLS)
                    {
                        let next_id = (next_row * COLS as i32 + next_col) as usize;
                        if arr[next_id].revealed ==false {
                            arr[next_id].adjacent_mines = num_adjacent_mines(&arr, ROWS, COLS, next_id);
                            if !arr[next_id].has_mine {
                                reveal_tile(arr, next_id);
                            }
                        }
                    }
                }
            }
        }
    } 

}

// chording action - both mouse buttons pressed on a revelealed tile whith a number equal to flagged adjacent cells
fn reveal_all_adjacent_tiles(arr: &mut [Tile], tile_id:usize) {
    // count flagged adjecent cells
    let mut flagged_cells = 0;

    for r in -1..2 {
        for c in -1..2{
            if r == 0 && c == 0 {
                continue;
            }
            else {
                let next_row:i32 = tile_id as i32 / COLS as i32 + r;
                let next_col:i32 = tile_id as i32 % COLS as i32 + c;
                if is_tile_in_grid(next_row, next_col, ROWS, COLS)
                {
                    let next_id = (next_row * COLS as i32 + next_col) as usize;
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
                        let next_row:i32 = tile_id as i32 / COLS as i32 + r;
                        let next_col:i32 = tile_id as i32 % COLS as i32 + c;
                        if is_tile_in_grid(next_row, next_col, ROWS, COLS)
                        {
                            let next_id = (next_row * COLS as i32 + next_col) as usize;
                            arr[next_id].adjacent_mines = num_adjacent_mines(&arr, ROWS, COLS, next_id);
                            reveal_tile(arr, next_id);
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

fn initialize_grid(arr: &mut[Tile]) {
    srand(get_time() as u64);
    for i in 0..GRID_SIZE {
        arr[i as usize] = Tile { revealed: false, has_mine: false, flagged: false, adjacent_mines:0 };
    }

    let mut placed_mines = 0;

    while placed_mines < MINES {
        let n:usize = gen_range(0, GRID_SIZE as usize);
        if arr[n].has_mine == true {
            continue;
        }
        else {
            arr[n].has_mine = true;
            placed_mines +=1;
        }
    }
}

fn update_game_state(arr: &[Tile], state: &mut GameState) {
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

    if revealed_tiles == (GRID_SIZE - MINES) as usize {
        *state = GameState::GameWon;
    } else if flagged_mines == MINES {
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
    let assets:Assets = Default::default();
    let game:Game = Default::default();

    let mut state = GameState::MeinMenu;
    let mut grid:[Tile; GRID_SIZE as usize] = [Tile { revealed: false, has_mine: false, flagged: false, adjacent_mines: 0 }; GRID_SIZE as usize];

    loop {
        clear_background(DARKBROWN);
        update_game_state(&grid, &mut state);

        match state {
            GameState::MeinMenu => {
                let screen_width = screen_width();
                let screen_height = screen_height();
                
                
                let text1 = "Rusty Mines";
                let font1_size = CELL_SIZE as u16;
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
                let font2_size = (CELL_SIZE / 2.0) as u16;
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
                    initialize_grid(& mut grid); 
                    state = GameState::GameRunning;
                }
            }

            GameState::GameRunning => {
                draw_grid(&grid, &assets, &state);
       
                if is_mouse_button_pressed(MouseButton::Right) {
                    let (mouse_x, mouse_y) = mouse_position();
                    let tile_id = screen_to_tile_id(mouse_x, mouse_y, COLS as i32, ROWS as i32, CELL_SIZE);
                    if tile_id >= 0 {
                        flag_tile(& mut grid, tile_id as usize);
                    }
                }

                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mouse_x, mouse_y) = mouse_position();
                    let tile_id = screen_to_tile_id(mouse_x, mouse_y, COLS as i32 , ROWS as i32, CELL_SIZE);
                    if tile_id >= 0 {
                        grid[tile_id as usize].adjacent_mines = num_adjacent_mines(&grid,ROWS, COLS, tile_id as usize);
                        reveal_tile(& mut grid, tile_id as usize);
                    }
                }

                if is_mouse_button_down(MouseButton::Left) && is_mouse_button_down(MouseButton::Right) {
                    let (mouse_x, mouse_y) = mouse_position();
                    let tile_id = screen_to_tile_id(mouse_x, mouse_y, COLS as i32 , ROWS as i32, CELL_SIZE);
                    if tile_id >= 0 {
                        reveal_all_adjacent_tiles(&mut grid, tile_id as usize);
                    }
                }
            }

            GameState::GameLost => {
                draw_grid(&grid, &assets, &state);

                let screen_width = screen_width();
                let screen_height = screen_height();

                let text_1 = "BOOM! You Lost...";
                let font1_size = (CELL_SIZE / 1.5) as u16;
                let text1_size = measure_text(&text_1, Some(&assets.font), font1_size, 1.0);
                let text1_x = screen_width / 2.0 - text1_size.width / 2.0;
                let text1_y = screen_height / 2.0 - text1_size.height / 2.0;

                let text2 = "Press ENTER to play again...";
                let font2_size = (CELL_SIZE / 2.0) as u16;
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
                    initialize_grid(& mut grid); 
                    state = GameState::GameRunning;
                }
            }
            GameState::GameWon => {

                for tile in &mut grid {
                    tile.revealed = true;
                }
                draw_grid(&grid, &assets, &state);

                let screen_width = screen_width();
                let screen_height = screen_height();

                let text1 = "You Won!";
                let font1_size = (CELL_SIZE / 1.5) as u16;
                let text1_size = measure_text(&text1, Some(&assets.font),font1_size,1.0);
                let text1_x = screen_width / 2.0 - text1_size.width / 2.0;
                let text1_y = screen_height / 2.0 - text1_size.height / 2.0;

                let text2 = "Press ENTER to play again...";
                let font2_size = (CELL_SIZE / 2.0) as u16;
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
                    initialize_grid(& mut grid);
                    state = GameState::GameRunning;
                }
            }
        }
        next_frame().await;
    }
}
