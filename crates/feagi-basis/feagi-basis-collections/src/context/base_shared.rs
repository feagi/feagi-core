use core::marker::PhantomData;
use serde::de::{self, IgnoredAny, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BaseSharedSpatial<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> {
    data: [QI; NUM_DIMS],
}

impl<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> serde::Serialize for BaseSharedSpatial<QI, NUM_DIMS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(NUM_DIMS))?;
        for value in self.data.iter() {
            seq.serialize_element(value)?;
        }
        seq.end()
    }
}

impl<'de, QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> serde::Deserialize<'de> for BaseSharedSpatial<QI, NUM_DIMS>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UnsignedIntegerSpatialVisitor<QI, const NUM_DIMS: usize>(PhantomData<QI>);

        impl<'de, QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> Visitor<'de>
        for UnsignedIntegerSpatialVisitor<QI, NUM_DIMS>
        {
            type Value = BaseSharedSpatial<QI, NUM_DIMS>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(formatter, "a sequence with exactly {} elements", NUM_DIMS)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values: [Option<QI>; NUM_DIMS] = core::array::from_fn(|_| None);
                for (idx, slot) in values.iter_mut().enumerate() {
                    *slot = Some(
                        seq.next_element()?
                            .ok_or_else(|| de::Error::invalid_length(idx, &self))?,
                    );
                }
                if seq.next_element::<IgnoredAny>()?.is_some() {
                    return Err(de::Error::invalid_length(NUM_DIMS + 1, &self));
                }
                let data = values.map(|value| value.expect("visitor filled every spatial element"));
                Ok(BaseSharedSpatial { data })
            }
        }

        deserializer.deserialize_seq(UnsignedIntegerSpatialVisitor::<QI, NUM_DIMS>(PhantomData))
    }
}