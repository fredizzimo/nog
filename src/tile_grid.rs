use crate::CONFIG;
use crate::tile::Tile;
use crate::window::Window;
use winapi::um::winuser::SetForegroundWindow;
use winapi::um::winuser::GetWindowRect;
use winapi::um::winuser::FindWindowA;
use winapi::um::winuser::ShowWindow;
use winapi::um::winuser::SW_HIDE;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::um::winuser::GetSystemMetrics;
use winapi::um::winuser::SM_CXSCREEN;
use winapi::um::winuser::SM_CYSCREEN;
use winapi::um::winuser::SetWindowPos;

#[derive(Clone, EnumString)]
pub enum SplitDirection {
    Horizontal,
    Vertical
}
//TODO(#20)
pub struct TileGrid {
    pub tiles: Vec<Tile>,
    pub focused_window_id: Option<i32>,
    pub taskbar_window: i32,
    pub rows: i32,
    pub columns: i32,
    pub height: i32,
    pub width: i32
}

impl TileGrid {
    pub fn new() -> Self {
        Self {
            tiles: Vec::<Tile>::new(),
            focused_window_id: None,
            taskbar_window: 0,
            rows: 0,
            columns: 0,
            height: 0,
            width: 0
        }
    }
    pub fn get_focused_tile(&self) -> Option<&Tile> {
        return self.focused_window_id
            .and_then(|id| self.tiles
                .iter()
                .find(|tile| tile.window.id == id));
    }
    pub fn get_focused_tile_mut(&mut self) -> Option<&mut Tile> {
        return self.focused_window_id
            .and_then(move |id| self.tiles
                .iter_mut()
                .find(|tile| tile.window.id == id));
    }
    pub fn set_focused_split_direction(&mut self, direction: SplitDirection) {
        if let Some(focused_tile) = self.get_focused_tile_mut() {
            focused_tile.split_direction = direction;
        }
    }
    pub unsafe fn focus_right(&mut self){
        if let Some(focused_tile) = self.get_focused_tile() {
            if focused_tile.column == Some(self.columns) || focused_tile.column == None {
                return;
            }

            let maybe_next_tile = self.tiles
                .iter()
                .find(|tile| (tile.row == None || tile.row == focused_tile.row) && tile.column == focused_tile.column.map(|x| x + 1));

            if let Some(next_tile) = maybe_next_tile {
                self.focused_window_id = Some(next_tile.window.id);
                SetForegroundWindow(next_tile.window.id as HWND);
            }
        }
    }
    pub unsafe fn focus_left(&mut self){
        if let Some(focused_tile) = self.get_focused_tile() {
            if focused_tile.column == Some(1) || focused_tile.column == None {
                return;
            }

            let maybe_next_tile = self.tiles
                .iter()
                .find(|tile| (tile.row == None || tile.row == focused_tile.row) && tile.column == focused_tile.column.map(|x| x - 1));

            if let Some(next_tile) = maybe_next_tile {
                self.focused_window_id = Some(next_tile.window.id);
                SetForegroundWindow(next_tile.window.id as HWND);
            }
        }
    }
    pub unsafe fn focus_up(&mut self){
        if let Some(focused_tile) = self.get_focused_tile() {
            if focused_tile.row == Some(1) || focused_tile.row == None {
                return;
            }

            let maybe_next_tile = self.tiles
                .iter()
                .find(|tile| (tile.column == None || tile.column == focused_tile.column) && tile.row == focused_tile.row.map(|x| x - 1));

            if let Some(next_tile) = maybe_next_tile {
                self.focused_window_id = Some(next_tile.window.id);
                SetForegroundWindow(next_tile.window.id as HWND);
            }
        }
    }
    pub unsafe fn focus_down(&mut self){
        if let Some(focused_tile) = self.get_focused_tile() {
            if focused_tile.row == Some(self.rows) || focused_tile.row == None {
                return;
            }

            let maybe_next_tile = self.tiles
                .iter()
                .find(|tile| (tile.column == None || tile.column == focused_tile.column) && tile.row == focused_tile.row.map(|x| x + 1));

            if let Some(next_tile) = maybe_next_tile {
                self.focused_window_id = Some(next_tile.window.id);
                SetForegroundWindow(next_tile.window.id as HWND);
            }
        }
    }
    pub unsafe fn fetch_resolution(&mut self) {
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0
        };

