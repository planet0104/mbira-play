use core::str::FromStr;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::gpio::{gpiob, gpioa, gpioc, Output, PushPull};
use embedded_hal::digital::v2::OutputPin;
// use cortex_m_semihosting::hprintln; //半主机功能
use crate::alloc::{vec::Vec, string::{String, ToString}};
use stm32f1xx_hal::delay::Delay;

pub const NOTE_D6:char = 'D';
pub const NOTE_B5:char = 'b';
pub const NOTE_G5:char = 'g';
pub const NOTE_E5:char = 'e';
pub const NOTE_C5:char = 'c';
pub const NOTE_A4:char = '6';
pub const NOTE_F4:char = '4';
pub const NOTE_D4:char = '2';
pub const NOTE_C4:char = '1';
pub const NOTE_E4:char = '3';
pub const NOTE_G4:char = '5';
pub const NOTE_B4:char = '7';
pub const NOTE_D5:char = 'd';
pub const NOTE_F5:char = 'f';
pub const NOTE_A5:char = 'a';
pub const NOTE_C6:char = 'C';
pub const NOTE_E6:char = 'E';

pub const NOTE_D6_IO:u8 = 0;
pub const NOTE_B5_IO:u8 = 1;
pub const NOTE_G5_IO:u8 = 2;
pub const NOTE_E5_IO:u8 = 3;
pub const NOTE_C5_IO:u8 = 4;
pub const NOTE_A4_IO:u8 = 5;
pub const NOTE_F4_IO:u8 = 6;
pub const NOTE_D4_IO:u8 = 7;
pub const NOTE_C4_IO:u8 = 8;
pub const NOTE_E4_IO:u8 = 9;
pub const NOTE_G4_IO:u8 = 10;
pub const NOTE_B4_IO:u8 = 11;
pub const NOTE_D5_IO:u8 = 12;
pub const NOTE_F5_IO:u8 = 13;
pub const NOTE_A5_IO:u8 = 14;
pub const NOTE_C6_IO:u8 = 15;
pub const NOTE_E6_IO:u8 = 16;
pub const NOTE_NO_IO:u8 = 255;

/// 存放琴弦对应的IO
pub struct ChordesIO{
    pub b1:gpiob::PB1<Output<PushPull>>,
    pub b0:gpiob::PB0<Output<PushPull>>,
    pub a7:gpioa::PA7<Output<PushPull>>,
    pub a6:gpioa::PA6<Output<PushPull>>,
    pub a5:gpioa::PA5<Output<PushPull>>,
    pub a4:gpioa::PA4<Output<PushPull>>,
    pub a3:gpioa::PA3<Output<PushPull>>,
    pub a2:gpioa::PA2<Output<PushPull>>,
    pub a1:gpioa::PA1<Output<PushPull>>,
    pub a0:gpioa::PA0<Output<PushPull>>,
    pub c15:gpioc::PC15<Output<PushPull>>,
    pub c14:gpioc::PC14<Output<PushPull>>,
    pub c13:gpioc::PC13<Output<PushPull>>,
    pub a8:gpioa::PA8<Output<PushPull>>,
    pub b14:gpiob::PB14<Output<PushPull>>,
    pub b15:gpiob::PB15<Output<PushPull>>,
    pub a11:gpioa::PA11<Output<PushPull>>,
}

impl ChordesIO{
    /// 点亮LED
    fn turn_on(&mut self, key:u8) -> bool{
        match self.switch(key, true){
            Ok(_) => true,
            _ => false
        }
    }
    /// 关闭LED
    fn turn_off(&mut self, key:u8) -> bool{
        match self.switch(key, false){
            Ok(_) => true,
            _ => false
        }
    }
 
    //蓝牙占用 B10,B11
    fn switch(&mut self, key:u8, on: bool) -> Result<(), core::convert::Infallible>{
        match key{
            NOTE_D6_IO => { if on {self.b1.set_high()} else {self.b1.set_low()} },
            NOTE_B5_IO => { if on {self.b0.set_high()} else {self.b0.set_low()}},
            NOTE_G5_IO => { if on {self.a7.set_high()} else {self.a7.set_low()}},
            NOTE_E5_IO => { if on {self.a6.set_high()} else {self.a6.set_low()}},
            NOTE_C5_IO => { if on {self.a5.set_high()} else {self.a5.set_low()}},
            NOTE_A4_IO => { if on {self.a4.set_high()} else {self.a4.set_low()}},
            NOTE_F4_IO => { if on {self.a3.set_high()} else {self.a3.set_low()}},
            NOTE_D4_IO => { if on {self.a2.set_high()} else {self.a2.set_low()}},
            NOTE_C4_IO => { if on {self.a1.set_high()} else {self.a1.set_low()}},
            NOTE_E4_IO => { if on {self.a0.set_high()} else {self.a0.set_low()}},
            NOTE_G4_IO => { if on {self.c15.set_high()} else {self.c15.set_low()}},
            NOTE_B4_IO => { if on {self.c13.set_high()} else {self.c13.set_low()}},
            NOTE_D5_IO => { if on {self.c14.set_high()} else {self.c14.set_low()}},
            NOTE_F5_IO => { if on {self.b14.set_high()} else {self.b14.set_low()}},
            NOTE_A5_IO => { if on {self.b15.set_high()} else {self.b15.set_low()}},
            NOTE_C6_IO => { if on {self.a8.set_high()} else {self.a8.set_low()}},
            NOTE_E6_IO => { if on {self.a11.set_high()} else {self.a11.set_low()}},
            _ => Ok(()),
        }
    }
}

