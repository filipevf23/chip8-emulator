use std::fs;
use std::io;
use macroquad::prelude::*;
use macroquad::audio::{load_sound, play_sound, PlaySoundParams};

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const SCALE: f32 = 15.0;
const MEMORY_AMOUNT: usize = 4096;

fn window_conf() -> Conf {
    Conf {
        window_title: "Chip-8 Emulator".to_owned(),
        window_width: (SCREEN_WIDTH as i32 * SCALE as i32),
        window_height: (SCREEN_HEIGHT as i32 * SCALE as i32),
        window_resizable: false,
        ..Default::default()
    }
}

struct Chip8 {
    memory : [u8; MEMORY_AMOUNT],
    stack : Vec<u16>,
    v : [u8; 16],
    i : u16,
    delay_timer: u8,
    sound_timer: u8,
    pc : u16,
    display : [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
}

#[macroquad::main(window_conf)]
async fn main() {
    
    let mut chip = Chip8::new();
    chip.load_font();

    let mut is_file_loaded: bool = false;
    let blip_sound = load_sound("blip.wav").await.unwrap();

    loop {

        if !is_file_loaded {

            let (mouse_x, mouse_y) = mouse_position();
            if mouse_x < 20f32 || mouse_x > 250f32 || mouse_y < 20f32 || mouse_y > 70f32 {
                draw_rectangle(20 as f32, 20 as f32, 230 as f32, 50 as f32, RED);
            } else {
                draw_rectangle(20 as f32, 20 as f32, 230 as f32, 50 as f32, GREEN);

                if is_mouse_button_pressed(MouseButton::Left) {
                    let file = rfd::FileDialog::new()
                        .set_directory("/GAMES/")
                        .pick_file();
                    if let Some(path) = file {
                        if let Ok(content) = fs::read(path) {
                            for i in 0..content.len() {
                                chip.memory[512 + i] = content[i];
                            }
                            is_file_loaded = true;
                        }
                    }
                }
            }

            draw_text("Escolha o Jogo", 30 as f32, 50 as f32, 32 as f32, BLUE);
            next_frame().await;
            continue;

        }
        
        for _ in 0..10 {
            if let Ok(opcode) = chip.fetch() {
                chip.execute(opcode);
            };
        }

        for p in 0..SCREEN_HEIGHT {
            for q in 0..SCREEN_WIDTH {
                let ind = q + (p*SCREEN_WIDTH);

                if chip.display[ind] != 0 {
                    draw_rectangle(q as f32 *SCALE,
                                   p as f32 *SCALE,
                                   SCALE,
                                   SCALE,
                                   RED);
                }
            }
        }

        if chip.delay_timer > 0 {
            chip.delay_timer -= 1;
        }
        if chip.sound_timer > 0 {
            chip.sound_timer -= 1;
            if chip.sound_timer == 0 {
                play_sound(&blip_sound, PlaySoundParams{looped: false, volume: 0.5},);
            }
        }

        next_frame().await;
    }
}

impl Chip8 {
    fn new() -> Self {
        Self {
            memory: [0; MEMORY_AMOUNT],
            display: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            v: [0; 16],
            stack: Vec::<u16>::new(),
            pc: 512, 
            i: 0,
            delay_timer: 0xFF,
            sound_timer: 0xFF,
        }
    }

    fn load_font(&mut self) {
        let font = [0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
                    0x20, 0x60, 0x20, 0x20, 0x70, // 1
                    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
                    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
                    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
                    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
                    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
                    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
                    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
                    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
                    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
                    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
                    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
                    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
                    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
                    0xF0, 0x80, 0xF0, 0x80, 0x80];// F
        for j in 0..font.len() {
            self.memory[(0x50 + j) as usize] = font[j];
        }
    }

    fn fetch(&mut self) -> io::Result<u16> {
        let high_byte = self.memory[self.pc as usize] as u16;
        let low_byte = self.memory[(self.pc+1) as usize] as u16;
        self.pc += 2;
        let data = (high_byte << 8) | low_byte;

        Ok(data)
    }

