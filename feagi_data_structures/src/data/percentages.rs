use crate::{define_signed_percentage, define_unsigned_percentage, define_2d_signed_or_unsigned_percentages, define_3d_signed_or_unsigned_percentages, define_4d_signed_or_unsigned_percentages};


define_unsigned_percentage!(Percentage, "A percentage value, from 0 to 100%");
define_signed_percentage!(SignedPercentage, "A signed percentage value, from -100 to 100%");
define_2d_signed_or_unsigned_percentages!(Percentage2D, Percentage, "Percentage2D", "Represents 2 percentages over 2 dimensions, going from 0 - 100%");
define_2d_signed_or_unsigned_percentages!(SignedPercentage2D, SignedPercentage, "SignedPercentage2D", "Represents 2 signed percentages over 2 dimensions, going from -100 - 100%");
define_3d_signed_or_unsigned_percentages!(Percentage3D, Percentage, "Percentage3D", "Represents 3 percentages over 3 dimensions, going from 0 - 100%");
define_3d_signed_or_unsigned_percentages!(SignedPercentage3D, SignedPercentage, "SignedPercentage3D", "Represents 3 signed percentages over 3 dimensions, going from -100 - 100%");
define_4d_signed_or_unsigned_percentages!(Percentage4D, Percentage, "Percentage4D", "Represents 4 percentages over 4 dimensions, going from 0 - 100%");
define_4d_signed_or_unsigned_percentages!(SignedPercentage4D, SignedPercentage, "SignedPercentage4D", "Represents 4 signed percentages over 4 dimensions, going from -100 - 100%");
