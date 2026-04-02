
#[macro_export]
macro_rules! define_bit_packed_u8_flags {
    (
        $(#[$struct_meta:meta])*
        $vis:vis struct $struct_name:ident(
            $b0:ident,
            $b1:ident,
            $b2:ident,
            $b3:ident,
            $b4:ident,
            $b5:ident,
            $b6:ident,
            $b7:ident $(,)?
        )
    ) => {
        $(#[$struct_meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        $vis struct $struct_name(u8);

        ::paste::paste! {
            impl $struct_name {
                $vis const ALL_ZEROS: Self = Self(0);

                #[inline(always)]
                $vis const fn from_raw(bits: u8) -> Self {
                    Self(bits)
                }

                #[inline(always)]
                $vis const fn raw(self) -> u8 {
                    self.0
                }

                #[inline(always)]
                $vis fn [<is_ $b0>](&self) -> bool {
                    self.0 & 0x01 != 0
                }
                #[inline(always)]
                $vis fn [<is_ $b1>](&self) -> bool {
                    self.0 & 0x02 != 0
                }
                #[inline(always)]
                $vis fn [<is_ $b2>](&self) -> bool {
                    self.0 & 0x04 != 0
                }
                #[inline(always)]
                $vis fn [<is_ $b3>](&self) -> bool {
                    self.0 & 0x08 != 0
                }
                #[inline(always)]
                $vis fn [<is_ $b4>](&self) -> bool {
                    self.0 & 0x10 != 0
                }
                #[inline(always)]
                $vis fn [<is_ $b5>](&self) -> bool {
                    self.0 & 0x20 != 0
                }
                #[inline(always)]
                $vis fn [<is_ $b6>](&self) -> bool {
                    self.0 & 0x40 != 0
                }
                #[inline(always)]
                $vis fn [<is_ $b7>](&self) -> bool {
                    self.0 & 0x80 != 0
                }

                #[inline(always)]
                $vis fn [<set_ $b0>](&mut self, $b0: bool) {
                    if $b0 { self.0 |= 0x01; } else { self.0 &= 0xFE; }
                }
                #[inline(always)]
                $vis fn [<set_ $b1>](&mut self, $b1: bool) {
                    if $b1 { self.0 |= 0x02; } else { self.0 &= 0xFD; }
                }
                #[inline(always)]
                $vis fn [<set_ $b2>](&mut self, $b2: bool) {
                    if $b2 { self.0 |= 0x04; } else { self.0 &= 0xFB; }
                }
                #[inline(always)]
                $vis fn [<set_ $b3>](&mut self, $b3: bool) {
                    if $b3 { self.0 |= 0x08; } else { self.0 &= 0xF7; }
                }
                #[inline(always)]
                $vis fn [<set_ $b4>](&mut self, $b4: bool) {
                    if $b4 { self.0 |= 0x10; } else { self.0 &= 0xEF; }
                }
                #[inline(always)]
                $vis fn [<set_ $b5>](&mut self, $b5: bool) {
                    if $b5 { self.0 |= 0x20; } else { self.0 &= 0xDF; }
                }
                #[inline(always)]
                $vis fn [<set_ $b6>](&mut self, $b6: bool) {
                    if $b6 { self.0 |= 0x40; } else { self.0 &= 0xBF; }
                }
                #[inline(always)]
                $vis fn [<set_ $b7>](&mut self, $b7: bool) {
                    if $b7 { self.0 |= 0x80; } else { self.0 &= 0x7F; }
                }

                #[inline(always)]
                $vis fn [<toggle_ $b0>](&mut self) {
                    self.0 ^= 0x01;
                }
                #[inline(always)]
                $vis fn [<toggle_ $b1>](&mut self) {
                    self.0 ^= 0x02;
                }
                #[inline(always)]
                $vis fn [<toggle_ $b2>](&mut self) {
                    self.0 ^= 0x04;
                }
                #[inline(always)]
                $vis fn [<toggle_ $b3>](&mut self) {
                    self.0 ^= 0x08;
                }
                #[inline(always)]
                $vis fn [<toggle_ $b4>](&mut self) {
                    self.0 ^= 0x10;
                }
                #[inline(always)]
                $vis fn [<toggle_ $b5>](&mut self) {
                    self.0 ^= 0x20;
                }
                #[inline(always)]
                $vis fn [<toggle_ $b6>](&mut self) {
                    self.0 ^= 0x40;
                }
                #[inline(always)]
                $vis fn [<toggle_ $b7>](&mut self) {
                    self.0 ^= 0x80;
                }
            }
        }
    };
}