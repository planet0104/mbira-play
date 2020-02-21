#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;
mod player;
use cortex_m::asm;
use cortex_m_rt::entry;
use panic_semihosting as _;
use stm32f1xx_hal::{delay::Delay, pac, prelude::*, serial::{Config, Serial}};
//use cortex_m_semihosting::hprintln; //半主机功能

use alloc::string::String;
use alloc_cortex_m::CortexMHeap;
use core::{alloc::Layout, fmt::Write};

use player::*;

/// 堆内存分配器
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();
/// 堆内存 16K
const HEAP_SIZE: usize = 16384;

#[entry]
fn main() -> ! {
    // 初始化内存分配器
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }
    //hprintln!("Start!").unwrap();

    // stm32f1xx设备专用外设
    let device = pac::Peripherals::take().unwrap();

    // 取得原始Flash和RCC设备的所有权，并将它们转换为相应的HAL(硬件抽象层)结构
    let mut flash = device.FLASH.constrain();
    let mut rcc = device.RCC.constrain();

    // 冻结系统中所有时钟的配置，并将冻结的频率存储在“clocks”中
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // 准备备用功能I/O寄存器
    let mut afio = device.AFIO.constrain(&mut rcc.apb2);

    //Cortex-M 外设
    let core = cortex_m::Peripherals::take().unwrap();

    //初始化计时器SysTick
    let delay = Delay::new(core.SYST, clocks);

    // 准备GPIO外设
    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
    let mut gpioc = device.GPIOC.split(&mut rcc.apb2);

    // 串口设备USART3
    // 将pb10配置为push_pull输出，这将是tx引脚
    let tx = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
    // 取得pb11控制权
    let rx = gpiob.pb11;

    //蓝牙占用 B10,B11
    let chordes = ChordesIO {
        b1: gpiob.pb1.into_push_pull_output(&mut gpiob.crl),
        b0: gpiob.pb0.into_push_pull_output(&mut gpiob.crl),
        a7: gpioa.pa7.into_push_pull_output(&mut gpioa.crl),
        a6: gpioa.pa6.into_push_pull_output(&mut gpioa.crl),
        a5: gpioa.pa5.into_push_pull_output(&mut gpioa.crl),
        a4: gpioa.pa4.into_push_pull_output(&mut gpioa.crl),
        a3: gpioa.pa3.into_push_pull_output(&mut gpioa.crl),
        a2: gpioa.pa2.into_push_pull_output(&mut gpioa.crl),
        a1: gpioa.pa1.into_push_pull_output(&mut gpioa.crl),
        a0: gpioa.pa0.into_push_pull_output(&mut gpioa.crl),
        c15: gpioc.pc15.into_push_pull_output(&mut gpioc.crh),
        c14: gpioc.pc14.into_push_pull_output(&mut gpioc.crh),
        c13: gpioc.pc13.into_push_pull_output(&mut gpioc.crh),
        //A9、A10是烧录用的，换成
        b14: gpiob.pb14.into_push_pull_output(&mut gpiob.crh), // B14
        b15: gpiob.pb15.into_push_pull_output(&mut gpiob.crh), // B15
        a8: gpioa.pa8.into_push_pull_output(&mut gpioa.crh),   // A8
        a11: gpioa.pa11.into_push_pull_output(&mut gpioa.crh), // A11
    };

    // 设置usart设备。 通过USART寄存器和 tx/rx 引脚获得所有权。 其余寄存器用于启用和配置设备。
    let serial = Serial::usart3(
        device.USART3,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()),
        clocks,
        &mut rcc.apb1,
    );

    // 将串行结构拆分为接收和发送部分
    let (mut tx, mut rx) = serial.split();

    let mut received = String::new();
    let mut music_str = String::new();

    let mut player = Player::new(chordes, delay).unwrap();

    //注意！loop中如果有hprintln、delay等会导致串口接收失败
    loop {
        //播放时无法接收蓝牙数据，因为play会有延迟！
        if !player.ended() {
            let _ = player.play();
        } else {
            if player.get_theme().is_some() {
                //重新播放
                player.reset();
                //答复小程序
                // writeln!(tx, "START").unwrap();
            }
        }

        //接收串口数据
        if let Ok(c) = rx.read() {
            received.push(c as char);
            //在数据接收的过程中不能写，否则会造成数据丢失。
            if received.ends_with("+") {
                let musics = received.replace("+", "");
                received = String::new();
                music_str.push_str(&musics);
            }
            //如果是#结束，开始播放
            if received.ends_with("#") {
                let musics = received.replace("#", "");
                received = String::new();
                music_str.push_str(&musics);

                let musics = music_str;
                player.set_song(musics);
                music_str = String::new();
                writeln!(tx, "START").unwrap();
            }
        }
    }
}

// 内存不足执行此处代码(调试用)
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    // hprintln!("Out of Memory!!!").unwrap();
    asm::bkpt();
    loop {}
}