/// 音符
#[derive(Debug, Clone)]
pub struct Note{
    /// 起始节拍(八分之一拍)
    start_beat: u16,
    /// 终止节拍(八分之一拍)
    end_beat: u16,
    /// 琴键
    key: u8,
    // 键名 (为了节省内存,不使用)
    // name: String
}

/// 曲子
#[derive(Debug)]
pub struct Song{
    /// 总共多少个八分之一拍
    total_beat: u16,
    /// 所有音符
    notes: Vec<Note>,
    /// 当前正在播放的音符
    cursor: usize
}

/// 音符播放器
pub struct Player{
    delay: Delay,
    chordes:ChordesIO,
    /// 节拍计数器
    current_beat: u16,
    /// 每小节几拍
    beat_per_group: u8,
    /// 每个八分之一拍多少微妙
    time_per_beat: u32,
    ended: bool,

    /// 主题
    theme: Option<Song>,

    /// 伴奏
    accompanies: Vec<Option<Song>>,
}

impl Player{
    pub fn new(chordes:ChordesIO, delay: Delay) -> Option<Player>{
        let mut player = Player{
            delay: delay,
            current_beat: 0,
            beat_per_group: 0,
            time_per_beat: 0,
            theme: None,
            chordes,
            ended: false,
            accompanies: Vec::new(),
        };
        player.reset()?;
        Some(player)
    }

    pub fn set_song(&mut self, songs:String) -> Option<()>{
        //分离歌曲信息、主题和伴奏
        let mut songs = songs.split("_");

        //歌曲信息
        let mut info = songs.next()?.split(",");
        let beat_per_group = parse::<u8>(info.next()?)?;//每小节几拍
        let beat_per_min = parse::<f32>(info.next()?)?;//每分钟多少拍

        // 计算每拍延迟(1ms=1000000us)
        // 一个u32可存储4294967295纳秒，即4294.9毫秒，足够一个八分之一节拍延时使用

        // 每拍毫秒数
        let time_per_total_beat = 60000.0 / beat_per_min;
        //每八分之一拍毫秒数
        let time_per_eighth_beat = time_per_total_beat / 8.0;
        //每八分之一拍微妙数
        let time_per_beat = (time_per_eighth_beat*1000.0) as u32;

        //至少有主题曲
        let theme_str = songs.next()?;
        self.theme = split_notes(theme_str);

        //解析所有伴奏曲
        while let Some(data) = songs.next(){
            self.accompanies.push(split_notes(data));
        }

        self.current_beat = 0;
        self.beat_per_group = beat_per_group;
        self.time_per_beat = time_per_beat;
        self.ended = false;
        Some(())
    }

    /// 播放主题曲和伴奏的下一个音符
    /// 此方法每隔八分之一拍调用一次
    pub fn play(&mut self) -> Option<Note>{
        if self.ended || self.theme.is_none(){
            self.ended = true;
            return None;
        }

        let theme = self.theme.as_mut().unwrap();

        //检查是否结束
        if self.current_beat == theme.total_beat{
            self.ended = true;
            return None;
        }

        let theme_note = play_note(self.current_beat, theme, &mut self.chordes);

        let accompanies:&mut Vec<Option<Song>> = self.accompanies.as_mut();
        for accompany in accompanies{
            if let Some(accompany) = accompany{
                let _ = play_note(self.current_beat, accompany, &mut self.chordes);
            }
        }
        
        //延时us(即微妙)
        self.delay.delay_us(self.time_per_beat);

        self.current_beat += 1;
        theme_note
    }

    pub fn ended(&self) -> bool{
        self.ended
    }

    pub fn get_theme(&self) -> Option<&Song>{
        self.theme.as_ref()
    }

