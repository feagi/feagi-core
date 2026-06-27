#[doc(hidden)]
#[macro_export]
macro_rules! __create_flag_struct {
    (
        $(#[$struct_meta:meta])*
        $vis:vis struct $struct_name:ident($storage:ty) {
            $(($bit:literal, $flag:ident)),+ $(,)?
        }
    ) => {
        $(#[$struct_meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        #[repr(transparent)]
        $vis struct $struct_name($storage);

        $crate::paste::paste! {
            impl $struct_name {
                $vis const ALL_ZEROS: Self = Self(0);

                #[inline(always)]
                $vis const fn from_raw(bits: $storage) -> Self {
                    Self(bits)
                }

                #[inline(always)]
                $vis const fn raw(self) -> $storage {
                    self.0
                }

                $(
                    #[inline(always)]
                    $vis const fn [<read_ $flag>](&self) -> bool {
                        self.0 & ((1 as $storage) << $bit) != 0
                    }

                    #[inline(always)]
                    $vis fn [<set_ $flag>](&mut self, $flag: bool) {
                        if $flag {
                            self.0 |= (1 as $storage) << $bit;
                        } else {
                            self.0 &= !((1 as $storage) << $bit);
                        }
                    }

                    #[inline(always)]
                    $vis fn [<toggle_ $flag>](&mut self) {
                        self.0 ^= (1 as $storage) << $bit;
                    }
                )+
            }
        }
    };
}

#[macro_export]
macro_rules! create_8bit_flag_struct {
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
        $crate::__create_flag_struct! {
            $(#[$struct_meta])*
            $vis struct $struct_name(u8) {
                (0, $b0),
                (1, $b1),
                (2, $b2),
                (3, $b3),
                (4, $b4),
                (5, $b5),
                (6, $b6),
                (7, $b7),
            }
        }
    };
}

#[macro_export]
macro_rules! create_16bit_flag_struct {
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
            $b7:ident,
            $b8:ident,
            $b9:ident,
            $b10:ident,
            $b11:ident,
            $b12:ident,
            $b13:ident,
            $b14:ident,
            $b15:ident $(,)?
        )
    ) => {
        $crate::__create_flag_struct! {
            $(#[$struct_meta])*
            $vis struct $struct_name(u16) {
                (0, $b0),
                (1, $b1),
                (2, $b2),
                (3, $b3),
                (4, $b4),
                (5, $b5),
                (6, $b6),
                (7, $b7),
                (8, $b8),
                (9, $b9),
                (10, $b10),
                (11, $b11),
                (12, $b12),
                (13, $b13),
                (14, $b14),
                (15, $b15),
            }
        }
    };
}

#[macro_export]
macro_rules! create_32bit_flag_struct {
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
            $b7:ident,
            $b8:ident,
            $b9:ident,
            $b10:ident,
            $b11:ident,
            $b12:ident,
            $b13:ident,
            $b14:ident,
            $b15:ident,
            $b16:ident,
            $b17:ident,
            $b18:ident,
            $b19:ident,
            $b20:ident,
            $b21:ident,
            $b22:ident,
            $b23:ident,
            $b24:ident,
            $b25:ident,
            $b26:ident,
            $b27:ident,
            $b28:ident,
            $b29:ident,
            $b30:ident,
            $b31:ident $(,)?
        )
    ) => {
        $crate::__create_flag_struct! {
            $(#[$struct_meta])*
            $vis struct $struct_name(u32) {
                (0, $b0),
                (1, $b1),
                (2, $b2),
                (3, $b3),
                (4, $b4),
                (5, $b5),
                (6, $b6),
                (7, $b7),
                (8, $b8),
                (9, $b9),
                (10, $b10),
                (11, $b11),
                (12, $b12),
                (13, $b13),
                (14, $b14),
                (15, $b15),
                (16, $b16),
                (17, $b17),
                (18, $b18),
                (19, $b19),
                (20, $b20),
                (21, $b21),
                (22, $b22),
                (23, $b23),
                (24, $b24),
                (25, $b25),
                (26, $b26),
                (27, $b27),
                (28, $b28),
                (29, $b29),
                (30, $b30),
                (31, $b31),
            }
        }
    };
}