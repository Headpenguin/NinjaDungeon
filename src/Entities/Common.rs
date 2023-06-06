use super::Traits::Counter;
use super::TypedID;

pub struct DeathCounter<'a, T: Counter> {
    dst: TypedID<'a, T>,
}