    /// 重置
    pub fn reset(&mut self) -> Option<()>{
        self.current_beat = 0;
        // self.theme = None;
        // self.accompany = None;
        if let Some(theme) = self.theme.as_mut(){
            theme.cursor = 0;
        }
        self.ended = false;

        let accompanies:&mut Vec<Option<Song>> = self.accompanies.as_mut();
        for accompany in accompanies{
            if let Some(accompany) = accompany{
                accompany.cursor = 0;
            }
        }
        self.chordes.turn_off(NOTE_D6_IO);
        self.chordes.turn_off(NOTE_B5_IO);
        self.chordes.turn_off(NOTE_G5_IO);
        self.chordes.turn_off(NOTE_E5_IO);
        self.chordes.turn_off(NOTE_C5_IO);
        self.chordes.turn_off(NOTE_A4_IO);
        self.chordes.turn_off(NOTE_F4_IO);
        self.chordes.turn_off(NOTE_D4_IO);
        self.chordes.turn_off(NOTE_C4_IO);
        self.chordes.turn_off(NOTE_E4_IO);
        self.chordes.turn_off(NOTE_G4_IO);
        self.chordes.turn_off(NOTE_B4_IO);
        self.chordes.turn_off(NOTE_D5_IO);
        self.chordes.turn_off(NOTE_F5_IO);
        self.chordes.turn_off(NOTE_A5_IO);
        self.chordes.turn_off(NOTE_C6_IO);
        self.chordes.turn_off(NOTE_E6_IO);
        Some(())
    }
}

/// 播放下一个音符，返回开始的弹奏的音符
fn play_note(current_beat: u16, song:&mut Song, chordes: &mut ChordesIO) -> Option<Note>{
    let mut current_note = song.notes.get(song.cursor)?;
    //如果当前节拍大于当前音符的结束拍，关闭音符对应的LED，并切换音符
    if current_beat > current_note.end_beat{
        let _ = chordes.turn_off(current_note.key);
        song.cursor += 1;
        current_note = song.notes.get(song.cursor)?;
    }

    //如果当前节拍等于当前音符的起始拍，点亮LED
    if current_beat == current_note.start_beat{
        if chordes.turn_on(current_note.key){
            return Some(current_note.clone());
        }
    }
    None
}

/// 解析音符
fn split_notes(data: &str) -> Option<Song>{
    // 分别解析每小节的音符
    let mut notes:Vec<Note> = Vec::new();
    let parts = data.split("|");

    //统计总共多少个八分之一拍
    let mut total_beat_count = 0;
    for part in parts{
        for note in part.split(","){
            let mut symbols = note.chars();
            let key_name = symbols.next()?;
            let key = match key_name{
                NOTE_D6 => NOTE_D6_IO,
                NOTE_B5 => NOTE_B5_IO,
                NOTE_G5 => NOTE_G5_IO,
                NOTE_E5 => NOTE_E5_IO,
                NOTE_C5 => NOTE_C5_IO,
                NOTE_A4 => NOTE_A4_IO,
                NOTE_F4 => NOTE_F4_IO,
                NOTE_D4 => NOTE_D4_IO,
                NOTE_C4 => NOTE_C4_IO,
                NOTE_E4 => NOTE_E4_IO,
                NOTE_G4 => NOTE_G4_IO,
                NOTE_B4 => NOTE_B4_IO,
                NOTE_D5 => NOTE_D5_IO,
                NOTE_F5 => NOTE_F5_IO,
                NOTE_A5 => NOTE_A5_IO,
                NOTE_C6 => NOTE_C6_IO,
                NOTE_E6 => NOTE_E6_IO,
                _ => NOTE_NO_IO  // 默认为255空引脚，即不亮灯。包括延长符(-)、停止符(0)
            };
            let mut beat_count = 8; //默认为整拍
            if let Some(n) = symbols.next(){
                //读取音符节拍数
                beat_count = parse::<u16>(&n.to_string())?;
            }

            /*
                假设第一个音符是1拍，第二个和第三个音符都是半拍，第四个音符又是1拍
                那么第一个音符是从0开始，第二个音符是从8开始，第三个音符从12开始，第四个音符从16开始
            */
            notes.push(Note{
                start_beat: total_beat_count,
                end_beat: total_beat_count+beat_count-1,
                key,
                // name: key_name.to_string(),
            });

            //每个音符8个八分之一拍
            total_beat_count += beat_count;
        }
    }
    let song = Song{
        notes,
        cursor: 0,
        total_beat: total_beat_count
    };
    Some(song)
}

/// 将字符串转换未某种类型的数字
fn parse<F: FromStr>(s:&str) -> Option<F>{
    match s.parse::<F>(){
        Ok(a) => Some(a),
        _ => None
    }
}