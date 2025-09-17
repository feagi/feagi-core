use crate::{define_signed_percentage, define_unsigned_percentage, define_xy_percentage_coordinates};


define_unsigned_percentage!(Percentage, "A percentage value, from 0 to 100%");
define_signed_percentage!(SignedPercentage, "A signed percentage value, from -100 to 100%");
define_xy_percentage_coordinates!(Percentage2D, Percentage, "Percentage2D", "Represents 2 percentages over 2 dimensions, going from 0 - 100%");
define_xy_percentage_coordinates!(SignedPercentage2D, SignedPercentage, "SignedPercentage2D", "Represents 2 signed percentages over 2 dimensions, going from -100 - 100%");

