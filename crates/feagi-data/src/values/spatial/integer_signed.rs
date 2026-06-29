
// TODO add equals, hash

//region Internal macros

// Yes this is mostly a duplicate but I don't care (no quantization here)

macro_rules! create_signed_coordinate {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $num_dimensions:expr,
        $( ($index:tt, $field:ident) ),+ $(,)?
    ) => {
        $(#[$meta])*
        pub struct $struct_name {
            inner: [i32; $num_dimensions],
        }

        ::paste::paste! {
            impl $struct_name {
                $vis fn new( $( $field: i32 ),+ ) -> Self {
                    $struct_name {
                        inner: [ $( $field ),+ ]
                    }
                }

                $(
                    $vis fn [<get_ $field>](&self) -> &i32 {
                        &self.inner[$index]
                    }
                )+
                $(
                    $vis fn [<get_ $field _mut>](&mut self) -> &mut i32 {
                        &mut self.inner[$index]
                    }
                )+
            }
        }
    };
}

//endregion

create_signed_coordinate!(
    /// A signed (positive or negative) coordinate in 2D
    pub SignedCoordinate2D,
    2,
    (0, x), (1, y)
);

create_signed_coordinate!(
    /// A signed (positive or negative) coordinate in 3D
    pub SignedCoordinate3D,
    3,
    (0, x), (1, y), (2, z)
);

create_signed_coordinate!(
    /// A signed (positive or negative) coordinate in 4D
    pub SignedCoordinate4D,
    4,
    (0, x), (1, y), (2, z), (3, w)
);