    fn execute(&mut self, opcode: u16) {
        let x = ((opcode >> 8) & 0x000F) as usize;
        let y = ((opcode >> 4) & 0x000F) as usize;
        let n = (opcode & 0x000F) as u8;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = (opcode & 0x0FFF) as u16;

        match ((opcode >> 12) & 0x000F) as u32 {
            0 => {
                if nn == 0xE0 {
                    self.op_clear_screen();
                } else if nn == 0xEE {
                    self.op_return_subroutine();
                }
            },
            1 => {
                self.op_jump(nnn);
            },
            2 => {
                self.op_subroutine(nnn);
            },
            3 => {
                self.op_skip_iie(x, nn);
            },
            4 => {
                self.op_skip_ine(x, nn);
            },
            5 => {
                self.op_skip_ie(x, y);
            },
            6 => {
                self.op_set_ivx(x, nn);
            },
            7 => {
                self.op_add_ivx(x, nn);
            },
            8 => {
                if n == 0 {
                    self.op_set_vx(x, y);
                } else if n == 1 {
                    self.op_or_vx(x, y);
                } else if n == 2 {
                    self.op_and_vx(x, y);
                } else if n == 3 {
                    self.op_xor_vx(x, y);
                } else if n == 4 {
                    self.op_add_vx(x, y);
                } else if n == 5 {
                    self.op_subtract_vx(x, y);
                } else if n == 6 {
                    self.op_shift_r(x, y);
                } else if n == 7 {
                    self.op_subtract_vxi(x, y);
                } else if n == 0xE {
                    self.op_shift_l(x, y);
                }
            },
            9 => {
                self.op_skip_ne(x, y);
            },
            0xA => {
                self.op_set_i(nnn);
            },
            0xB => {
                self.op_jump_offset(nnn);
            },
            0xC => {
                self.op_random(x, nn);
            },
            0xD => {
                self.op_draw(x, y, n);
            },
            0xE => {
                if nn == 0x9E {
                    self.op_skip_ik(x);
                } else if nn == 0xA1 {
                    self.op_skip_ink(x);
                }
            },
            0xF => {
                if nn == 0x07 {
                    self.op_set_dt(x);
                } else if nn == 0x15 {
                    self.op_set_delay(x);
                } else if nn == 0x18 {
                    self.op_set_sound(x);
                } else if nn == 0x1E {
                    self.op_add_i(x);
                } else if nn == 0x0A {
                    self.op_get_k(x);
                } else if nn == 0x29 {
                    self.op_get_font_char(x);
                } else if nn == 0x33 {
                    self.op_bin_3decimal(x);
                } else if nn == 0x55 {
                    self.op_store_v(x);
                } else if nn == 0x65 {
                    self.op_load_v(x);
                }
            },
            _ => println!("Uknown Opcode."),
        }
    }   

