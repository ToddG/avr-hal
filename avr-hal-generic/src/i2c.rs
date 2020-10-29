//! I2C Implementations

/// TWI Status Codes
pub mod twi_status {
    // The status codes defined in the C header are meant to be used with the
    // masked status value: (TWSR & TW_STATUS_MASK).  In our case, svd2rust
    // already added code to shift it to just the status value, so all status
    // codes need to be shifted to the right as well.

    /// Start condition transmitted
    pub const TW_START: u8 = 0x08 >> 3;

    /// Repeated start condition transmitted
    pub const TW_REP_START: u8 = 0x10 >> 3;

    // Master Transmitter -----------------------------------------------------
    /// SLA+W transmitted, ACK received
    pub const TW_MT_SLA_ACK: u8 = 0x18 >> 3;

    /// SLA+W transmitted, NACK received
    pub const TW_MT_SLA_NACK: u8 = 0x20 >> 3;

    /// Data transmitted, ACK received
    pub const TW_MT_DATA_ACK: u8 = 0x28 >> 3;

    /// Data transmitted, NACK received
    pub const TW_MT_DATA_NACK: u8 = 0x30 >> 3;

    /// Arbitration lost in SLA+W or data
    pub const TW_MT_ARB_LOST: u8 = 0x38 >> 3;

    // Master Receiver --------------------------------------------------------
    /// Arbitration lost in SLA+R or NACK
    pub const TW_MR_ARB_LOST: u8 = 0x38 >> 3;

    /// SLA+R transmitted, ACK received
    pub const TW_MR_SLA_ACK: u8 = 0x40 >> 3;

    /// SLA+R transmitted, NACK received
    pub const TW_MR_SLA_NACK: u8 = 0x48 >> 3;

    /// Data received, ACK returned
    pub const TW_MR_DATA_ACK: u8 = 0x50 >> 3;

    /// Data received, NACK returned
    pub const TW_MR_DATA_NACK: u8 = 0x58 >> 3;

    // Slave Transmitter ------------------------------------------------------
    /// SLA+R received, ACK returned
    pub const TW_ST_SLA_ACK: u8 = 0xA8 >> 3;

    /// Arbitration lost in SLA+RW, SLA+R received, ACK returned
    pub const TW_ST_ARB_LOST_SLA_ACK: u8 = 0xB0 >> 3;

    /// Data transmitted, ACK received
    pub const TW_ST_DATA_ACK: u8 = 0xB8 >> 3;

    /// Data transmitted, NACK received
    pub const TW_ST_DATA_NACK: u8 = 0xC0 >> 3;

    /// Last data byte transmitted, ACK received
    pub const TW_ST_LAST_DATA: u8 = 0xC8 >> 3;

    // Slave Receiver ---------------------------------------------------------
    /// SLA+W received, ACK returned
    pub const TW_SR_SLA_ACK: u8 = 0x60 >> 3;

    /// Arbitration lost in SLA+RW, SLA+W received, ACK returned
    pub const TW_SR_ARB_LOST_SLA_ACK: u8 = 0x68 >> 3;

    /// General call received, ACK returned
    pub const TW_SR_GCALL_ACK: u8 = 0x70 >> 3;

    /// Arbitration lost in SLA+RW, general call received, ACK returned
    pub const TW_SR_ARB_LOST_GCALL_ACK: u8 = 0x78 >> 3;

    /// Data received, ACK returned
    pub const TW_SR_DATA_ACK: u8 = 0x80 >> 3;

    /// Data received, NACK returned
    pub const TW_SR_DATA_NACK: u8 = 0x88 >> 3;

    /// General call data received, ACK returned
    pub const TW_SR_GCALL_DATA_ACK: u8 = 0x90 >> 3;

    /// General call data received, NACK returned
    pub const TW_SR_GCALL_DATA_NACK: u8 = 0x98 >> 3;

    /// Stop or repeated start condition received while selected
    pub const TW_SR_STOP: u8 = 0xA0 >> 3;

    // Misc -------------------------------------------------------------------
    /// No state information available
    pub const TW_NO_INFO: u8 = 0xF8 >> 3;

    /// Illegal start or stop condition
    pub const TW_BUS_ERROR: u8 = 0x00 >> 3;
}

