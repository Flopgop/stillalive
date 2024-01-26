#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::{Debug, Formatter};
use core::arch::x86_64::_rdtsc;
use cmos_rtc::{
    ReadRTC,
    Time,
};
use crate::speaker::Speaker;
use buddy_system_allocator::LockedHeap;

mod vga_buffer;
mod frequencies;
mod speaker;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::new();

static mut HEAP: [u8; 131072] = [0; 131072];


static STILL_ALIVE: &[u8; 1394] = b"This was a triumph
I'm making a note here
\"Huge success\"
It's hard to overstate my satisfaction
Aperture Science
We do what we must because we can
For the good of all of us
Except the ones who are dead
But there's no sense crying over every mistake
You just keep on trying till you run out of cake
And the science gets done and you make a neat gun
For the people who are still alive
I'm not even angry
I'm being so sincere right now
Even though you broke my heart
And killed me
And tore me to pieces
And threw every piece into a fire
As they burned it hurt because
I was so happy for you
Now, these points of data make a beautiful line
And we're out of beta, we're releasing on time
So I'm GLaD I got burned, think of all the things we learned
For the people who are still alive
Go ahead and leave me
I think I'd prefer to stay inside
Maybe you'll find someone else
To help you
Maybe Black Mesa?
That was a joke, ha-ha, fat chance
Anyway, this cake is great
It's so delicious and moist
Look at me, still talking when there's science to do
When I look out there, it makes me GLaD I'm not you
I've experiments to run, there is research to be done
On the people who are still alive
And believe me, I am still alive
I'm doing science and I'm still alive
I feel fantastic and I'm still alive
While you're dying, I'll be still alive
And when you're dead, I will be still alive
Still alive
Still alive";

static PRINT_FREQUENCY: f64 = 50.0;
static ENDLINE_FREQUENCY: f64 = 1500.0;

fn get_timestamp(cmos: &mut ReadRTC) -> Time {
    cmos.read()
}

fn rdtsc() -> u64 {
    unsafe {
        _rdtsc()
    }
}

fn cur_time_millis(freq: u64) -> f64 {
    (rdtsc() as f64) / freq as f64 * 1000.0
}

pub fn to_seconds(time: &Time) -> u64 {
    // Assuming each month has 30 days for simplicity
    const SECONDS_IN_MINUTE: u64 = 60;
    const SECONDS_IN_HOUR: u64 = 60 * SECONDS_IN_MINUTE;
    const SECONDS_IN_DAY: u64 = 24 * SECONDS_IN_HOUR;
    const DAYS_IN_MONTH: u64 = 30;

    let years_in_seconds = u64::from(time.century * 100 + time.year) * 365 * SECONDS_IN_DAY;
    let months_in_seconds = u64::from(time.month) * DAYS_IN_MONTH * SECONDS_IN_DAY;
    let days_in_seconds = u64::from(time.day - 1) * SECONDS_IN_DAY;
    let hours_in_seconds = u64::from(time.hour) * SECONDS_IN_HOUR;
    let minutes_in_seconds = u64::from(time.minute) * SECONDS_IN_MINUTE;
    let seconds = u64::from(time.second);

    years_in_seconds + months_in_seconds + days_in_seconds + hours_in_seconds + minutes_in_seconds + seconds
}

struct TypedDebugWrapper<'a, T: ?Sized>(&'a T);

impl<T: Debug> Debug for TypedDebugWrapper<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}::{:?}", core::any::type_name::<T>(), self.0)
    }
}

trait TypedDebug: Debug {
    fn typed_debug(&self) -> TypedDebugWrapper<'_, Self> {
        TypedDebugWrapper(self)
    }
}

impl<T: ?Sized + Debug> TypedDebug for T {}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    let mut cmos = ReadRTC::new(0x00, 0x00);
    let mut speaker = Speaker::new();
    unsafe {
        // Give the allocator some memory to allocate.
        HEAP_ALLOCATOR.lock().init(HEAP.as_mut_ptr() as usize, HEAP.len());
    }

    println!("Measuring Frequency...");
    // measure frequency
    let pre = rdtsc();
    let cur_time = get_timestamp(&mut cmos);
    loop {
        if to_seconds(&get_timestamp(&mut cmos))-to_seconds(&cur_time) == 1 {break;}
    }
    let post = rdtsc();

    let frequency = post-pre;
    println!("Frequency in Hz: {}", frequency);
    let frequency_ghz = frequency as f64/1000000000.0 as f64;
    println!("Frequency in Ghz: {}", frequency_ghz);
    println!("Cur time millis: {}ms", cur_time_millis(frequency));

    let mut char_counter = 0;
    let mut tick_counter = 0;
    let mut last_char_time = cur_time_millis(frequency);
    let mut last_note_time = cur_time_millis(frequency) + frequencies::NOTES[tick_counter].start_time*1000.0;
    let start_time = cur_time_millis(frequency);
    loop {
        if tick_counter < frequencies::NOTES.len() {
            let note = &frequencies::NOTES[tick_counter];
            
            if cur_time_millis(frequency) - start_time >= note.start_time*1000.0 {
                speaker.play_sound(note.frequency as u32);
                if cur_time_millis(frequency) - last_note_time >= note.duration*1000.0 {
                    speaker.nosound();
                    last_note_time = start_time + note.start_time*1000.0 + note.duration*1000.0;
                    tick_counter += 1;
                }
            }
        }
        if char_counter < STILL_ALIVE.len() {
            let time = cur_time_millis(frequency);
            if time-last_char_time >= PRINT_FREQUENCY {
                let next_char = STILL_ALIVE[char_counter] as char;
                if next_char != '\n' {
                    print!("{}", next_char);
                    char_counter += 1;
                    last_char_time = cur_time_millis(frequency);
                } else if time-&last_char_time >= ENDLINE_FREQUENCY {
                    println!(" ");
                
                    char_counter += 1;
                    last_char_time = cur_time_millis(frequency);
                }
            }
        }
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}