    fn op_jump(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn op_jump_offset(&mut self, nnn: u16) {
        self.pc = nnn + self.v[0] as u16;
    }

    fn op_store_v(&mut self, x: usize) {
        for j in 0..=x {
            self.memory[self.i as usize + j] = self.v[j];
        }
    } 

    fn op_load_v(&mut self, x: usize) {
        for j in 0..=x {
            self.v[j] = self.memory[self.i as usize + j];
        }
    }

    fn op_random(&mut self, x: usize, nn: u8) {
        self.v[x] = (macroquad::rand::gen_range(0, 256) as u8) & nn;
    }

    fn op_get_font_char(&mut self, x: usize) {
        // font starts at 0x50, 5 bytes per char
        self.i = (0x50 + (self.v[x] & 0xF) * 5) as u16;
    }

    fn op_bin_3decimal(&mut self, x: usize) {
        let value = self.v[x];
        let i = self.i as usize;
        self.memory[i] = value / 100;         // bigger  (max is 255)
        self.memory[i + 1] = (value / 10) % 10; // middle
        self.memory[i + 2] = value % 10 ;       // unit
    }

    fn op_set_dt(&mut self, x: usize) {
        self.v[x] = self.delay_timer;
    }

    fn op_set_delay(&mut self, x: usize) {
        self.delay_timer = self.v[x];
    }

    fn op_set_sound(&mut self, x: usize) {
        self.sound_timer = self.v[x];
    }

    fn op_return_subroutine(&mut self) {
        self.pc = self.stack.pop().expect("REASON");
    }

    fn op_subroutine(&mut self, nnn: u16) {
        self.stack.push(self.pc);
        self.pc = nnn;
    }

    fn op_get_k(&mut self, x: usize) {
        // checking every possible key
        for j in 0..16 {
            if check_key(j as u8) {
                self.v[x] = j as u8;
                return;
            }
        }
        self.pc -= 2;
    }

    fn op_skip_ik(&mut self, x: usize) {
        if check_key(self.v[x]) {
            self.pc += 2;
        }
    }

    fn op_skip_ink(&mut self, x: usize) {
        if !check_key(self.v[x]) {
            self.pc += 2;
        }
    }

    fn op_skip_iie(&mut self, x: usize, nn: u8) {
        if self.v[x] == nn {
            self.pc += 2;
        }
    }

    fn op_skip_ine(&mut self, x: usize, nn: u8) {
        if self.v[x] != nn {
            self.pc += 2;
        }
    }

    fn op_skip_ie(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    fn op_skip_ne(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    fn op_clear_screen(&mut self) {
        self.display = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    fn op_set_ivx(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
    }

    fn op_add_ivx(&mut self, x: usize, nn: u8) {
        self.v[x] = ((self.v[x] as u16 + nn as u16) & 0x00FF) as u8;
    }

    fn op_set_vx(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
    }

    fn op_or_vx(&mut self, x: usize, y:usize) {
        self.v[x] = self.v[x] | self.v[y];
    }

    fn op_and_vx(&mut self, x: usize, y:usize) {
        self.v[x] = self.v[x] & self.v[y];
    }

    fn op_xor_vx(&mut self, x: usize, y:usize) {
        self.v[x] = self.v[x] ^ self.v[y];
    }

    fn op_add_vx(&mut self, x: usize, y:usize) {
        let sum: u16 = self.v[x] as u16 + self.v[y] as u16;
        self.v[x] = ((self.v[x] as u16 + self.v[y] as u16) & 0x00FF) as u8;
        if sum > 0x00FF {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
    }

    fn op_subtract_vx(&mut self, x: usize, y:usize) {
        if self.v[x] >= self.v[y] {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
    }

    fn op_subtract_vxi(&mut self, x: usize, y:usize) {
        if self.v[y] >= self.v[x] {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
    }

    fn op_shift_l(&mut self, x: usize, y: usize) {
        self.v[0xf] = ((self.v[y] >> 7) & 1) as u8;
        self.v[x] = (((self.v[y] as u16) << 1) & 0xFF) as u8;
    }

    fn op_shift_r(&mut self, x: usize, y: usize) {
        self.v[0xf] = (self.v[y] & 1) as u8;
        self.v[x] = (self.v[y] >> 1) as u8;
    }

    fn op_set_i(&mut self, nnn: u16) {
        self.i = nnn;
        return;
    }

    fn op_add_i(&mut self, x: usize) {
        self.i += self.v[x] as u16;
        if self.i > 0x0FFF {
            self.v[0xF] = 1;
        }
        self.i = self.i & 0x0FFF;
    }

    fn op_draw(&mut self, x: usize, y: usize, n: u8) {
        let x_coord = (self.v[x] & 63) as usize; // to clip out-of-screen-sprite
        let y_coord = (self.v[y] & 31) as usize;
        self.v[0xF] = 0u8;
        for j in 0..n {
            if  j > 31 {
                break;
            }

            let byte = self.memory[(self.i + j as u16) as usize];
            for k in 0..8 {

                let bit = (byte >> (7-k)) & 1;
                if bit != 0 {
                    let x_final = (x_coord + k as usize) & 63;
                    let y_final = (y_coord + j as usize) & 31;
                    let index = x_final + (y_final * SCREEN_WIDTH);
                    if self.display[index] == 1 {
                        self.display[index] = 0;
                        self.v[0xF] = 1;
                    } else {
                        self.display[index] = 1;
                    }
                }
            }
        }
        return;
    }
}

fn check_key(key: u8) -> bool {
    match key {
        0 => {
            return is_key_down(KeyCode::X);
        },
        1 => {
            return is_key_down(KeyCode::Key1);
        },
        2 => {
            return is_key_down(KeyCode::Key2);
        },
        3 => {
            return is_key_down(KeyCode::Key3);
        },
        4 => {
            return is_key_down(KeyCode::Q);
        },
        5 => {
            return is_key_down(KeyCode::W);
        },
        6 => {
            return is_key_down(KeyCode::E);
        },
        7 => {
            return is_key_down(KeyCode::A);
        },
        8 => {
            return is_key_down(KeyCode::S);
        },
        9 => {
            return is_key_down(KeyCode::D);
        },
        0xA => {
            return is_key_down(KeyCode::Z);
        },
        0xB => {
            return is_key_down(KeyCode::C);
        },
        0xC => {
            return is_key_down(KeyCode::Key4);
        },
        0xD => {
            return is_key_down(KeyCode::R);
        },
        0xE => {
            return is_key_down(KeyCode::F);
        },
        0xF => {
            return is_key_down(KeyCode::V);
        },
        _ => {
            println!("Unknown Key {}", key);
        },
    }
    return false;
}