/// I2C Error
#[derive(ufmt::derive::uDebug, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Error {
    /// Lost arbitration while trying to acquire bus
    ArbitrationLost,
    /// No slave answered for this address or a slave replied NACK
    AddressNack,
    /// Slave replied NACK to sent data
    DataNack,
    /// A bus-error occurred
    BusError,
    /// An unknown error occurred.  The bus might be in an unknown state.
    Unknown,
}

/// I2C Transfer Direction
#[derive(ufmt::derive::uDebug, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    /// Write to a slave (LSB is 0)
    Write,
    /// Read from a slave (LSB is 1)
    Read,
}

#[doc(hidden)]
pub fn i2cdetect<W: ufmt::uWrite, F>(s: &mut W, mut f: F) -> Result<(), W::Error>
where
    // Detection function
    F: FnMut(u8) -> Result<bool, Error>,
{
    s.write_str("\
-    0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f\r\n\
00:      ")?;

    fn u4_to_hex(b: u8) -> char {
        match b {
            x if x < 0xa => (0x30 + x).into(),
            x if x < 0x10 => (0x57 + x).into(),
            _ => '?',
        }
    }

    for addr in 0x02..=0x77 {
        let (ah, al) = (u4_to_hex(addr >> 4), u4_to_hex(addr & 0xf));

        if addr % 0x10 == 0 {
            s.write_str("\r\n")?;
            s.write_char(ah)?;
            s.write_str("0:")?;
        }

        match f(addr) {
            Ok(true) => {
                s.write_char(' ')?;
                s.write_char(ah)?;
                s.write_char(al)?;
            },
            Ok(false) => {
                s.write_str(" --")?;
            },
            Err(e) => {
                s.write_str(" E")?;
                s.write_char(u4_to_hex(e as u8))?;
            },
        }
    }

    s.write_str("\r\n")?;

    Ok(())
}

#[doc(hidden)]
pub type I2cFloating = crate::port::mode::Input<crate::port::mode::Floating>;
#[doc(hidden)]
pub type I2cPullUp = crate::port::mode::Input<crate::port::mode::PullUp>;

