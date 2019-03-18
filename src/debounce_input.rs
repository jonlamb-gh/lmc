use cortex_m::asm;
use embedded_hal::digital::InputPin;

const DELAY: u32 = 10;

pub trait DebounceInput {
    fn is_low_debounce(&self) -> bool;
}

impl<T> DebounceInput for T
where
    T: InputPin,
{
    fn is_low_debounce(&self) -> bool {
        if self.is_low() {
            asm::delay(DELAY);
            if self.is_low() {
                asm::delay(DELAY);
                return self.is_low();
            }
        }

        false
    }
}
