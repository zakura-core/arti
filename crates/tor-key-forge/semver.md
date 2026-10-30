
BREAKING: `ItemType` now extends `std::any::Any`, and `EncodableItem` inherits
that bound through `ItemType`, replacing the `downcast_rs::Downcast` supertrait.
The generated `is`, `downcast`, `downcast_rc`, `downcast_ref` and `downcast_mut`
methods on both trait objects, and inherited `into_any`, `into_any_rc`,
`as_any` and `as_any_mut`, are removed. Upcast the owned/reference trait object
to `dyn Any` and use its standard checked downcasts. `ErasedKey` remains
`Box<dyn ItemType>`.