/// Implement I2C traits for a TWI peripheral.
///
/// Macro generates the following structs:
///
/// * I2c (master)
/// * I2cSlave (slave)
#[macro_export]
macro_rules! impl_twi_i2c {
    (
        $(#[$i2c_attr:meta])*
        pub struct $I2c:ident {
            peripheral: $I2C:ty,
            pins: {
                sda: $sdamod:ident::$SDA:ident,
                scl: $sclmod:ident::$SCL:ident,
            },
            registers: {
                control: $twcr:ident {
                    enable: $twen:ident,
                    ack: $twea:ident,
                    int: $twint:ident,
                    start: $twstart:ident,
                    stop: $twstop:ident,
                },
                status: $twsr:ident {
                    prescaler: $twps:ident,
                    status: $tws:ident,
                },
                bitrate: $twbr:ident,
                data: $twdr:ident,
                address: $twar:ident,
            },
        }
    ) => {$crate::paste::paste! {
        /// I2C Master
        $(#[$i2c_attr])*
        pub struct [<$I2c Master>]<CLOCK: $crate::clock::Clock, M> {
            p: $I2C,
            _clock: ::core::marker::PhantomData<CLOCK>,
            sda: $sdamod::$SDA<M>,
            scl: $sclmod::$SCL<M>,
        }

        impl<CLOCK, M> [<$I2c Master>]<CLOCK, M>
        where
            CLOCK: $crate::clock::Clock,
        {
            fn initialize_i2c(
                p: &$I2C,
                speed: &u32,
                ){
                // Calculate the TWI Bit Rate Register (TWBR)
                //
                // (datasheet) 22.5.2 Bit Rate Generator Unit
                //
                //      This unit controls the period of SCL when operating in a Master mode. The SCL period is controlled by settings
                //      in the TWI Bit Rate Register (TWBR) and the Prescaler bits in the TWI Status Register (TWSR). Slave operation
                //      does not depend on Bit Rate or Prescaler settings, but the CPU clock frequency in the Slave must be at least 16
                //      times higher than the SCL frequency. Note that slaves may prolong the SCL low period, thereby reducing the
                //      average TWI bus clock period.
                let twbr = ((CLOCK::FREQ / speed) - 16) / 2;
                p.$twbr.write(|w| unsafe { w.bits(twbr as u8) });

                // Disable prescaler
                //
                // (datasheet) 22.5.2 Bit Rate Generator Unit
                //
                //      SCL frequency = CPU Clock Frequency
                //                     --------------------------------
                //                     16 + 2(TWBR) * (Prescaler Value)
                //
                // Setting the prescaler to 1 makes the math easy.
                p.$twsr.write(|w| w.$twps().prescaler_1());
            }
        }

        impl<CLOCK> [<$I2c Master>]<CLOCK, $crate::i2c::I2cPullUp>
        where
            CLOCK: $crate::clock::Clock,
        {
            /// Initialize the I2C bus
            ///
            /// `new()` will enable the internal pull-ups to comply with the I2C
            /// specification.  If you have external pull-ups connected, please
            /// use `new_with_external_pullup()` instead.
            pub fn new(
                p: $I2C,
                sda: $sdamod::$SDA<$crate::port::mode::Input<$crate::port::mode::PullUp>>,
                scl: $sclmod::$SCL<$crate::port::mode::Input<$crate::port::mode::PullUp>>,
                speed: u32,
            ) -> [<$I2c Master>]<CLOCK, $crate::i2c::I2cPullUp> {

                // init i2c
                [<$I2c Master>]::<CLOCK, $crate::i2c::I2cPullUp>::initialize_i2c(&p, &speed);

                [<$I2c Master>] {
                    p,
                    sda,
                    scl,
                    _clock: ::core::marker::PhantomData,
                }
            }
        }

        impl<CLOCK> [<$I2c Master>]<CLOCK, $crate::i2c::I2cFloating>
        where
            CLOCK: $crate::clock::Clock,
        {
            /// Initialize the I2C bus, without enabling internal pull-ups
            ///
            /// This function should be used if your hardware design includes
            /// pull-up resistors outside the MCU.  If you do not have these,
            /// please use `new()` instead.
            pub fn new_with_external_pullup(
                p: $I2C,
                sda: $sdamod::$SDA<$crate::port::mode::Input<$crate::port::mode::Floating>>,
                scl: $sclmod::$SCL<$crate::port::mode::Input<$crate::port::mode::Floating>>,
                speed: u32,
            ) -> [<$I2c Master>]<CLOCK, $crate::i2c::I2cFloating> {

                // init i2c
                [<$I2c Master>]::<CLOCK, $crate::i2c::I2cFloating>::initialize_i2c(&p, &speed);

                [<$I2c Master>] {
                    p,
                    sda,
                    scl,
                    _clock: ::core::marker::PhantomData,
                }
            }
        }

        impl<CLOCK, M> [<$I2c Master>]<CLOCK, M>
        where
            CLOCK: $crate::clock::Clock,
        {
            /// Check whether a slave answers ACK for a given address
            ///
            /// Note that some devices might not respond to both read and write
            /// operations.
            pub fn ping_slave(
                &mut self,
                addr: u8,
                dir: $crate::i2c::Direction,
            ) -> Result<bool, $crate::i2c::Error> {
                match self.start(addr, dir) {
                    Err($crate::i2c::Error::AddressNack) => Ok(false),
                    Err(e) => Err(e),
                    Ok(()) => {
                        self.stop();
                        Ok(true)
                    },
                }
            }

            fn start(
                &mut self,
                addr: u8,
                dir: $crate::i2c::Direction,
            ) -> Result<(), $crate::i2c::Error> {
                // Write start condition
                //
                // (datsheet) 22.9.2 TWCR – TWI Control Register
                //
                //    Bit 2 – TWEN: TWI Enable Bit
                //
                //    The TWEN bit enables TWI operation and activates the TWI interface. When TWEN is written to one, the TWI
                //    takes control over the I/O pins connected to the SCL and SDA pins, enabling the slew-rate limiters and spike
                //    filters. If this bit is written to zero, the TWI is switched off and all TWI transmissions are terminated, regardless of
                //    any ongoing operation.
                //
                //    Bit 7 – TWINT: TWI Interrupt Flag
                //
                //    This bit is set by hardware when the TWI has finished its current job and expects application software response.
                //    If the I-bit in SREG and TWIE in TWCR are set, the MCU will jump to the TWI Interrupt Vector. While the TWINT
                //    Flag is set, the SCL low period is stretched. The TWINT Flag must be cleared by software by writing a logic one
                //    to it. Note that this flag is not automatically cleared by hardware when executing the interrupt routine. Also note
                //    that clearing this flag starts the operation of the TWI, so all accesses to the TWI Address Register (TWAR), TWI
                //    Status Register (TWSR), and TWI Data Register (TWDR) must be complete before clearing this flag.
                //
                //
                //    Bit 5 – TWSTA: TWI START Condition Bit
                //
                //    The application writes the TWSTA bit to one when it desires to become a Master on the 2-wire Serial Bus. The
                //    TWI hardware checks if the bus is available, and generates a START condition on the bus if it is free. However,
                //    if the bus is not free, the TWI waits until a STOP condition is detected, and then generates a new START
                //    condition to claim the bus Master status. TWSTA must be cleared by software when the START condition has
                //    been transmitted.
                //
                //    NOTE: `twstart` macro token is defined as `twsta` in chips/[CHIP]/src/lib.rs
                self.p.$twcr.write(|w| w
                    .$twen().set_bit()
                    .$twint().set_bit()
                    .$twstart().set_bit()
                );
                // NOTE: why do we need to wait here? `wait()` is just reading twcr.twint to see if
                // the bit is set...but it was just set in the line above...
                self.wait();

                // Validate status
                match self.p.$twsr.read().$tws().bits() {
                      $crate::i2c::twi_status::TW_START
                    | $crate::i2c::twi_status::TW_REP_START => (),
                      $crate::i2c::twi_status::TW_MT_ARB_LOST
                    | $crate::i2c::twi_status::TW_MR_ARB_LOST => {
                        return Err($crate::i2c::Error::ArbitrationLost);
                    },
                    $crate::i2c::twi_status::TW_BUS_ERROR => {
                        return Err($crate::i2c::Error::BusError);
                    },
                    _ => {
                        return Err($crate::i2c::Error::Unknown);
                    },
                }

                // Send slave address
                let dirbit = if dir == $crate::i2c::Direction::Read { 1 } else { 0 };
                let rawaddr = (addr << 1) | dirbit;
                self.p.$twdr.write(|w| unsafe { w.bits(rawaddr) });
                self.transact();

                // Check if the slave responded
                match self.p.$twsr.read().$tws().bits() {
                      $crate::i2c::twi_status::TW_MT_SLA_ACK
                    | $crate::i2c::twi_status::TW_MR_SLA_ACK => (),
                      $crate::i2c::twi_status::TW_MT_SLA_NACK
                    | $crate::i2c::twi_status::TW_MR_SLA_NACK => {
                        // Stop the transaction if it did not respond
                        self.stop();
                        return Err($crate::i2c::Error::AddressNack);
                    },
                      $crate::i2c::twi_status::TW_MT_ARB_LOST
                    | $crate::i2c::twi_status::TW_MR_ARB_LOST => {
                        return Err($crate::i2c::Error::ArbitrationLost);
                    },
                    $crate::i2c::twi_status::TW_BUS_ERROR => {
                        return Err($crate::i2c::Error::BusError);
                    },
                    _ => {
                        return Err($crate::i2c::Error::Unknown);
                    },
                }

                Ok(())
            }

            fn wait(&mut self) {
                // (datasheet) 22.9.2 TWCR – TWI Control Register
                //
                //    Bit 7 – TWINT: TWI Interrupt Flag
                //
                //    This bit is set by hardware when the TWI has finished its current job and expects application software response.
                //    If the I-bit in SREG and TWIE in TWCR are set, the MCU will jump to the TWI Interrupt Vector. While the TWINT
                //    Flag is set, the SCL low period is stretched. The TWINT Flag must be cleared by software by writing a logic one
                //    to it. Note that this flag is not automatically cleared by hardware when executing the interrupt routine. Also note
                //    that clearing this flag starts the operation of the TWI, so all accesses to the TWI Address Register (TWAR), TWI
                //    Status Register (TWSR), and TWI Data Register (TWDR) must be complete before clearing this flag
                //
                // NOTE: In the start() fn above sets the `twint` flag, which `I think` is setting it to true:
                //    
                //      .$twint().set_bit()
                //
                // But 'true' === 1 which in the language of the spec here means `cleared`...
                //
                //    The TWINT Flag must be cleared by software by writing a logic one to it.
                //
                // Here in avr-device/src/generic.rs
                //
                //    impl<FI> FieldReader<bool, FI> {
                //        /// Value of the field as raw bits.
                //        #[inline(always)]
                //        pub fn bit(&self) -> bool {
                //            self.bits
                //        }
                //        /// Returns `true` if the bit is clear (0).
                //        #[inline(always)]
                //        pub fn bit_is_clear(&self) -> bool {
                //            !self.bit()
                //        }
                //        /// Returns `true` if the bit is set (1).
                //        #[inline(always)]
                //        pub fn bit_is_set(&self) -> bool {
                //            self.bit()
                //        }
                //    }
                //
                while self.p.$twcr.read().$twint().bit_is_clear() { }
            }

            fn transact(&mut self) {
                self.p.$twcr.write(|w| w.$twen().set_bit().$twint().set_bit());
                while self.p.$twcr.read().$twint().bit_is_clear() { }
            }

            fn write_data(&mut self, bytes: &[u8]) -> Result<(), $crate::i2c::Error> {
                for byte in bytes {
                    self.p.$twdr.write(|w| unsafe { w.bits(*byte) });
                    self.transact();

                    match self.p.$twsr.read().$tws().bits() {
                        $crate::i2c::twi_status::TW_MT_DATA_ACK => (),
                        $crate::i2c::twi_status::TW_MT_DATA_NACK => {
                            self.stop();
                            return Err($crate::i2c::Error::DataNack);
                        },
                        $crate::i2c::twi_status::TW_MT_ARB_LOST => {
                            return Err($crate::i2c::Error::ArbitrationLost);
                        },
                        $crate::i2c::twi_status::TW_BUS_ERROR => {
                            return Err($crate::i2c::Error::BusError);
                        },
                        _ => {
                            return Err($crate::i2c::Error::Unknown);
                        },
                    }
                }
                Ok(())
            }

            fn read_data(&mut self, buffer: &mut [u8]) -> Result<(), $crate::i2c::Error> {
                let last = buffer.len() - 1;
                for (i, byte) in buffer.iter_mut().enumerate() {
                    if i != last {
                        self.p.$twcr.write(|w| w.$twint().set_bit().$twen().set_bit().$twea().set_bit());
                        self.wait();
                    } else {
                        self.p.$twcr.write(|w| w.$twint().set_bit().$twen().set_bit());
                        self.wait();
                    }

                    match self.p.$twsr.read().$tws().bits() {
                          $crate::i2c::twi_status::TW_MR_DATA_ACK
                        | $crate::i2c::twi_status::TW_MR_DATA_NACK => (),
                        $crate::i2c::twi_status::TW_MR_ARB_LOST => {
                            return Err($crate::i2c::Error::ArbitrationLost);
                        },
                        $crate::i2c::twi_status::TW_BUS_ERROR => {
                            return Err($crate::i2c::Error::BusError);
                        },
                        _ => {
                            return Err($crate::i2c::Error::Unknown);
                        },
                    }

                    *byte = self.p.$twdr.read().bits();
                }
                Ok(())
            }

            fn stop(&mut self) {
                // Send stop
                self.p.$twcr.write(|w| w
                    .$twen().set_bit()
                    .$twint().set_bit()
                    .$twstop().set_bit()
                );
            }
        }

        impl<CLOCK, M> [<$I2c Master>]<CLOCK, M>
        where
            CLOCK: $crate::clock::Clock,
            $crate::delay::Delay<CLOCK>: $crate::hal::blocking::delay::DelayMs<u16>,
        {
            /// Output an `i2cdetect`-like summary of connected slaves to a serial device
            ///
            /// Note that output for `Read` and `Write` might differ.
            pub fn i2cdetect<W: $crate::ufmt::uWrite>(
                &mut self,
                w: &mut W,
                dir: $crate::i2c::Direction,
            ) -> Result<(), W::Error> {
                let mut delay = $crate::delay::Delay::<CLOCK>::new();
                $crate::i2c::i2cdetect(w, |a| {
                    use $crate::prelude::*;

                    delay.delay_ms(10u16);
                    self.ping_slave(a, dir)
                })
            }
        }


        impl<CLOCK, M> $crate::hal::blocking::i2c::Write for [<$I2c Master>]<CLOCK, M>
        where
            CLOCK: $crate::clock::Clock,
        {
            type Error = $crate::i2c::Error;

            fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
                self.start(address, $crate::i2c::Direction::Write)?;
                self.write_data(bytes)?;
                self.stop();
                Ok(())
            }
        }

        impl<CLOCK, M> $crate::hal::blocking::i2c::Read for [<$I2c Master>]<CLOCK, M>
        where
            CLOCK: $crate::clock::Clock,
        {
            type Error = $crate::i2c::Error;

            fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
                self.start(address, $crate::i2c::Direction::Read)?;
                self.read_data(buffer)?;
                self.stop();
                Ok(())
            }
        }

        impl<CLOCK, M> $crate::hal::blocking::i2c::WriteRead for [<$I2c Master>]<CLOCK, M>
        where
            CLOCK: $crate::clock::Clock,
        {
            type Error = $crate::i2c::Error;

            fn write_read(
                &mut self,
                address: u8,
                bytes: &[u8],
                buffer: &mut [u8],
            ) -> Result<(), Self::Error> {
                self.start(address, $crate::i2c::Direction::Write)?;
                self.write_data(bytes)?;
                self.start(address, $crate::i2c::Direction::Read)?;
                self.read_data(buffer)?;
                self.stop();
                Ok(())
            }
        }

        /// I2C Slave
        ///
        /// I2C Slave is similar to the I2C Master. The similarities are that both structs have the
        /// `new()` and `new_with_external_pullup()` constructors. However the Slave constructors
        /// do not need to pass the speed/bitrate as that is determined by the Master. However, the
        /// Slave constructors do need to pass the slave address.
        $(#[$i2c_attr])*
        pub struct [<$I2c Slave>]<CLOCK: $crate::clock::Clock, M> {
            p: $I2C,
            _clock: ::core::marker::PhantomData<CLOCK>,
            sda: $sdamod::$SDA<M>,
            scl: $sclmod::$SCL<M>,
            address: u8,
        }

        impl<CLOCK> [<$I2c Slave>]<CLOCK, $crate::i2c::I2cPullUp>
        where
            CLOCK: $crate::clock::Clock,
        {
            /// Initialize the I2C bus
            ///
            /// `new()` will enable the internal pull-ups to comply with the I2C
            /// specification.  If you have external pull-ups connected, please
            /// use `new_with_external_pullup()` instead.
            pub fn new(
                p: $I2C,
                sda: $sdamod::$SDA<$crate::port::mode::Input<$crate::port::mode::PullUp>>,
                scl: $sclmod::$SCL<$crate::port::mode::Input<$crate::port::mode::PullUp>>,
                address: u8,
            ) -> [<$I2c Slave>]<CLOCK, $crate::i2c::I2cPullUp> {
                [<$I2c Slave>] {
                    p,
                    sda,
                    scl,
                    _clock: ::core::marker::PhantomData,
                    address,
                }
            }
        }

        impl<CLOCK> [<$I2c Slave>]<CLOCK, $crate::i2c::I2cFloating>
        where
            CLOCK: $crate::clock::Clock,
        {
            /// Initialize the I2C bus, without enabling internal pull-ups
            ///
            /// This function should be used if your hardware design includes
            /// pull-up resistors outside the MCU.  If you do not have these,
            /// please use `new()` instead.
            pub fn new_with_external_pullup(
                p: $I2C,
                sda: $sdamod::$SDA<$crate::port::mode::Input<$crate::port::mode::Floating>>,
                scl: $sclmod::$SCL<$crate::port::mode::Input<$crate::port::mode::Floating>>,
                address: u8,
            ) -> [<$I2c Slave>]<CLOCK, $crate::i2c::I2cFloating> {
                [<$I2c Slave>] {
                    p,
                    sda,
                    scl,
                    _clock: ::core::marker::PhantomData,
                    address,
                }
            }
        }

        impl<CLOCK, M> [<$I2c Slave>]<CLOCK, M>
        where
            CLOCK: $crate::clock::Clock,
        {
            pub fn start(&mut self) {
                self.p.$twar.write(|w| unsafe {w.bits(self.address)});
            }
        }

    }};
}