        self.height = GetSystemMetrics(SM_CYSCREEN) - 20;
        self.width = GetSystemMetrics(SM_CXSCREEN);

        if !CONFIG.remove_title_bar {
            self.height = self.height + 9;
            self.width = self.width + 15;
        } 

        self.taskbar_window = FindWindowA("Shell_TrayWnd".as_ptr() as *const i8, std::ptr::null()) as i32;
        let taskbar_is_visible = rect.top + 2 < self.height;

        GetWindowRect(self.taskbar_window as HWND, &mut rect);

        if taskbar_is_visible {
            if CONFIG.remove_task_bar {
                ShowWindow(self.taskbar_window as HWND, SW_HIDE);
            }
            else {
                self.height = self.height - (rect.bottom - rect.top);
            }
        }
    }
    pub fn close_tile_by_window_id(&mut self, id: i32) -> Option<Tile> {
        let maybe_removed_tile = self.tiles
            .iter()
            .position(|tile| tile.window.id == id)
            .map(|idx| self.tiles.remove(idx));

        if let Some(removed_tile) = maybe_removed_tile.clone() {
            let is_empty_row = !self.tiles
                .iter()
                .any(|tile| tile.row == removed_tile.row);

            let is_empty_column = !self.tiles
                .iter()
                .any(|tile| tile.column == removed_tile.column);

            println!("Row is empty = {} | Column is empty = {}", is_empty_row, is_empty_column);

            if is_empty_row {
                self.rows = self.rows - 1;
                let tiles_after_deleted_tile = self.tiles
                    .iter_mut()
                    .filter(|t| t.row > removed_tile.row);

                for tile in tiles_after_deleted_tile {
                    tile.row = tile.row.map(|x| x - 1);
                }
            }

            if is_empty_column {
                self.columns = self.columns - 1;
                let tiles_after_deleted_tile = self.tiles
                    .iter_mut()
                    .filter(|t| t.column > removed_tile.column);

                for tile in tiles_after_deleted_tile {
                    tile.column = tile.column.map(|x| x - 1);
                }
            }

            if self.tiles.len() == 0 {
                self.focused_window_id = None;
            }
            else if let Some(focused_window_id) = self.focused_window_id {
                if focused_window_id == removed_tile.window.id {
                    let next_column = removed_tile.column.map(|column| {
                        return if column > self.columns {
                            column - 1
                        } else {
                            column
                        }
                    });

                    let next_row = removed_tile.row.map(|row| {
                        return if row > self.rows {
                            row - 1
                        } else {
                            row
                        }
                    });

                    let maybe_next_tile: Option<&Tile> = self.tiles
                        .iter()
                        .find(|tile| {
                            return tile.column == next_column && tile.row == next_row;
                        });

                    if let Some(next_tile) = maybe_next_tile {
                        self.focused_window_id = Some(next_tile.window.id);
                    }
                }
            }
        }

        return maybe_removed_tile;
    }
    pub fn split(&mut self, window: Window){
        if self.tiles.iter().any(|t| t.window.id == window.id) {
            return;
        }

        match self.get_focused_tile_mut() {
            Some(focused_tile) => {
                let mut new_tile = Tile::new(window);

                match focused_tile.split_direction {
                    SplitDirection::Horizontal => {
                        new_tile.column = focused_tile.column;
                        match focused_tile.row {
                            Some(row) => new_tile.row = Some(row + 1),
                            None => {
                                focused_tile.row = Some(1);
                                new_tile.row = Some(2);
                            }
                        }
                        self.rows = self.rows + 1;
                    },
                    SplitDirection::Vertical => {
                        new_tile.row = focused_tile.row;
                        match focused_tile.column {
                            Some(column) => new_tile.column = Some(column + 1),
                            None => {
                                focused_tile.column = Some(1);
                                new_tile.column = Some(2);
                            }
                        }
                        self.columns = self.columns + 1;
                    }
                }

                self.focused_window_id = Some(new_tile.window.id);
                self.tiles.push(new_tile);
            },
            None => {
                self.rows = 1;
                self.columns = 1;
                self.focused_window_id = Some(window.id);
                self.tiles.push(Tile::new(window));
            } 
        }
    }
    unsafe fn draw_tile_with_title_bar(&self, tile: &Tile) {
        let column_width = self.width / self.columns;
        let row_height = self.height / self.rows;

        let column_delta = match tile.column {
            Some(column) => if column > 1 {
                15
            } else {
                0
            },
            None => 0
        };

        let row_delta = match tile.row {
            Some(row) => if row > 1 {
                10
            } else {
                0
            },
            None => 0
        };

        let x = match tile.column {
            Some(column) => column_width * (column - 1) - 8 - column_delta,
            None => -8
        };

        let y = match tile.row {
            Some(row) => row_height * (row - 1) - row_delta - 1,
            None => -1
        };

        let height = match tile.row {
            Some(_row) => row_height + row_delta,
            None => self.height
        };

        let width = match tile.column {
            Some(_column) => column_width + column_delta,
            None => self.width
        };

        SetWindowPos(tile.window.id as HWND, std::ptr::null_mut(), x, y + 20, width, height, 0);
    }

    unsafe fn draw_tile(&self, tile: &Tile){
        let column_width = self.width / self.columns;
        let row_height = self.height / self.rows;

        let x = match tile.column {
            Some(column) => column_width * (column - 1),
            None => 0
        };

        let y = match tile.row {
            Some(row) => row_height * (row - 1),
            None => 0
        };

        let height = match tile.row {
            Some(_row) => row_height,
            None => self.height
        };  

        let width = match tile.column {
            Some(_column) => column_width,
            None => self.width
        };

        SetWindowPos(tile.window.id as HWND, std::ptr::null_mut(), x, y + 20, width, height, 0);
    }

    pub fn print_grid(&self) -> () {
        println!("rows: {} | columns: {}", self.rows, self.columns);

        if self.rows == 0 || self.columns == 0 {
            return;
        }

        let mut rows = [[std::ptr::null(); 10]; 10];

        for tile in &self.tiles {
            match tile.row {
                Some(row) => match tile.column {
                    Some(column) => rows[(row - 1) as usize][(column - 1) as usize] = &tile.window,
                    None => for i in 0..self.columns {
                        rows[(row - 1) as usize][i as usize] = &tile.window;
                    }
                },
                None => match tile.column {
                    Some(column) => for i in 0..self.rows {
                        rows[i as usize][(column - 1) as usize] = &tile.window;
                    }
                    None => rows[0][0] = &tile.window
                }
            }
            println!("{} {}", tile.column.unwrap_or(0), tile.row.unwrap_or(0));
            unsafe {
                if CONFIG.remove_title_bar {
                    self.draw_tile(tile);
                } else {
                    self.draw_tile_with_title_bar(tile);
                }
            }
        }
        
        for row in &rows {
            print!("|");
            for column in row {
                print!(" {} |", (*column) as usize);
            }
            print!("\n");
        }

        println!();

        for row in 0..self.rows {
            print!("|");
            for column in 0..self.columns {
                unsafe {
                    let window = &(*rows[row as usize][column as usize]);
                    if let Some(id) = self.focused_window_id {
                        match window.id == id {
                            true => print!("* {}({}) *|", window.name, window.id),
                            false => print!(" {}({}) |", window.name, window.id)
                        }
                    }
                }
            }
            print!("\n");
        }
    }
}